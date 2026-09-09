//! Harness dos testes da CLI: subir o binario de verdade e falar com ele.
//!
//! Os quatro testes de integracao (`mcp.rs`, `laser.rs`, `serve.rs`, `commands_json.rs`) sobem
//! `spellcore` como processo — e' o empacotamento que quebra na pratica, nao o `Spell` em memoria.
//! Cada binario de teste compila este modulo por conta propria; por isso o `allow(dead_code)`:
//! quem usa so' `bin()` nao usa o `Mcp` inteiro.

#![allow(dead_code)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// O binario `spellcore` recem compilado, pronto para receber subcomando e argumentos.
pub fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spellcore"))
}

/// `spellcore mcp` em stdio: uma mensagem JSON-RPC por linha, o id contado aqui.
pub struct Mcp {
    p: Child,
    inp: ChildStdin,
    out: BufReader<ChildStdout>,
    id: u32,
}

impl Mcp {
    /// Sobe o servidor SEM handshake: quem quer conferir a resposta do `initialize` chama `rpc`.
    pub fn cru() -> Mcp {
        let mut p = bin()
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spellcore mcp");
        let inp = p.stdin.take().expect("stdin");
        let out = BufReader::new(p.stdout.take().expect("stdout"));
        Mcp { p, inp, out, id: 0 }
    }

    /// Sobe o servidor e faz o handshake (initialize + notifications/initialized).
    pub fn start() -> Mcp {
        let mut m = Mcp::cru();
        m.rpc(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "teste", "version": "0"}}),
        );
        m.notifica("notifications/initialized");
        m
    }

    /// Notificacao (sem id, sem resposta).
    pub fn notifica(&mut self, method: &str) {
        writeln!(self.inp, "{}", json!({"jsonrpc": "2.0", "method": method}))
            .expect("escrever no servidor");
        self.inp.flush().expect("flush");
    }

    /// Request e a resposta com o mesmo id (pulando notificacoes). Falha em erro JSON-RPC.
    pub fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let id = self.id;
        writeln!(
            self.inp,
            "{}",
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        )
        .expect("escrever no servidor");
        self.inp.flush().expect("flush");
        loop {
            let mut l = String::new();
            let n = self.out.read_line(&mut l).expect("ler do servidor");
            assert!(n > 0, "servidor fechou o stdout esperando {}", method);
            let v: Value = serde_json::from_str(&l)
                .unwrap_or_else(|e| panic!("stdout nao e' JSON-RPC ({}): {:?}", e, l));
            if v["id"] == json!(id) {
                assert!(v["error"].is_null(), "{}: {}", method, v["error"]);
                return v["result"].clone();
            }
        }
    }

    /// `tools/call` cru: o teste decide o que fazer com `isError`.
    pub fn tool(&mut self, name: &str, args: Value) -> Value {
        self.rpc("tools/call", json!({"name": name, "arguments": args}))
    }

    /// Chama o comando e devolve o JSON que ele retornou. Falha se o comando errou.
    pub fn cmd(&mut self, name: &str, args: Value) -> Value {
        let r = self.tool(name, args);
        let t = texto(&r);
        assert_eq!(r["isError"], json!(false), "{}: {}", name, t);
        serde_json::from_str(&t).unwrap_or(Value::String(t))
    }

    /// Chama o comando esperando erro; devolve o texto do erro.
    pub fn erro(&mut self, name: &str, args: Value) -> String {
        let r = self.tool(name, args);
        assert_eq!(r["isError"], json!(true), "{} devia ter falhado", name);
        texto(&r)
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        self.p.kill().ok();
        self.p.wait().ok();
    }
}

/// Texto da primeira parte de um `tools/call` (ou de um `resources/read`).
pub fn texto(r: &Value) -> String {
    let v = if r["content"].is_array() {
        &r["content"]
    } else {
        &r["contents"]
    };
    v[0]["text"].as_str().unwrap_or_default().to_string()
}
