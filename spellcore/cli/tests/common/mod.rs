//! Harness of the CLI tests: bring the real binary up and talk to it.
//!
//! The four integration tests (`mcp.rs`, `laser.rs`, `serve.rs`, `commands_json.rs`) bring
//! `spellcore` up as a process — what breaks in practice is the packaging, not the `Spell` in
//! memory. Each test binary compiles this module on its own; hence the `allow(dead_code)`:
//! whoever uses only `bin()` does not use the whole `Mcp`.

#![allow(dead_code)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// The freshly compiled `spellcore` binary, ready to take a subcommand and arguments.
pub fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spellcore"))
}

/// `spellcore mcp` over stdio: one JSON-RPC message per line, the id counted here.
pub struct Mcp {
    p: Child,
    inp: ChildStdin,
    out: BufReader<ChildStdout>,
    id: u32,
}

impl Mcp {
    /// Brings the server up WITHOUT a handshake: whoever wants to check the `initialize` answer
    /// calls `rpc`.
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

    /// Brings the server up and does the handshake (initialize + notifications/initialized).
    pub fn start() -> Mcp {
        let mut m = Mcp::cru();
        m.rpc(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "test", "version": "0"}}),
        );
        m.notifica("notifications/initialized");
        m
    }

    /// A notification (no id, no answer).
    pub fn notifica(&mut self, method: &str) {
        writeln!(self.inp, "{}", json!({"jsonrpc": "2.0", "method": method}))
            .expect("write to the server");
        self.inp.flush().expect("flush");
    }

    /// A request and the answer with the same id (skipping notifications). It fails on a JSON-RPC
    /// error.
    pub fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let id = self.id;
        writeln!(
            self.inp,
            "{}",
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        )
        .expect("write to the server");
        self.inp.flush().expect("flush");
        loop {
            let mut l = String::new();
            let n = self.out.read_line(&mut l).expect("read from the server");
            assert!(
                n > 0,
                "the server closed the stdout while waiting for {}",
                method
            );
            let v: Value = serde_json::from_str(&l)
                .unwrap_or_else(|e| panic!("the stdout is not JSON-RPC ({}): {:?}", e, l));
            if v["id"] == json!(id) {
                assert!(v["error"].is_null(), "{}: {}", method, v["error"]);
                return v["result"].clone();
            }
        }
    }

    /// A raw `tools/call`: the test decides what to do with `isError`.
    pub fn tool(&mut self, name: &str, args: Value) -> Value {
        self.rpc("tools/call", json!({"name": name, "arguments": args}))
    }

    /// Calls the command and returns the JSON it gave back. It fails if the command errored.
    pub fn cmd(&mut self, name: &str, args: Value) -> Value {
        let r = self.tool(name, args);
        let t = texto(&r);
        assert_eq!(r["isError"], json!(false), "{}: {}", name, t);
        serde_json::from_str(&t).unwrap_or(Value::String(t))
    }

    /// Calls the command expecting an error; it returns the error text.
    pub fn erro(&mut self, name: &str, args: Value) -> String {
        let r = self.tool(name, args);
        assert_eq!(r["isError"], json!(true), "{} should have failed", name);
        texto(&r)
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        self.p.kill().ok();
        self.p.wait().ok();
    }
}

/// Text of the first part of a `tools/call` (or of a `resources/read`).
pub fn texto(r: &Value) -> String {
    let v = if r["content"].is_array() {
        &r["content"]
    } else {
        &r["contents"]
    };
    v[0]["text"].as_str().unwrap_or_default().to_string()
}
