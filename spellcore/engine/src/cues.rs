//! Cue list: GO manual, `wait` antes de comecar, `follow` automatico ao terminar, fade linear
//! entre snapshots. Porte de `spellcaster/timeline/cues.py`.
//!
//! Snapshot = {"universo/endereco": [valores]} no .spell; vira `((universo, endereco), valores)`.
//! O estado corrente e' escrito nos Universes por `update()`, sem alocar por frame: os `Vec` de
//! valores sao reaproveitados (so' o primeiro toque em cada chave aloca).

use crate::universe::Universes;
use serde_json::Value;

type Key = (u16, u16);
type Snap = Vec<(Key, Vec<f64>)>;

/// `"1/100"` -> `(1, 100)`; `"100"` -> `(1, 100)`.
// ponytail: chave malformada devolve None e o Cue a descarta com aviso (o Python levanta
// ValueError) ; virar erro de carga quando o .spell tiver validacao de schema.
pub fn key(s: &str) -> Option<Key> {
    match s.split_once('/') {
        Some((u, a)) => Some((u.trim().parse().ok()?, a.trim().parse().ok()?)),
        None => Some((1, s.trim().parse().ok()?)),
    }
}

pub struct Cue {
    pub name: String,
    pub fade: f64,
    /// Atraso entre o gatilho e o inicio do fade.
    pub wait: f64,
    /// Ao terminar, dispara a proxima.
    pub follow: bool,
    pub values: Vec<(Key, Vec<f64>)>,
}

impl Cue {
    pub fn new(spec: &Value) -> Cue {
        let num = |k: &str| spec.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let mut values = Vec::new();
        // ponytail: a ordem das chaves e' a do Map do serde_json (alfabetica), nao a do arquivo
        // ; so' muda o resultado se duas chaves da MESMA cue escreverem no mesmo canal
        // ; ligar a feature "preserve_order" do serde_json se algum show depender disso.
        if let Some(m) = spec.get("values").and_then(|v| v.as_object()) {
            for (k, v) in m {
                match key(k) {
                    Some(kk) => values.push((
                        kk,
                        match v {
                            Value::Array(a) => {
                                a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()
                            }
                            other => vec![other.as_f64().unwrap_or(0.0)],
                        },
                    )),
                    None => eprintln!("aviso: cue com chave invalida: {}", k),
                }
            }
        }
        Cue {
            name: spec
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            fade: num("fade"),
            wait: num("wait"),
            follow: spec
                .get("follow")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            values,
        }
    }
}

pub struct CueList {
    cues: Vec<Cue>,
    state: Snap,
    from: Snap,
    index: i32,
    cur: Option<usize>,
    t0: f64,
    /// (indice, instante do gatilho) — vira cue corrente quando passa o `wait`.
    pending: Option<(usize, f64)>,
}

impl CueList {
    pub fn new(specs: &[Value]) -> CueList {
        CueList {
            cues: specs.iter().map(Cue::new).collect(),
            state: Vec::new(),
            from: Vec::new(),
            index: -1,
            cur: None,
            t0: 0.0,
            pending: None,
        }
    }

