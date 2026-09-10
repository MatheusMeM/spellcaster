//! Gravacao: track `dmx` armado le o universo de ENTRADA (`input.rs`) a cada frame e escreve
//! keyframe so' quando o valor muda. A escrita passa pelo mesmo funil da edicao manual
//! (`edit::key_put`), entao `rev` sobe e o barramento avisa os clientes.
//!
//! Comandos: `rec_arm {track, on}` e `rec_state`. PARAR o transporte desarma tudo: a thread de
//! transporte chama `disarm()` na borda de entrada em Stop, nao enquanto parado — armar com o
//! transporte parado e so' depois dar play e' o caminho normal do operador.

use crate::edit;
use crate::input::Inputs;
use crate::registry::{lock, NoArgs, Registry};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Um track armado: onde ler a entrada e o que ja' foi gravado.
struct Arm {
    track: usize,
    universe: u16,
    address: u16,
    width: usize,
    /// Ultimo valor gravado; vazio = nada gravado ainda (o primeiro frame sempre grava).
    last: Vec<u8>,
}

static ARMED: Mutex<Vec<Arm>> = Mutex::new(Vec::new());
/// Espelho do tamanho de `ARMED`: a thread de transporte le isto por frame e so' pega o mutex
/// quando ha algo armado.
static N: AtomicUsize = AtomicUsize::new(0);

/// Arma ou desarma um track. O tipo, o universo e o endereco vem do show ABERTO (`edit`), nao da
/// timeline do player: e' no show aberto que o keyframe vai ser gravado.
pub fn arm(indice: usize, on: bool) -> Result<Value, String> {
    // O `track_dmx` pega o lock de OPEN e o solta antes de ARMED entrar: a ordem dos dois locks
    // e' sempre ARMED -> OPEN (o `tick` faz assim), nunca o contrario.
    let info = if on {
        Some(edit::track_dmx(indice)?)
    } else {
        None
    };
    let mut g = lock(&ARMED);
    g.retain(|a| a.track != indice);
    if let Some((universe, address, width)) = info {
        g.push(Arm {
            track: indice,
            universe,
            address,
            width,
            last: Vec::new(),
        });
    }
    N.store(g.len(), Ordering::Relaxed);
    Ok(json!({"track": indice, "on": on, "tracks": lista(&g)}))
}

fn lista(g: &[Arm]) -> Vec<usize> {
    g.iter().map(|a| a.track).collect()
}

/// Tracks armados, em ordem de armamento.
pub fn state() -> Value {
    let g = lock(&ARMED);
    json!({"recording": !g.is_empty(), "tracks": lista(&g)})
}

/// Desarma tudo (o transporte parou). Barato quando nada esta armado.
pub fn disarm() {
    if N.load(Ordering::Relaxed) == 0 {
        return;
    }
    lock(&ARMED).clear();
    N.store(0, Ordering::Relaxed);
}

/// Um frame de gravacao: para cada track armado, le o universo de entrada e grava keyframe em
/// `t` quando os canais do track mudam. Roda no passo 3 do frame (efeito colateral), antes das
/// cues, e nao toca nos Universes de saida.
// ponytail: um keyframe por MUDANCA, sem thinning ; um fader andando a 60 fps deixa 60 keys por
// segundo. Entra reducao de curva (Douglas-Peucker) quando o arquivo gravado incomodar.
pub fn tick(t: f64, inputs: &Inputs) {
    if N.load(Ordering::Relaxed) == 0 {
        return;
    }
    let mut g = lock(&ARMED);
    let mut caiu: Vec<usize> = Vec::new();
    for (n, a) in g.iter_mut().enumerate() {
        let Some(frame) = inputs.get(a.universe) else {
            continue;
        };
        let i = (a.address - 1) as usize;
        let novo = &frame[i..(i + a.width).min(512)];
        if a.last == novo {
            continue;
        }
        a.last.clear();
        a.last.extend_from_slice(novo);
        let v = if novo.len() == 1 {
            json!(novo[0])
        } else {
            json!(novo)
        };
        if let Err(e) = edit::key_put(a.track, t, v, "linear") {
            // Falhar aqui e' o track ter sumido (`track_del`) ou o show ter trocado: o indice
            // guardado no arme nao existe mais. Desarma UMA vez, em vez de repetir o erro a cada
            // frame com um indice velho.
            eprintln!("gravacao do track {}: {} - desarmado", a.track, e);
            caiu.push(n);
        }
    }
    if !caiu.is_empty() {
        for n in caiu.into_iter().rev() {
            g.remove(n);
        }
        N.store(g.len(), Ordering::Relaxed);
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct RecArmArgs {
    /// Indice do track em `tracks` (tem que ser `dmx` ou `artnet`).
    pub track: usize,
    /// true arma, false desarma.
    #[serde(default = "sim")]
    pub on: bool,
}

fn sim() -> bool {
    true
}

pub fn register(r: &mut Registry) {
    r.add::<RecArmArgs>(
        "rec_arm",
        "Arma (ou desarma) a gravacao de um track dmx: com o transporte tocando, o universo de entrada vira keyframe.",
        |a| arm(a.track, a.on),
    );
    r.add::<NoArgs>(
        "rec_state",
        "Tracks armados para gravacao neste processo.",
        |_| Ok(state()),
    );
}

// Teste em `engine/tests/rec.rs`, binario proprio: armar consulta o show ABERTO, e `OPEN` e' um
// por processo — o teste aqui abriria um show e derrubaria os outros testes do mesmo binario.
