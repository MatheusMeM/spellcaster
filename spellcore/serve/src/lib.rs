//! Barramento do Spellcaster: um processo toca o hardware, e toda pagina e toda IA falam com
//! ele. HTTP para o registry, o show e os arquivos da GUI; WebSocket JSON-RPC para comando e
//! evento; frame binario para o monitor DMX; MCP streamable em `/mcp`.
//!
//! Contrato palavra por palavra em `spellcore/README.md`, secao "`serve` — barramento". Aqui so'
//! ha transporte: quem tem logica e' o `Registry`, e este crate nunca chama o engine por fora
//! dele (a unica excecao e' `player::current()`, que so' e' lido para publicar o transporte).

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use engine::hook::FrameHook;
use engine::registry::Registry;
use engine::universe::Universes;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Frame binario do monitor: `topic:u8 | universe:u16 LE | 512 bytes`.
const TOPIC_DMX: u8 = 1;
const FRAME: usize = 1 + 2 + 512;

/// Teto do monitor: 40 Hz (um show a 60 fps nao manda 60 frames por universo para a GUI).
const MONITOR_MS: u64 = 25;

#[derive(Clone)]
enum Out {
    Text(String),
    Bin(Vec<u8>),
}

struct St {
    reg: Arc<Registry>,
    tx: broadcast::Sender<Out>,
    dir: PathBuf,
}

impl St {
    fn evento(&self, event: &str, data: Value) {
        let _ = self.tx.send(Out::Text(texto(event, data)));
    }
}

fn texto(event: &str, data: Value) -> String {
    json!({"event": event, "data": data}).to_string()
}

/// O sink de evento do graph vive na CLI e nao tem `St`: `out.widget` chega ao WS por aqui.
/// Sem `serve` no ar, nao faz nada.
// ponytail: um `serve()` por processo ; um segundo `serve()` no mesmo processo fica com o sender
// do primeiro (OnceLock nao troca) e seus widgets saem no barramento errado. Passar o `St` ate' o
// sink da CLI quando houver dois barramentos vivos.
static TX: OnceLock<broadcast::Sender<Out>> = OnceLock::new();

/// `{"event":"widget","data":{"id","prop","value"}}` para todo cliente do barramento.
pub fn widget(id: &str, prop: &str, value: f64) {
    if let Some(tx) = TX.get() {
        let _ = tx.send(Out::Text(texto(
            "widget",
            json!({"id": id, "prop": prop, "value": value}),
        )));
    }
}

// --------------------------------------------------------------------- HTTP

async fn commands(State(st): State<Arc<St>>) -> Response {
    axum::Json(st.reg.schema()).into_response()
}

async fn show_get(State(st): State<Arc<St>>) -> Response {
    match st.reg.call("show_get", json!({"full": true})) {
        Ok(v) => axum::Json(v).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

// ponytail: quatro tipos ; o `--dir` tem html, js, css e .spell — imagem e fonte entram quando
// alguma pagina trouxer uma.
fn mime(p: &str) -> &'static str {
    match p.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "spell" => "application/json",
        _ => "application/octet-stream",
    }
}

/// Arquivo de `--dir`. Limite de confianca: so' segmentos simples relativos. `..`, segmento
/// vazio, `\` e `:` (unidade, ADS do Windows) sao recusados ANTES de tocar o disco, e o caminho
/// nao e' percent-decodificado — `%2e%2e` fica literal e vira 404, nao subida de diretorio.
// ponytail: `--dir` padrao e' a raiz do repo, entao tudo que esta nela (`.git` inclusive) e'
// legivel em 127.0.0.1 ; filtrar por lista quando o `--host` e o token existirem.
async fn estatico(State(st): State<Arc<St>>, uri: Uri) -> Response {
    let p = uri.path().trim_start_matches('/');
    let p = if p.is_empty() { "index.html" } else { p };
    let ruim = p.contains(':')
        || p.contains('\\')
        || p.split('/').any(|s| s.is_empty() || s == "." || s == "..");
    if ruim {
        return (StatusCode::FORBIDDEN, "caminho recusado").into_response();
    }
    match tokio::fs::read(st.dir.join(p)).await {
        Ok(b) => ([(header::CONTENT_TYPE, mime(p))], b).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, format!("nao ha {}", p)).into_response(),
    }
}

