//! Spellcaster MCP server over the official `rmcp` SDK. A port of
//! `spellcaster/mcp/server.py`: the tools COME FROM THE REGISTRY — no product logic here.
//!
//! Transport: stdio (`spellcore mcp`) and streamable HTTP at `/mcp`, mounted by the `serve`
//! crate (the rmcp `StreamableHttpService` is a `tower::Service`; the one with the axum is
//! `serve`).
//
//! Resources: `spell://show` (the open .spell), `spell://commands` (the whole registry as JSON),
//! `spell://graph` (the graph of section 10) and `spell://face` (the control surface).
//! The one that builds the `Registry` is the CLI: it is the one that knows `play_show` and `net`.

use engine::Registry;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData as McpError,
    Implementation, InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt};
use serde_json::{Map, Value};
use std::sync::Arc;

pub mod install;

const INSTRUCTIONS: &str = concat!(
    "Spellcaster: show control (sACN, Art-Net, OSC, laser ILDA). Ordem util: `net` para achar os ",
    "nos da rede, `show_get` para ler o show, `play_show` para tocar, `transport_state`/`pause`/",
    "`stop`/`locate`/`cue_go` para o transporte. Para montar um show: `show_new` ou `show_get` ",
    "com file, `profiles` + `patch_add`, `track_add` + `key_set`, `cue_set`, `show_save`. Os ",
    "resources spell://show, spell://commands, spell://graph e spell://face dao o show aberto, o ",
    "registry inteiro, o graph e a face sem gastar uma chamada de tool. Para editar o show, ",
    "prefira `show_patch` (JSON Patch, devolve as ops de undo) a `show_set`; o comportamento do ",
    "show (teclas, OSC, botoes) se le com `graph_get`, se edita com `show_patch` em /graph e se ",
    "valida com `graph_check`."
);

/// A command that blocks until the end of the show or until Ctrl+C: it runs in a thread and the
/// tool comes back at once (the `BACKGROUND` of `spellcaster/mcp/server.py`). `serve` asks the
/// same.
pub fn background(name: &str) -> bool {
    name == "play_show"
}

const SHOW: &str = "spell://show";
const COMMANDS: &str = "spell://commands";
const GRAPH: &str = "spell://graph";
const FACE: &str = "spell://face";

pub struct Spell {
    reg: Arc<Registry>,
}

impl Spell {
    /// It takes a `Registry` (stdio: one server per process) or an `Arc<Registry>` (`serve`: one
    /// `Spell` per HTTP session, all over the same registry).
    pub fn new(reg: impl Into<Arc<Registry>>) -> Spell {
        Spell { reg: reg.into() }
    }

    /// One tool per registry command: name, doc and the JSON schema `schemars` generated.
    fn tools(&self) -> Vec<Tool> {
        self.reg
            .iter()
            .map(|c| {
                let schema: Map<String, Value> = c.schema.as_object().cloned().unwrap_or_default();
                Tool::new(c.name.clone(), c.doc.clone(), Arc::new(schema))
            })
            .collect()
    }

    /// A resource that is just a registry command (no product logic lives here).
    fn leia(&self, cmd: &str) -> Result<String, McpError> {
        self.reg
            .call(cmd, Value::Object(Map::new()))
            .map(|v| texto(&v))
            .map_err(|e| McpError::internal_error(format!("{}: {}", cmd, e), None))
    }

    fn run(&self, name: &str, args: Value) -> CallToolResult {
        // An unknown command falls through to the `Registry::call` down below, which already
        // returns the message; the check here exists only so an invalid name is not sent to the
        // thread.
        if background(name) && self.reg.get(name).is_some() {
            let (reg, n) = (self.reg.clone(), name.to_string());
            std::thread::spawn(move || {
                if let Err(e) = reg.call(&n, args) {
                    eprintln!("{}: {}", n, e); // the stdout is the JSON-RPC channel
                }
            });
            return CallToolResult::success(vec![ContentBlock::text(format!(
                "{} started in the background; use transport_state, pause or stop to follow it",
                name
            ))]);
        }
        match self.reg.call(name, args) {
            Ok(v) => CallToolResult::success(vec![ContentBlock::text(texto(&v))]),
            Err(e) => CallToolResult::error(vec![ContentBlock::text(e)]),
        }
    }
}

