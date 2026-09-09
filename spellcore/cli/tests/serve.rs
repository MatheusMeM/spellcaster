//! O barramento como a GUI o ve: `spellcore serve --port 0` de verdade, HTTP e WebSocket na mao.
//! Sobe o binario (nao o axum em memoria) porque o que quebra na pratica e' o empacotamento:
//! porta, cabecalho, ordem de evento, rota do MCP.
//!
//! Cliente WS proprio em vez de `tokio-tungstenite`: o handshake e o enquadramento cabem em
//! cinquenta linhas de `std::net` e nao valem um crate de dev-dependency.

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
        // a porta sai no stderr, junto com os avisos do show: le ate achar a linha do contrato
        let mut err = BufReader::new(p.stderr.take().expect("stderr"));
        let mut porta = 0;
        for _ in 0..40 {
            let mut l = String::new();
            if err.read_line(&mut l).unwrap_or(0) == 0 {
                break;
            }
            if let Some(u) = l.trim().strip_prefix("serve http://127.0.0.1:") {
                porta = u.parse().expect("porta na linha do serve");
                break;
            }
        }
        // o resto do stderr continua sendo drenado: pipe cheio trava o servidor
        std::thread::spawn(move || std::io::copy(&mut err, &mut std::io::sink()));
        assert!(porta > 0, "serve nao imprimiu a porta no stderr");
        Servidor { p, porta }
    }

    fn tcp(&self) -> TcpStream {
        let s = TcpStream::connect(("127.0.0.1", self.porta)).expect("conectar");
        s.set_read_timeout(Some(Duration::from_secs(10))).ok();
        s
    }

    /// Uma request HTTP/1.0 crua; devolve (linha de status + cabecalhos, corpo).
    fn http(&self, req: &str) -> (String, String) {
        let mut s = self.tcp();
        s.write_all(req.as_bytes()).expect("escrever");
        let mut buf = String::new();
        s.read_to_string(&mut buf).expect("ler resposta");
        match buf.split_once("\r\n\r\n") {
            Some((h, b)) => (h.to_string(), b.to_string()),
            None => (buf, String::new()),
        }
    }

    /// Codigo de status da resposta a um GET.
    fn code(&self, path: &str) -> u16 {
        self.get(path)
            .0
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse().ok())
            .expect("linha de status")
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

// ------------------------------------------------------------ cliente WS cru

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
            assert_eq!(s.read(&mut b).expect("ler handshake"), 1, "servidor fechou");
            h.push(b[0]);
        }
        let h = String::from_utf8_lossy(&h).to_string();
        assert!(h.starts_with("HTTP/1.1 101"), "handshake recusado: {}", h);
        Ws(s)
    }

    fn send(&mut self, id: u32, cmd: &str, args: Value) {
        let p = json!({"id": id, "cmd": cmd, "args": args}).to_string();
        let p = p.as_bytes();
        let mut f = vec![0x81u8]; // FIN + texto
        if p.len() < 126 {
            f.push(0x80 | p.len() as u8); // mascarado, como todo frame de cliente
        } else {
            f.push(0x80 | 126);
            f.extend_from_slice(&(p.len() as u16).to_be_bytes());
        }
        f.extend_from_slice(&[0, 0, 0, 0]); // mascara zero: XOR com 0 e' o proprio dado
        f.extend_from_slice(p);
        self.0.write_all(&f).expect("enviar");
    }

    fn n(&mut self, n: usize) -> Vec<u8> {
        let mut b = vec![0u8; n];
        self.0.read_exact(&mut b).expect("ler frame");
        b
    }

    /// (opcode, payload). Frame de servidor nunca vem mascarado.
    fn recv(&mut self) -> (u8, Vec<u8>) {
        let h = self.n(2);
        let n = match h[1] & 0x7f {
            126 => {
                let e = self.n(2);
                u16::from_be_bytes([e[0], e[1]]) as usize
            }
            127 => panic!("frame de 64 bits nao existe neste contrato"),
            k => k as usize,
        };
        (h[0] & 0x0f, self.n(n))
    }

    fn json(&mut self) -> Value {
        loop {
            let (op, p) = self.recv();
            if op == 1 {
                return serde_json::from_slice(&p).expect("texto do ws e' JSON");
            }
        }
    }

    /// Le ate `cond` casar, ou estoura em `secs`. Devolve a mensagem que casou.
    fn ate(&mut self, secs: f64, mut cond: impl FnMut(&Value) -> bool) -> Value {
        let fim = Instant::now() + Duration::from_secs_f64(secs);
        while Instant::now() < fim {
            let v = self.json();
            if cond(&v) {
                return v;
            }
        }
        panic!("nada casou em {} s", secs);
    }
}