/// A GUI recarrega do servidor, sempre: um `.js` velho em cache e' uma noite de depuracao.
async fn no_store(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let mut r = next.run(req).await;
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

// ----------------------------------------------------------------- WebSocket

/// Uma request do WS. `{"id":7,"cmd":"locate","args":{...}}` responde
/// `{"id":7,"result":...,"rev":n}` ou `{"id":7,"error":"texto","rev":n}`; TODA resposta carrega a
/// revisao do show, e' por ela que a pagina sabe se o `show` que chegar e' eco proprio ou de
/// outro cliente. Erro de comando NAO derruba a conexao.
async fn request(st: &Arc<St>, txt: &str) -> String {
    let v: Value = match serde_json::from_str(txt) {
        Ok(v) => v,
        Err(e) => {
            return json!({"id": null, "error": format!("json invalido: {}", e),
                          "rev": engine::edit::rev()})
            .to_string()
        }
    };
    let id = v.get("id").cloned().unwrap_or(Value::Null);
    let cmd = v
        .get("cmd")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let args = v.get("args").cloned().unwrap_or_else(|| json!({}));
    if st.reg.get(&cmd).is_none() {
        return json!({"id": id, "error": format!("comando desconhecido: {}", cmd),
                      "rev": engine::edit::rev()})
        .to_string();
    }
    // `play_show` bloqueia ate o fim do show: roda em thread e a resposta volta na hora (a mesma
    // lista BACKGROUND do MCP; um so' lugar decide o que e' comando de longa duracao).
    if mcp::background(&cmd) {
        let (reg, nome, s) = (st.reg.clone(), cmd.clone(), st.clone());
        std::thread::spawn(move || {
            if let Err(e) = reg.call(&nome, args) {
                s.evento("log", json!({ "text": format!("{}: {}", nome, e) }));
            }
        });
        let t = format!("{} iniciado em background", cmd);
        return json!({"id": id, "result": t, "rev": engine::edit::rev()}).to_string();
    }
    // comando do registry e' sincrono e pode bloquear (arquivo, varredura de rede)
    let (reg, nome) = (st.reg.clone(), cmd.clone());
    // Nao ha' lista de comandos de leitura: quem diz se o show mudou e' o proprio contador do
    // engine. Comando que edita sobe `edit::rev()`; comando que so' le, nao.
    // ponytail: antes/depois de rev em volta de chamada concorrente: edicao de B durante leitura
    // de A sai como eco de A ; fila unica de comandos quando duas paginas editarem junto.
    let antes = engine::edit::rev();
    let r = tokio::task::spawn_blocking(move || reg.call(&nome, args)).await;
    let depois = engine::edit::rev();
    match r {
        Ok(Ok(v)) => {
            if depois != antes {
                st.evento("show", json!({ "rev": depois }));
            }
            json!({"id": id, "result": v, "rev": depois}).to_string()
        }
        Ok(Err(e)) => json!({"id": id, "error": e, "rev": depois}).to_string(),
        Err(e) => {
            json!({"id": id, "error": format!("{} caiu: {}", cmd, e), "rev": depois}).to_string()
        }
    }
}

async fn upgrade(State(st): State<Arc<St>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |s| cliente(s, st))
}

