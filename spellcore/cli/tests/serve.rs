//! The bus as the GUI sees it: a real `spellcore serve --port 0`, HTTP and WebSocket by hand.
//! It brings the binary up (not axum in memory) because what breaks in practice is the
//! packaging: port, header, event order, MCP route.
//!
//! Its own WS client instead of `tokio-tungstenite`: the handshake and the framing fit in fifty
//! lines of `std::net` and are not worth a dev-dependency crate.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

mod common;

const RAIZ: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

struct Servidor {
    p: Child,
    porta: u16,
}

impl Servidor {
    fn start() -> Servidor {
        let mut p = common::bin()
            .args(["serve", "--port", "0", "--dir"])
            .arg(RAIZ)
            .arg("--show")
            .arg(format!("{}/shows/medgrupo.spell", RAIZ))
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spellcore serve");
        // the port comes out on stderr, together with the show warnings: read until the contract
        // line shows up
        let mut err = BufReader::new(p.stderr.take().expect("stderr"));
        let mut porta = 0;
        for _ in 0..40 {
            let mut l = String::new();
            if err.read_line(&mut l).unwrap_or(0) == 0 {
                break;
            }
            if let Some(u) = l.trim().strip_prefix("serve http://127.0.0.1:") {
                porta = u.parse().expect("port on the serve line");
                break;
            }
        }
        // the rest of stderr keeps being drained: a full pipe hangs the server
        std::thread::spawn(move || std::io::copy(&mut err, &mut std::io::sink()));
        assert!(porta > 0, "serve did not print the port on stderr");
        Servidor { p, porta }
    }

    fn tcp(&self) -> TcpStream {
        let s = TcpStream::connect(("127.0.0.1", self.porta)).expect("connect");
        s.set_read_timeout(Some(Duration::from_secs(10))).ok();
        s
    }

    /// A raw HTTP/1.0 request; returns (status line + headers, body).
    fn http(&self, req: &str) -> (String, String) {
        let mut s = self.tcp();
        s.write_all(req.as_bytes()).expect("write");
        let mut buf = String::new();
        s.read_to_string(&mut buf).expect("read the response");
        match buf.split_once("\r\n\r\n") {
            Some((h, b)) => (h.to_string(), b.to_string()),
            None => (buf, String::new()),
        }
    }

    /// Status code of the response to a GET.
    fn code(&self, path: &str) -> u16 {
        self.get(path)
            .0
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse().ok())
            .expect("status line")
    }

    fn get(&self, path: &str) -> (String, String) {
        self.http(&format!(
            "GET {} HTTP/1.0\r\nHost: 127.0.0.1:{}\r\n\r\n",
            path, self.porta
        ))
    }
}

impl Drop for Servidor {
    fn drop(&mut self) {
        self.p.kill().ok();
        self.p.wait().ok();
    }
}

// ------------------------------------------------------------ raw WS client

struct Ws(TcpStream);

impl Ws {
    fn open(sv: &Servidor) -> Ws {
        let mut s = sv.tcp();
        let req = format!(
            "GET /ws HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUpgrade: websocket\r\n\
             Connection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n",
            sv.porta
        );
        s.write_all(req.as_bytes()).expect("handshake");
        let mut h = Vec::new();
        let mut b = [0u8; 1];
        while !h.ends_with(b"\r\n\r\n") {
            assert_eq!(
                s.read(&mut b).expect("read the handshake"),
                1,
                "the server closed"
            );
            h.push(b[0]);
        }
        let h = String::from_utf8_lossy(&h).to_string();
        assert!(h.starts_with("HTTP/1.1 101"), "handshake refused: {}", h);
        Ws(s)
    }

    fn send(&mut self, id: u32, cmd: &str, args: Value) {
        let p = json!({"id": id, "cmd": cmd, "args": args}).to_string();
        let p = p.as_bytes();
        let mut f = vec![0x81u8]; // FIN + text
        if p.len() < 126 {
            f.push(0x80 | p.len() as u8); // masked, like every client frame
        } else {
            f.push(0x80 | 126);
            f.extend_from_slice(&(p.len() as u16).to_be_bytes());
        }
        f.extend_from_slice(&[0, 0, 0, 0]); // zero mask: XOR with 0 is the data itself
        f.extend_from_slice(p);
        self.0.write_all(&f).expect("send");
    }

    fn n(&mut self, n: usize) -> Vec<u8> {
        let mut b = vec![0u8; n];
        self.0.read_exact(&mut b).expect("read the frame");
        b
    }