    pub fn len(&self) -> usize {
        self.cues.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    /// Indice da ultima cue disparada; -1 = nenhuma.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Dispara a proxima cue (ou a de indice dado). O fade comeca depois do `wait` dela.
    pub fn go(&mut self, t: f64, index: Option<usize>) -> bool {
        let i = match index {
            Some(i) => i as i64,
            None => self.index as i64 + 1,
        };
        if i < 0 || i >= self.cues.len() as i64 {
            return false;
        }
        self.pending = Some((i as usize, t));
        true
    }

    /// Avanca o fade e escreve o snapshot corrente nos Universes.
    pub fn update(&mut self, t: f64, uni: &mut Universes) {
        if let Some((i, t0)) = self.pending {
            if t >= t0 + self.cues[i].wait {
                self.pending = None;
                self.index = i as i32;
                self.cur = Some(i);
                self.t0 = t;
                self.from.clone_from(&self.state); // ponto de partida do fade
            }
        }
        if let Some(i) = self.cur {
            let CueList {
                cues,
                state,
                from,
                t0,
                ..
            } = self;
            let c = &cues[i];
            let u = if c.fade <= 0.0 {
                1.0
            } else {
                ((t - *t0) / c.fade).clamp(0.0, 1.0)
            };
            for (k, v) in &c.values {
                // valor de partida: o que estava no estado, completado com 0.0 (igual ao Python)
                let a = from
                    .iter()
                    .find(|(kk, _)| kk == k)
                    .map(|(_, x)| x.as_slice())
                    .unwrap_or(&[]);
                let dst = slot(state, *k);
                dst.clear();
                if u >= 1.0 {
                    dst.extend_from_slice(v);
                } else {
                    for (j, y) in v.iter().enumerate() {
                        let x = a.get(j).copied().unwrap_or(0.0);
                        dst.push(x + (y - x) * u);
                    }
                }
            }
            if u >= 1.0 {
                let follow = c.follow;
                self.cur = None;
                if follow {
                    self.go(t, None);
                }
            }
        }
        for ((u, a), v) in &self.state {
            uni.get_or_create(*u).set(*a, v);
        }
    }

    pub fn reset(&mut self) {
        self.state.clear();
        self.from.clear();
        self.index = -1;
        self.cur = None;
        self.pending = None;
    }
}

/// Entrada de `snap` para a chave, criando-a no fim se ainda nao existir.
// ponytail: busca linear ; uma cue tem dezenas de chaves, nao milhares — virar Vec ordenado
// com busca binaria se um show passar a ter centenas de canais por cue.
fn slot(snap: &mut Snap, k: Key) -> &mut Vec<f64> {
    match snap.iter().position(|(kk, _)| *kk == k) {
        Some(i) => &mut snap[i].1,
        None => {
            snap.push((k, Vec::new()));
            &mut snap.last_mut().expect("acabou de entrar").1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_nas_duas_formas() {
        assert_eq!(key("1/100"), Some((1, 100)));
        assert_eq!(key("100"), Some((1, 100)));
        assert_eq!(key("3/512"), Some((3, 512)));
        assert_eq!(key(" 2 / 7 "), Some((2, 7)));
        assert_eq!(key("x"), None);
        assert_eq!(key("1/x"), None);
    }

    fn ch(uni: &Universes, u: u16, a: u16) -> u8 {
        uni.get(u).expect("universo").data[(a - 1) as usize]
    }

    /// wait + fade + follow com os valores na mao.
    #[test]
    fn wait_fade_follow() {
        let specs = vec![
            json!({"name": "a", "wait": 1.0, "fade": 2.0, "follow": true,
                   "values": {"1/1": [100]}}),
            json!({"name": "b", "fade": 0.0, "values": {"1/1": [200], "2/5": [10, 20]}}),
        ];
        let mut cl = CueList::new(&specs);
        let mut uni = Universes::new();
        uni.get_or_create(1);
        assert_eq!(cl.len(), 2);
        assert_eq!(cl.index(), -1);

        // sem GO nada acontece
        cl.update(0.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 0);

        assert!(cl.go(0.0, None));
        // dentro do wait: ainda nao comecou
        cl.update(0.5, &mut uni);
        assert_eq!(cl.index(), -1);
        assert_eq!(ch(&uni, 1, 1), 0);

        // t = 1.0: passou o wait, o fade comeca aqui (u = 0)
        cl.update(1.0, &mut uni);
        assert_eq!(cl.index(), 0);
        assert_eq!(ch(&uni, 1, 1), 0);
        // meio do fade de 2 s
        cl.update(2.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 50);
        cl.update(2.5, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 75);
        // fim do fade: valor cheio e o follow arma a proxima
        cl.update(3.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 100);
        assert_eq!(cl.index(), 0, "a proxima so' entra no update seguinte");
        // cue b: fade 0 => u = 1 de cara
        cl.update(3.1, &mut uni);
        assert_eq!(cl.index(), 1);
        assert_eq!(ch(&uni, 1, 1), 200);
        assert_eq!(ch(&uni, 2, 5), 10);
        assert_eq!(ch(&uni, 2, 6), 20);
        // fim da lista: o follow da ultima nao tem para onde ir
        cl.update(4.0, &mut uni);
        assert_eq!(cl.index(), 1);

        // GO com indice explicito volta para a cue 0
        assert!(cl.go(4.0, Some(0)));
        cl.update(5.0, &mut uni); // wait 1.0 cumprido: u = 0, parte de 200
        assert_eq!(ch(&uni, 1, 1), 200);
        cl.update(6.0, &mut uni); // metade do caminho de 200 para 100
        assert_eq!(ch(&uni, 1, 1), 150);

        assert!(!cl.go(6.0, Some(9)), "indice fora da lista");
        cl.reset();
        assert_eq!(cl.index(), -1);
        cl.update(7.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 150, "reset nao apaga o que ja foi escrito");
    }

    #[test]
    fn fade_zero_e_valor_solto() {
        let specs = vec![json!({"values": {"7": 255, "1/2": [1, 2, 3]}})];
        let mut cl = CueList::new(&specs);
        let mut uni = Universes::new();
        cl.go(0.0, None);
        cl.update(0.0, &mut uni);
        assert_eq!(ch(&uni, 1, 7), 255, "chave sem universo e' o universo 1");
        assert_eq!(ch(&uni, 1, 2), 1);
        assert_eq!(ch(&uni, 1, 4), 3);
        assert_eq!(cl.index(), 0);
    }

    #[test]
    fn lista_vazia_e_chave_invalida() {
        let mut cl = CueList::new(&[]);
        let mut uni = Universes::new();
        assert!(cl.is_empty());
        assert!(!cl.go(0.0, None));
        cl.update(0.0, &mut uni);
        assert!(uni.is_empty());

        let cl = CueList::new(&[json!({"values": {"nao/e/chave": [1]}})]);
        assert_eq!(cl.len(), 1, "chave invalida nao derruba a cue");
    }
}