async fn cliente(mut sock: WebSocket, st: Arc<St>) {
    let mut rx = st.tx.subscribe();
    loop {
        tokio::select! {
            m = sock.recv() => match m {
                Some(Ok(Message::Text(t))) => {
                    let r = request(&st, t.as_str()).await;
                    if sock.send(Message::Text(r.into())).await.is_err() {
                        break;
                    }
                }
                Some(Ok(_)) => {} // ping/pong/binario do cliente: o contrato nao usa
                _ => break,
            },
            o = rx.recv() => match o {
                Ok(Out::Text(t)) => {
                    if sock.send(Message::Text(t.into())).await.is_err() {
                        break;
                    }
                }
                Ok(Out::Bin(b)) => {
                    if sock.send(Message::Binary(b.into())).await.is_err() {
                        break;
                    }
                }
                // ponytail: cliente lento perde frame de monitor e segue ; o dado e' o proximo
                // frame, nao o historico — nada a reenviar.
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
}

/// Transporte no WS: a cada mudanca, e a 10 Hz enquanto o player anda (tocando, o `t` muda todo
/// frame, entao comparar o ultimo JSON ja' da' as duas coisas).
// ponytail: sondagem a 10 Hz ; virar aviso do proprio player se a GUI pedir menos latencia.
async fn transporte(st: Arc<St>) {
    let mut last = String::new();
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if st.tx.receiver_count() == 0 {
            continue;
        }
        let Some(h) = engine::player::current() else {
            last.clear();
            continue;
        };
        let v = serde_json::to_value(h.state()).unwrap_or(Value::Null);
        let s = v.to_string();
        if s != last {
            last = s;
            st.evento("transport", v);
        }
    }
}

// ------------------------------------------------------------------ monitor

/// Copia para o WS os universos do frame. Instalado como gancho global, roda depois dos ganchos
/// do show: e' o frame que de fato sai na rede.
struct Monitor {
    tx: broadcast::Sender<Out>,
    at: Instant,
}

impl FrameHook for Monitor {
    // ponytail: manda todo universo do frame, sem cache do ultimo enviado ; o teto de 40 Hz em
    // loopback ja' segura a banda — cachear entra se um show com dezenas de universos parados
    // aparecer no perfil.
    fn frame(&mut self, _t: f64, uni: &mut Universes) {
        if self.tx.receiver_count() == 0 || self.at.elapsed() < Duration::from_millis(MONITOR_MS) {
            return;
        }
        self.at = Instant::now();
        for u in uni.iter() {
            let mut b = Vec::with_capacity(FRAME);
            b.push(TOPIC_DMX);
            b.extend_from_slice(&u.number.to_le_bytes());
            b.extend_from_slice(&u.data);
            let _ = self.tx.send(Out::Bin(b));
        }
    }
}

// -------------------------------------------------------------------- boot

/// `--show`: abre o arquivo no registry (e' o que `GET /show` le) e sobe o player parado em
/// t=0, para monitor e transporte ja' terem o que mostrar.
fn abre(reg: Arc<Registry>, file: String) {
    if let Err(e) = reg.call("load", json!({ "path": file })) {
        return eprintln!("--show {}: {}", file, e);
    }
    let (r, f) = (reg.clone(), file.clone());
    std::thread::spawn(move || {
        if let Err(e) = r.call("play_show", json!({ "file": f })) {
            eprintln!("play_show {}: {}", f, e);
        }
    });
    std::thread::spawn(move || {
        // `play_show` da' play assim que o player sobe: pausar antes disso perde a corrida,
        // entao a pausa espera o estado "play" aparecer.
        for _ in 0..200 {
            match engine::player::current() {
                Some(h) if h.state().state == "play" => {
                    h.pause();
                    h.locate(0.0);
                    return;
                }
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        eprintln!("--show {}: player nao subiu em 2 s", file);
    });
}

/// Sobe o barramento e bloqueia ate o processo morrer. `reg` e' o registry completo da CLI:
/// e' ele que o WS, o `GET /commands` e as tools do MCP expoem.
///
/// `port` 0 = porta aleatoria (a linha `serve http://127.0.0.1:<porta>` no stderr diz qual);
/// `dir` = raiz do estatico; `show` = .spell aberto no boot, com o player parado em t=0.
pub fn serve(reg: Registry, port: u16, dir: PathBuf, show: Option<String>) -> Result<(), String> {
    let (tx, _rx) = broadcast::channel(256);
    let st = Arc::new(St {
        reg: Arc::new(reg),
        tx: tx.clone(),
        dir,
    });
    let _ = TX.set(tx.clone());
    engine::player::hook_global(move || {
        Box::new(Monitor {
            tx: tx.clone(),
            at: Instant::now(),
        })
    });

    let reg_mcp = st.reg.clone();
    let mcp_http = StreamableHttpService::new(
        move || Ok(mcp::Spell::new(reg_mcp.clone())),
        Arc::new(LocalSessionManager::default()),
        // sem sessao e com resposta JSON: o barramento nao guarda estado de cliente, e o
        // POST /mcp responde um JSON-RPC direto em vez de um fluxo SSE.
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    );

    let app = Router::new()
        .route("/commands", get(commands))
        .route("/show", get(show_get))
        .route("/ws", get(upgrade))
        .nest_service("/mcp", mcp_http)
        .fallback(estatico)
        .layer(axum::middleware::from_fn(no_store))
        .with_state(st.clone());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async move {
        // ponytail: so' loopback ; autenticacao esta fora de escopo, e sem ela abrir a LAN
        // entrega o hardware a quem estiver no wifi — `--host` entra junto com o token.
        let l = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|e| format!("porta {}: {}", port, e))?;
        let porta = l.local_addr().map_err(|e| e.to_string())?.port();
        eprintln!("serve http://127.0.0.1:{}", porta);
        if let Some(f) = show {
            abre(st.reg.clone(), f);
        }
        tokio::spawn(transporte(st.clone()));
        axum::serve(l, app).await.map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st() -> Arc<St> {
        let (tx, _rx) = broadcast::channel(8);
        Arc::new(St {
            reg: Arc::new(engine::registry::base()),
            tx,
            dir: PathBuf::from("."),
        })
    }

    /// A forma da resposta e o contador unico do engine: leitura nao mexe nele, edicao mexe.
    #[tokio::test]
    async fn request_responde_por_id_e_conta_rev() {
        let st = st();
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":7,"cmd":"show_new","args":{}}"#).await)
                .unwrap();
        assert_eq!(r["id"], json!(7));
        assert!(r["result"]["name"].is_string(), "{}", r);
        let apos_edicao = r["rev"].as_u64().expect("resposta carrega rev");
        assert_eq!(apos_edicao, engine::edit::rev(), "show_new e' edicao");

        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":8,"cmd":"show_get"}"#).await).unwrap();
        assert_eq!(r["id"], json!(8));
        assert_eq!(r["rev"], json!(apos_edicao), "show_get e' leitura");

        // erro de comando volta como {"id","error"}, nao como panico nem conexao fechada
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":9,"cmd":"pause"}"#).await).unwrap();
        assert_eq!(r["error"], json!("sem player em execucao"));
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":9,"cmd":"nao_existe"}"#).await).unwrap();
        assert_eq!(r["error"], json!("comando desconhecido: nao_existe"));
        let r: Value = serde_json::from_str(&request(&st, "isso nao e json").await).unwrap();
        assert!(r["error"].as_str().unwrap().starts_with("json invalido"));
        assert_eq!(engine::edit::rev(), apos_edicao, "erro nao conta rev");
    }

    /// `out.widget` do graph, vindo do sink da CLI, sai no WS com a forma do contrato.
    #[test]
    fn widget_vira_evento() {
        let (tx, mut rx) = broadcast::channel(4);
        TX.set(tx).expect("TX ainda livre neste binario de teste");
        widget("go", "hold", 1.0);
        let Ok(Out::Text(t)) = rx.try_recv() else {
            panic!("nada no barramento");
        };
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(
            v,
            json!({"event": "widget", "data": {"id": "go", "prop": "hold", "value": 1.0}})
        );
    }

    #[test]
    fn mime_por_extensao() {
        assert_eq!(mime("index.html"), "text/html; charset=utf-8");
        assert_eq!(mime("a/b/quatro.face.json"), "application/json");
        assert_eq!(mime("sem_extensao"), "application/octet-stream");
    }
}