    /// (opcode, payload). A server frame never comes masked.
    fn recv(&mut self) -> (u8, Vec<u8>) {
        let h = self.n(2);
        let n = match h[1] & 0x7f {
            126 => {
                let e = self.n(2);
                u16::from_be_bytes([e[0], e[1]]) as usize
            }
            127 => panic!("a 64-bit frame does not exist in this contract"),
            k => k as usize,
        };
        (h[0] & 0x0f, self.n(n))
    }

    fn json(&mut self) -> Value {
        loop {
            let (op, p) = self.recv();
            if op == 1 {
                return serde_json::from_slice(&p).expect("the ws text is JSON");
            }
        }
    }

    /// Reads until `cond` matches, or gives up after `secs`. Returns the message that matched.
    fn ate(&mut self, secs: f64, mut cond: impl FnMut(&Value) -> bool) -> Value {
        let fim = Instant::now() + Duration::from_secs_f64(secs);
        while Instant::now() < fim {
            let v = self.json();
            if cond(&v) {
                return v;
            }
        }
        panic!("nothing matched in {} s", secs);
    }
}

#[test]
fn bus_http_ws_monitor_and_mcp() {
    let sv = Servidor::start();

    // ---- HTTP: registry, show and static files
    let (h, b) = sv.get("/commands");
    assert!(h.contains(" 200 OK"), "{}", h);
    assert!(
        h.to_lowercase().contains("cache-control: no-store"),
        "{}",
        h
    );
    let cmds: Value = serde_json::from_str(&b).expect("/commands returns JSON");
    let nomes: Vec<&str> = cmds
        .as_array()
        .expect("command list")
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    for n in ["show_get", "play_show", "input", "key_set"] {
        assert!(nomes.contains(&n), "command {} missing: {:?}", n, nomes);
    }

    let (_, b) = sv.get("/show");
    let sh: Value = serde_json::from_str(&b).expect("/show returns JSON");
    assert_eq!(sh["fps"], json!(30), "--show left the .spell open");
    assert!(sh["tracks"].is_array(), "/show is the whole .spell: {}", b);

    // `--dir` is the repo root: the page and what it references (`../../design/tokens`,
    // `../../shows`) come from the same server
    assert_eq!(sv.code("/spellgui/web/index.html"), 200);
    assert_eq!(sv.code("/design/tokens/spellcaster.css"), 200);
    assert_eq!(sv.code("/shows/medgrupo.spell"), 200);
    // trust boundary: no climbing out of the directory
    assert_eq!(sv.code("/../../Cargo.toml"), 403);
    assert_eq!(sv.code("/no_such_file.js"), 404);

    // ---- WS: request, response and show event
    let mut ws = Ws::open(&sv);
    ws.send(1, "show_get", json!({}));
    let r = ws.ate(5.0, |v| v["id"] == json!(1));
    assert_eq!(r["result"]["fps"], json!(30), "{}", r);

    // every response carries the engine `rev`; an edit emits `show` with the SAME number
    let rev0 = r["rev"].as_u64().expect("the response carries rev");
    ws.send(2, "key_set", json!({"track": 0, "t": 3.0, "value": 42}));
    let r = ws.ate(5.0, |v| v["id"] == json!(2));
    assert!(r["error"].is_null(), "key_set: {}", r);
    let rev1 = r["rev"].as_u64().expect("the response carries rev");
    assert!(rev1 > rev0, "key_set edits: {} > {}", rev1, rev0);
    let ev = ws.ate(5.0, |v| v["event"] == json!("show"));
    assert_eq!(
        ev["data"]["rev"],
        json!(rev1),
        "show with the rev of the response"
    );

    // a read does not touch rev and does NOT emit show: the `locate` below only shows up in the
    // response
    ws.send(3, "locate", json!({"t": 1.0}));
    let r = ws.ate(5.0, |v| v["id"] == json!(3));
    assert_eq!(r["rev"], json!(rev1), "locate is a read: {}", r);

    // a `load` of another file swaps the whole show: it is not a read, it has to tell the GUI
    ws.send(
        10,
        "load",
        json!({"file": format!("{}/shows/medgrupo_r0.spell", RAIZ)}),
    );
    let r = ws.ate(10.0, |v| v["id"] == json!(10));
    assert!(r["error"].is_null(), "load: {}", r);
    let rev2 = r["rev"].as_u64().expect("the response carries rev");
    assert!(rev2 > rev1, "load bumps rev: {} > {}", rev2, rev1);
    // the only pending `show` is the one from the load: the `locate` in between emitted none
    let ev = ws.ate(5.0, |v| v["event"] == json!("show"));
    assert_eq!(ev["data"]["rev"], json!(rev2), "load bumps rev: {}", ev);

    // ---- transport and binary monitor. The `transport` comes out of a 10 Hz poll and the
    // `load` does not touch the player (it only opens the show for editing): the event of the
    // `locate` above may already have gone by (discarded by the `ate` calls above) or may not
    // have come out yet. A fresh `locate`, sent AFTER everything was read, produces an event
    // that can only arrive from here on.
    ws.send(4, "locate", json!({"t": 2.0}));
    let ev = ws.ate(5.0, |v| {
        v["event"] == json!("transport") && v["data"]["t"] == json!(2.0)
    });
    assert_eq!(
        ev["data"]["state"],
        json!("pause"),
        "--show leaves the player stopped: {}",
        ev
    );
    ws.send(5, "resume", json!({}));
    ws.ate(5.0, |v| v["id"] == json!(5));
    let ev = ws.ate(5.0, |v| {
        v["event"] == json!("transport") && v["data"]["state"] == json!("play")
    });
    assert!(ev["data"]["t"].is_number(), "{}", ev);

    let mut visto = false;
    let fim = Instant::now() + Duration::from_secs(5);
    while !visto && Instant::now() < fim {
        let (op, p) = ws.recv();
        if op == 2 {
            assert_eq!(p.len(), 515, "topic + universe + 512 channels");
            assert_eq!(p[0], 1, "topic 1 = output dmx");
            assert_eq!(
                u16::from_le_bytes([p[1], p[2]]),
                1,
                "universe 1 of the medgrupo show"
            );
            visto = true;
        }
    }
    assert!(visto, "no monitor frame in 5 s");

    // the global hook runs AFTER the programmer (position 6 of the frame): the monitor sees the
    // manual operator override, not only what the timeline wrote
    ws.send(
        20,
        "level_set",
        json!({"universe": 1, "address": 500, "values": [222]}),
    );
    let r = ws.ate(5.0, |v| v["id"] == json!(20));
    assert!(r["error"].is_null(), "level_set: {}", r);
    let mut visto = false;
    let fim = Instant::now() + Duration::from_secs(5);
    while !visto && Instant::now() < fim {
        let (op, p) = ws.recv();
        // channel 500 = byte 3 + (500 - 1)
        if op == 2 && p.len() == 515 && p[3 + 499] == 222 {
            visto = true;
        }
    }
    assert!(visto, "the monitor did not see the programmer value in 5 s");
    ws.send(21, "level_clear", json!({}));
    ws.ate(5.0, |v| v["id"] == json!(21));

    // `input` feeds the hooks of the live player
    ws.send(6, "input", json!({"key": "widget:go", "value": 1.0}));
    let r = ws.ate(5.0, |v| v["id"] == json!(6));
    assert!(r["error"].is_null(), "input: {}", r);

    // `play_show` blocks until the show ends: it comes back at once, as in the MCP
    ws.send(
        7,
        "play_show",
        json!({"file": format!("{}/shows/medgrupo.spell", RAIZ)}),
    );
    let r = ws.ate(5.0, |v| v["id"] == json!(7));
    assert_eq!(
        r["result"],
        json!("play_show started in the background"),
        "{}",
        r
    );

    ws.send(8, "stop", json!({}));
    ws.ate(5.0, |v| v["id"] == json!(8));

    // a command error comes back as {"id","error"} and the connection stays alive
    ws.send(9, "no_such_command", json!({}));
    let r = ws.ate(5.0, |v| v["id"] == json!(9));
    assert_eq!(r["error"], json!("unknown command: no_such_command"));

    // ---- MCP over HTTP on the same server
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                      "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                                 "clientInfo": {"name": "test", "version": "0"}}})
    .to_string();
    let (h, b) = sv.http(&format!(
        "POST /mcp HTTP/1.0\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
        sv.porta,
        body.len(),
        body
    ));
    assert!(h.contains(" 200 OK"), "{}\n{}", h, b);
    let v: Value = serde_json::from_str(&b).expect("POST /mcp returns JSON-RPC");
    assert_eq!(
        v["result"]["serverInfo"]["name"],
        json!("spellcaster"),
        "{}",
        b
    );
}