/// A string from the command goes out raw (it is a finished report, like `net` without
/// `--json`); the rest goes out as indented JSON. The same rule as the CLI.
fn texto(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(v).unwrap_or_default(),
    }
}

impl ServerHandler for Spell {
    fn get_info(&self) -> ServerInfo {
        InitializeResult::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new(
            "spellcaster",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _r: Option<PaginatedRequestParams>,
        _c: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tools()))
    }

    async fn call_tool(
        &self,
        r: CallToolRequestParams,
        _c: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let args = Value::Object(r.arguments.unwrap_or_default());
        Ok(self.run(&r.name, args).into())
    }

    async fn list_resources(
        &self,
        _r: Option<PaginatedRequestParams>,
        _c: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(SHOW, "show")
                .with_title("Open show")
                .with_description(
                    "Summary of the open .spell: fps, duration, outputs, tracks, cues.",
                )
                .with_mime_type("application/json"),
            Resource::new(COMMANDS, "commands")
                .with_title("Registry")
                .with_description(
                    "Every product command: name, doc and JSON schema of the parameters.",
                )
                .with_mime_type("application/json"),
            Resource::new(GRAPH, "graph")
                .with_title("Show graph")
                .with_description("Behavior of the show: nodes and edges of section 10 of the PRD.")
                .with_mime_type("application/json"),
            Resource::new(FACE, "face")
                .with_title("Show face")
                .with_description("Control surface: faces/<name>.face.json or the inline object.")
                .with_mime_type("application/json"),
        ]))
    }

    async fn read_resource(
        &self,
        r: ReadResourceRequestParams,
        _c: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let txt = match r.uri.as_str() {
            SHOW => texto(
                &self
                    .reg
                    .call("show_get", Value::Object(Map::new()))
                    .map_err(|e| McpError::internal_error(format!("show_get: {}", e), None))?,
            ),
            COMMANDS => texto(&self.reg.schema()),
            GRAPH => self.leia("graph_get")?,
            FACE => self.leia("face_get")?,
            u => {
                return Err(McpError::resource_not_found(
                    format!("resource {}", u),
                    None,
                ))
            }
        };
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(txt, &r.uri).with_mime_type("application/json")
        ])
        .into())
    }
}

/// MCP server over stdio: one JSON-RPC message per line on stdin/stdout, log on stderr.
/// It blocks until the client closes the channel.
pub fn serve_stdio(reg: Registry) -> Result<(), String> {
    // ponytail: a current_thread runtime built here ; rmcp is async and the rest of spellcore is
    // not — swap it for multi_thread if some day a tool needs real concurrency inside the MCP.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let s = Spell::new(reg)
            .serve(stdio())
            .await
            .map_err(|e| e.to_string())?;
        eprintln!("MCP stdio ready");
        s.waiting().await.map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tools_come_from_the_registry() {
        let s = Spell::new(engine::registry::base());
        let t = s.tools();
        for n in [
            "load",
            "show_get",
            "pause",
            "stop",
            "locate",
            "transport_state",
        ] {
            assert!(t.iter().any(|x| x.name == n), "tool {} missing", n);
        }
        let locate = t.iter().find(|x| x.name == "locate").unwrap();
        assert_eq!(locate.input_schema["type"], json!("object"));
        assert!(locate.input_schema["properties"]["t"].is_object());
        assert!(locate
            .description
            .as_deref()
            .unwrap_or("")
            .contains("instant"));
    }

    #[test]
    fn call_tool_returns_the_registry_error_without_taking_the_server_down() {
        let s = Spell::new(engine::registry::base());
        let r = s.run("does_not_exist", json!({}));
        assert_eq!(r.is_error, Some(true));
        // with no live player, the transport answers with a tool error (the client reads the
        // text), not JSON-RPC
        let r = s.run("pause", json!({}));
        assert_eq!(r.is_error, Some(true));
        let r = s.run("show_get", json!({}));
        assert_eq!(r.is_error, Some(false));
    }

    #[test]
    fn raw_text_for_a_string() {
        assert_eq!(texto(&json!("report")), "report");
        assert_eq!(texto(&json!({"a": 1})), "{\n  \"a\": 1\n}");
    }
}