#[test]
fn barramento_http_ws_monitor_e_mcp() {
    let sv = Servidor::start();

    // ---- HTTP: registry, show e estatico
    let (h, b) = sv.get("/commands");
    assert!(h.contains(" 200 OK"), "{}", h);
    assert!(
        h.to_lowercase().contains("cache-control: no-store"),
        "{}",
        h
    );
    let cmds: Value = serde_json::from_str(&b).expect("/commands devolve JSON");
    let nomes: Vec<&str> = cmds
        .as_array()
        .expect("lista de comandos")
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    for n in ["show_get", "play_show", "input", "key_set"] {
        assert!(nomes.contains(&n), "comando {} ausente: {:?}", n, nomes);
    }

    let (_, b) = sv.get("/show");
    let sh: Value = serde_json::from_str(&b).expect("/show devolve JSON");
    assert_eq!(sh["fps"], json!(30), "--show deixou o .spell aberto");
    assert!(sh["tracks"].is_array(), "/show e' o .spell inteiro: {}", b);

    // `--dir` e' a raiz do repo: a pagina e o que ela referencia (`../../design/tokens`,
    // `../../shows`) saem do mesmo servidor
    assert_eq!(sv.code("/spellgui/web/index.html"), 200);
    assert_eq!(sv.code("/design/tokens/spellcaster.css"), 200);
    assert_eq!(sv.code("/shows/medgrupo.spell"), 200);
    // limite de confianca: nada de subir de diretorio
    assert_eq!(sv.code("/../../Cargo.toml"), 403);
    assert_eq!(sv.code("/nao_existe.js"), 404);

    // ---- WS: request, resposta e evento de show
    let mut ws = Ws::open(&sv);
    ws.send(1, "show_get", json!({}));
    let r = ws.ate(5.0, |v| v["id"] == json!(1));
    assert_eq!(r["result"]["fps"], json!(30), "{}", r);

    // toda resposta carrega o `rev` do engine; edicao emite `show` com o MESMO numero
    let rev0 = r["rev"].as_u64().expect("resposta carrega rev");
    ws.send(2, "key_set", json!({"track": 0, "t": 3.0, "value": 42}));
    let r = ws.ate(5.0, |v| v["id"] == json!(2));
    assert!(r["error"].is_null(), "key_set: {}", r);
    let rev1 = r["rev"].as_u64().expect("resposta carrega rev");
    assert!(rev1 > rev0, "key_set edita: {} > {}", rev1, rev0);
    let ev = ws.ate(5.0, |v| v["event"] == json!("show"));
    assert_eq!(ev["data"]["rev"], json!(rev1), "show com o rev da resposta");

    // leitura nao mexe no rev e NAO emite show: o `locate` abaixo so' aparece na resposta
    ws.send(3, "locate", json!({"t": 1.0}));
    let r = ws.ate(5.0, |v| v["id"] == json!(3));
    assert_eq!(r["rev"], json!(rev1), "locate e' leitura: {}", r);

    // `load` de outro arquivo troca o show inteiro: nao e' leitura, tem que avisar a GUI
    ws.send(
        10,
        "load",
        json!({"path": format!("{}/shows/medgrupo_r0.spell", RAIZ)}),
    );
    let r = ws.ate(10.0, |v| v["id"] == json!(10));
    assert!(r["error"].is_null(), "load: {}", r);
    let rev2 = r["rev"].as_u64().expect("resposta carrega rev");
    assert!(rev2 > rev1, "load incrementa rev: {} > {}", rev2, rev1);
    // o unico `show` pendente e' o do load: o `locate` no meio nao emitiu nenhum
    let ev = ws.ate(5.0, |v| v["event"] == json!("show"));
    assert_eq!(ev["data"]["rev"], json!(rev2), "load incrementa rev: {}", ev);

    // ---- transporte e monitor binario: `--show` deixou o player parado em t=0
    let ev = ws.ate(5.0, |v| {
        v["event"] == json!("transport") && v["data"]["state"] == json!("pause")
    });
    assert_eq!(ev["data"]["t"], json!(0.0), "--show para em t=0: {}", ev);
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
            assert_eq!(p.len(), 515, "topic + universe + 512 canais");
            assert_eq!(p[0], 1, "topic 1 = dmx de saida");
            assert_eq!(
                u16::from_le_bytes([p[1], p[2]]),
                1,
                "universo 1 do medgrupo"
            );
            visto = true;
        }
    }
    assert!(visto, "nenhum frame de monitor em 5 s");

    // o gancho global roda DEPOIS do programmer (posicao 6 do frame): o monitor ve o override
    // manual do operador, nao so' o que a timeline escreveu
    ws.send(20, "level_set", json!({"universe": 1, "address": 500, "values": [222]}));
    let r = ws.ate(5.0, |v| v["id"] == json!(20));
    assert!(r["error"].is_null(), "level_set: {}", r);
    let mut visto = false;
    let fim = Instant::now() + Duration::from_secs(5);
    while !visto && Instant::now() < fim {
        let (op, p) = ws.recv();
        // canal 500 = byte 3 + (500 - 1)
        if op == 2 && p.len() == 515 && p[3 + 499] == 222 {
            visto = true;
        }
    }
    assert!(visto, "o monitor nao viu o valor do programmer em 5 s");
    ws.send(21, "level_clear", json!({}));
    ws.ate(5.0, |v| v["id"] == json!(21));

    // `input` alimenta os ganchos do player vivo
    ws.send(6, "input", json!({"key": "widget:go", "value": 1.0}));
    let r = ws.ate(5.0, |v| v["id"] == json!(6));
    assert!(r["error"].is_null(), "input: {}", r);

    // `play_show` bloqueia ate o fim do show: volta na hora, como no MCP
    ws.send(
        7,
        "play_show",
        json!({"file": format!("{}/shows/medgrupo.spell", RAIZ)}),
    );
    let r = ws.ate(5.0, |v| v["id"] == json!(7));
    assert_eq!(
        r["result"],
        json!("play_show iniciado em background"),
        "{}",
        r
    );

    ws.send(8, "stop", json!({}));
    ws.ate(5.0, |v| v["id"] == json!(8));

    // erro de comando volta como {"id","error"} e a conexao segue viva
    ws.send(9, "nao_existe", json!({}));
    let r = ws.ate(5.0, |v| v["id"] == json!(9));
    assert_eq!(r["error"], json!("comando desconhecido: nao_existe"));

    // ---- MCP por HTTP no mesmo servidor
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                      "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                                 "clientInfo": {"name": "teste", "version": "0"}}})
    .to_string();
    let (h, b) = sv.http(&format!(
        "POST /mcp HTTP/1.0\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
        sv.porta,
        body.len(),
        body
    ));
    assert!(h.contains(" 200 OK"), "{}\n{}", h, b);
    let v: Value = serde_json::from_str(&b).expect("POST /mcp devolve JSON-RPC");
    assert_eq!(
        v["result"]["serverInfo"]["name"],
        json!("spellcaster"),
        "{}",
        b
    );
}
