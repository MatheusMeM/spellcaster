//! Spellcaster bus: one process touches the hardware, and every page and every AI talk to it.
//! HTTP for the registry, the show and the GUI files; WebSocket JSON-RPC for command and event;
//! binary frame for the DMX monitor; streamable MCP at `/mcp`.
//!
//! Word-for-word contract in `spellcore/README.md`, section "`serve` — bus". There is only
//! transport here: the logic belongs to the `Registry`, and this crate never calls the engine
//! outside it (the only exception is `player::current()`, read only to publish the transport).

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

/// Binary monitor frame: `topic:u8 | universe:u16 LE | 512 bytes`.
const TOPIC_DMX: u8 = 1;
/// Same format, but the INPUT universe (`show.inputs`), before any edit.
const TOPIC_IN: u8 = 2;
const FRAME: usize = 1 + 2 + 512;

/// Monitor ceiling: 40 Hz (a show at 60 fps does not send 60 frames per universe to the GUI).
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
    /// Last `rev` that already went out as a `show` event. It stops the doubled event: the WS
    /// command announces it right away, and the `revisao` poll only covers what changes OUTSIDE
    /// a command.
    visto: std::sync::atomic::AtomicU64,
}

impl St {
    fn evento(&self, event: &str, data: Value) {
        let _ = self.tx.send(Out::Text(texto(event, data)));
    }

    /// `{"event":"show","data":{"rev":n}}`, once per revision. The `swap` is what guarantees
    /// uniqueness: without it the poll and the command response announce the SAME revision when
    /// `revisao` wakes up between the edit and the response, and the client reloads twice.
    fn show_ev(&self, rev: u64) {
        if self.visto.swap(rev, std::sync::atomic::Ordering::Relaxed) != rev {
            self.evento("show", json!({ "rev": rev }));
        }
    }
}

fn texto(event: &str, data: Value) -> String {
    json!({"event": event, "data": data}).to_string()
}

/// The graph event sink lives in the CLI and has no `St`: `out.widget` reaches the WS through
/// here. With no `serve` running, it does nothing.
// ponytail: one `serve()` per process ; a second `serve()` in the same process keeps the sender
// of the first (OnceLock does not swap) and its widgets go out on the wrong bus. Pass the `St`
// down to the CLI sink when there are two live buses.
static TX: OnceLock<broadcast::Sender<Out>> = OnceLock::new();

/// `{"event":"widget","data":{"id","prop","value"}}` to every client of the bus.
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

// ponytail: closed list ; `--dir` holds the pages, the .spell, the font and the laser3d media —
// a new type comes in when some page brings one.
fn mime(p: &str) -> &'static str {
    match p.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "spell" => "application/json",
        "md" => "text/markdown; charset=utf-8", // design/SHORTCUTS.md, which help.html reads
        "woff2" => "font/woff2",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

/// File from `--dir`. Trust boundary: plain relative segments only. `..`, an empty segment, `\`
/// and `:` (drive, Windows ADS) are refused BEFORE touching the disk, and the path is not
/// percent-decoded — `%2e%2e` stays literal and becomes a 404, not a directory climb.
// ponytail: the default `--dir` is the repo root, so everything in it (`.git` included) is
// readable on 127.0.0.1 ; filter by list once `--host` and the token exist.
async fn estatico(State(st): State<Arc<St>>, uri: Uri) -> Response {
    let p = uri.path().trim_start_matches('/');
    let p = if p.is_empty() { "index.html" } else { p };
    let ruim = p.contains(':')
        || p.contains('\\')
        || p.split('/').any(|s| s.is_empty() || s == "." || s == "..");
    if ruim {
        return (StatusCode::FORBIDDEN, "path refused").into_response();
    }
    match tokio::fs::read(st.dir.join(p)).await {
        Ok(b) => ([(header::CONTENT_TYPE, mime(p))], b).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, format!("no {}", p)).into_response(),
    }
}

/// The GUI always reloads from the server: a stale `.js` in the cache is a night of debugging.
async fn no_store(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let mut r = next.run(req).await;
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

// ----------------------------------------------------------------- WebSocket

/// One WS request. `{"id":7,"cmd":"locate","args":{...}}` answers
/// `{"id":7,"result":...,"rev":n}` or `{"id":7,"error":"text","rev":n}`; EVERY response carries
/// the show revision, it is how the page knows whether the `show` that arrives is its own echo
/// or another client's. A command error does NOT drop the connection.
async fn request(st: &Arc<St>, txt: &str) -> String {
    let v: Value = match serde_json::from_str(txt) {
        Ok(v) => v,
        Err(e) => {
            return json!({"id": null, "error": format!("invalid json: {}", e),
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
        return json!({"id": id, "error": format!("unknown command: {}", cmd),
                      "rev": engine::edit::rev()})
        .to_string();
    }
    // `play_show` blocks until the show ends: it runs on a thread and the response comes back at
    // once (the same BACKGROUND list as the MCP; a single place decides what is a long-running
    // command).
    if mcp::background(&cmd) {
        let (reg, nome, s) = (st.reg.clone(), cmd.clone(), st.clone());
        std::thread::spawn(move || {
            if let Err(e) = reg.call(&nome, args) {
                s.evento("log", json!({ "text": format!("{}: {}", nome, e) }));
            }
        });
        let t = format!("{} started in the background", cmd);
        return json!({"id": id, "result": t, "rev": engine::edit::rev()}).to_string();
    }
    // a registry command is synchronous and may block (file, network scan)
    let (reg, nome) = (st.reg.clone(), cmd.clone());
    // There is no list of read commands: what tells whether the show changed is the engine's own
    // counter. A command that edits bumps `edit::rev()`; a command that only reads does not.
    // ponytail: before/after rev around a concurrent call: an edit of B during a read of A goes
    // out as an echo of A ; single command queue when two pages edit at the same time.
    let antes = engine::edit::rev();
    let r = tokio::task::spawn_blocking(move || reg.call(&nome, args)).await;
    let depois = engine::edit::rev();
    match r {
        Ok(Ok(v)) => {
            if depois != antes {
                st.show_ev(depois);
            }
            json!({"id": id, "result": v, "rev": depois}).to_string()
        }
        Ok(Err(e)) => json!({"id": id, "error": e, "rev": depois}).to_string(),
        Err(e) => {
            json!({"id": id, "error": format!("{} crashed: {}", cmd, e), "rev": depois}).to_string()
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
                Some(Ok(_)) => {} // ping/pong/binary from the client: the contract does not use it
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
                // ponytail: a slow client drops a monitor frame and moves on ; the data is the
                // next frame, not the history — nothing to resend.
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
}

/// `show {rev}` for an edit that did NOT come from a WS command — today only the recording
/// (`rec.rs`), which writes a keyframe inside the player frame. Without this the timeline would
/// never reload while recording.
// ponytail: 4 events per second, not one per keyframe ; a fader recording at 60 fps bumps `rev`
// 60 times per second and each event costs a whole GET /show on the page. Lower the period only
// if the lane being recorded looks late.
async fn revisao(st: Arc<St>) {
    loop {
        tokio::time::sleep(Duration::from_millis(250)).await;
        let r = engine::edit::rev();
        if r != st.visto.load(std::sync::atomic::Ordering::Relaxed) {
            st.show_ev(r);
        }
    }
}

/// Transport on the WS: on every change, and at 10 Hz while the player runs (while playing, `t`
/// changes every frame, so comparing the last JSON already gives both things).
// ponytail: 10 Hz polling ; turn it into a notice from the player itself if the GUI asks for
// less latency.
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

/// Copies the universes of the frame to the WS. Installed as a global hook, it runs after the
/// show hooks: it is the frame that actually goes out on the network.
struct Monitor {
    tx: broadcast::Sender<Out>,
    at: Instant,
    /// Player handle of this frame, found on the first frame (`start()` publishes the CURRENT
    /// after spawning the thread, so on frame 1 it may still be empty).
    h: Option<engine::player::Handle>,
}

fn bin(topic: u8, universe: u16, data: &[u8; 512]) -> Vec<u8> {
    let mut b = Vec::with_capacity(FRAME);
    b.push(topic);
    b.extend_from_slice(&universe.to_le_bytes());
    b.extend_from_slice(data);
    b
}

impl FrameHook for Monitor {
    // ponytail: sends every universe of the frame, with no cache of the last one sent ; the
    // 40 Hz ceiling on loopback already holds the bandwidth — caching comes in if a show with
    // dozens of idle universes shows up in the profile.
    fn frame(&mut self, _t: f64, uni: &mut Universes) {
        if self.tx.receiver_count() == 0 || self.at.elapsed() < Duration::from_millis(MONITOR_MS) {
            return;
        }
        self.at = Instant::now();
        for u in uni.iter() {
            let _ = self.tx.send(Out::Bin(bin(TOPIC_DMX, u.number, &u.data)));
        }
        if self.h.is_none() {
            self.h = engine::player::current();
        }
        if let Some(h) = self.h.as_ref() {
            for (n, d) in h.input_frames() {
                let _ = self.tx.send(Out::Bin(bin(TOPIC_IN, n, &d)));
            }
        }
    }
}

// -------------------------------------------------------------------- boot

/// `--show`: opens the file in the registry (it is what `GET /show` reads) and brings the player
/// up stopped at t=0, so that the monitor and the transport already have something to show.
fn abre(reg: Arc<Registry>, file: String) {
    if let Err(e) = reg.call("load", json!({ "file": file })) {
        return eprintln!("--show {}: {}", file, e);
    }
    let (r, f) = (reg.clone(), file.clone());
    std::thread::spawn(move || {
        if let Err(e) = r.call("play_show", json!({ "file": f })) {
            eprintln!("play_show {}: {}", f, e);
        }
    });
    std::thread::spawn(move || {
        // `play_show` plays as soon as the player comes up: pausing before that loses the race,
        // so the pause waits for the "play" state to appear.
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
        eprintln!("--show {}: player did not come up within 2 s", file);
    });
}

/// Brings the bus up and blocks until the process dies. `reg` is the full CLI registry: it is
/// what the WS, the `GET /commands` and the MCP tools expose.
///
/// `port` 0 = random port (the `serve http://127.0.0.1:<port>` line on stderr says which one);
/// `dir` = static root; `show` = .spell opened at boot, with the player stopped at t=0.
///
/// `ligou` receives once the address actually bound, before the first request: it is how the
/// window (`spellcaster.exe`) finds out the port when it asks for `port` 0 and runs this on a
/// thread. The CLI, which prints the stderr line and does not need the number, passes `|_| {}`.
pub fn serve(
    reg: Registry,
    port: u16,
    dir: PathBuf,
    show: Option<String>,
    ligou: impl FnOnce(std::net::SocketAddr),
) -> Result<(), String> {
    let (tx, _rx) = broadcast::channel(256);
    let st = Arc::new(St {
        reg: Arc::new(reg),
        tx: tx.clone(),
        dir,
        visto: std::sync::atomic::AtomicU64::new(engine::edit::rev()),
    });
    let _ = TX.set(tx.clone());
    engine::player::hook_global(move || {
        Box::new(Monitor {
            tx: tx.clone(),
            at: Instant::now(),
            h: None,
        })
    });

    let reg_mcp = st.reg.clone();
    let mcp_http = StreamableHttpService::new(
        move || Ok(mcp::Spell::new(reg_mcp.clone())),
        Arc::new(LocalSessionManager::default()),
        // no session and with a JSON response: the bus keeps no client state, and POST /mcp
        // answers a plain JSON-RPC instead of an SSE stream.
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
        // ponytail: loopback only ; authentication is out of scope, and without it opening the
        // LAN hands the hardware to whoever is on the wifi — `--host` comes in together with the
        // token.
        let l = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|e| format!("port {}: {}", port, e))?;
        let addr = l.local_addr().map_err(|e| e.to_string())?;
        eprintln!("serve http://127.0.0.1:{}", addr.port());
        ligou(addr);
        if let Some(f) = show {
            abre(st.reg.clone(), f);
        }
        tokio::spawn(transporte(st.clone()));
        tokio::spawn(revisao(st.clone()));
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
            visto: std::sync::atomic::AtomicU64::new(engine::edit::rev()),
        })
    }

    /// The same revision announced twice (poll + response) becomes a single event.
    #[test]
    fn show_ev_does_not_repeat_the_same_rev() {
        let st = st();
        let mut rx = st.tx.subscribe();
        let r = engine::edit::rev() + 1;
        st.show_ev(r);
        st.show_ev(r);
        assert!(rx.try_recv().is_ok(), "the first announcement goes out");
        assert!(
            rx.try_recv().is_err(),
            "the second announcement of the same rev is swallowed"
        );
    }

    /// The shape of the response and the single engine counter: a read does not touch it, an
    /// edit does.
    #[tokio::test]
    async fn request_answers_by_id_and_counts_rev() {
        let st = st();
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":7,"cmd":"show_new","args":{}}"#).await)
                .unwrap();
        assert_eq!(r["id"], json!(7));
        assert!(r["result"]["name"].is_string(), "{}", r);
        let apos_edicao = r["rev"].as_u64().expect("the response carries rev");
        assert_eq!(apos_edicao, engine::edit::rev(), "show_new is an edit");

        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":8,"cmd":"show_get"}"#).await).unwrap();
        assert_eq!(r["id"], json!(8));
        assert_eq!(r["rev"], json!(apos_edicao), "show_get is a read");

        // a command error comes back as {"id","error"}, not as a panic nor a closed connection
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":9,"cmd":"pause"}"#).await).unwrap();
        assert_eq!(r["error"], json!("no player running"));
        let r: Value =
            serde_json::from_str(&request(&st, r#"{"id":9,"cmd":"no_such_command"}"#).await)
                .unwrap();
        assert_eq!(r["error"], json!("unknown command: no_such_command"));
        let r: Value = serde_json::from_str(&request(&st, "this is not json").await).unwrap();
        assert!(r["error"].as_str().unwrap().starts_with("invalid json"));
        assert_eq!(
            engine::edit::rev(),
            apos_edicao,
            "an error does not count rev"
        );
    }

    /// `out.widget` from the graph, coming from the CLI sink, goes out on the WS in the shape of
    /// the contract.
    #[test]
    fn widget_becomes_an_event() {
        let (tx, mut rx) = broadcast::channel(4);
        TX.set(tx).expect("TX still free in this test binary");
        widget("go", "hold", 1.0);
        let Ok(Out::Text(t)) = rx.try_recv() else {
            panic!("nothing on the bus");
        };
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(
            v,
            json!({"event": "widget", "data": {"id": "go", "prop": "hold", "value": 1.0}})
        );
    }

    #[test]
    fn mime_by_extension() {
        assert_eq!(mime("index.html"), "text/html; charset=utf-8");
        assert_eq!(mime("a/b/quatro.face.json"), "application/json");
        assert_eq!(mime("design/SHORTCUTS.md"), "text/markdown; charset=utf-8");
        assert_eq!(mime("no_extension"), "application/octet-stream");
        // font and media of the 3D pages
        assert_eq!(mime("laser3d/michroma.woff2"), "font/woff2");
        assert_eq!(mime("a.svg"), "image/svg+xml");
        assert_eq!(mime("a.png"), "image/png");
        assert_eq!(mime("a.mp3"), "audio/mpeg");
        assert_eq!(mime("a.mp4"), "video/mp4");
    }
}
