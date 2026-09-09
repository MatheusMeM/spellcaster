//! Servidor MCP do Spellcaster sobre o SDK oficial `rmcp`. Porte do
//! `spellcaster/mcp/server.py`: as tools SAEM DO REGISTRY — nada de logica de produto aqui.
//!
//! Transporte: stdio (`spellcore mcp`).
// ponytail: so' stdio ; o HTTP streamable do rmcp e' `StreamableHttpService`, um `tower::Service`
// que ainda exige axum/hyper para virar servidor (feature `server-side-http`, +11 crates) —
// entra quando alguem pedir MCP remoto no Pi, junto com o `serve` da GUI.
//
//! Resources: `spell://show` (o .spell aberto), `spell://commands` (o registry inteiro em
//! JSON), `spell://graph` (o graph da secao 10) e `spell://face` (a superficie de operacao).
//! Quem monta o `Registry` e' a CLI: e' ela que conhece `play_show` e `net`.

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
    "show (teclas, OSC, botoes) se le com `graph_get`, se troca com `graph_set` e se valida com ",
    "`graph_check`."
);

/// Comandos que bloqueiam ate o fim do show ou ate Ctrl+C: rodam em thread e a tool volta na hora
/// (o `BACKGROUND` do `spellcaster/mcp/server.py`).
const BACKGROUND: [&str; 1] = ["play_show"];

const SHOW: &str = "spell://show";
const COMMANDS: &str = "spell://commands";
const GRAPH: &str = "spell://graph";
const FACE: &str = "spell://face";

pub struct Spell {
    reg: Arc<Registry>,
}

impl Spell {
    pub fn new(reg: Registry) -> Spell {
        Spell { reg: Arc::new(reg) }
    }

    /// Uma tool por comando do registry: nome, doc e o schema JSON que o `schemars` gerou.
    fn tools(&self) -> Vec<Tool> {
        self.reg
            .iter()
            .map(|c| {
                let schema: Map<String, Value> = c.schema.as_object().cloned().unwrap_or_default();
                Tool::new(c.name.clone(), c.doc.clone(), Arc::new(schema))
            })
            .collect()
    }

    /// Resource que e' so' um comando do registry (nenhuma logica de produto mora aqui).
    fn leia(&self, cmd: &str) -> Result<String, McpError> {
        self.reg
            .call(cmd, Value::Object(Map::new()))
            .map(|v| texto(&v))
            .map_err(|e| McpError::internal_error(format!("{}: {}", cmd, e), None))
    }

    fn run(&self, name: &str, args: Value) -> CallToolResult {
        if self.reg.get(name).is_none() {
            return CallToolResult::error(vec![ContentBlock::text(format!(
                "comando desconhecido: {}",
                name
            ))]);
        }
        if BACKGROUND.contains(&name) {
            let (reg, n) = (self.reg.clone(), name.to_string());
            std::thread::spawn(move || {
                if let Err(e) = reg.call(&n, args) {
                    eprintln!("{}: {}", n, e); // stdout e' o canal JSON-RPC
                }
            });
            return CallToolResult::success(vec![ContentBlock::text(format!(
                "{} iniciado em background; use transport_state, pause ou stop para acompanhar",
                name
            ))]);
        }
        match self.reg.call(name, args) {
            Ok(v) => CallToolResult::success(vec![ContentBlock::text(texto(&v))]),
            Err(e) => CallToolResult::error(vec![ContentBlock::text(e)]),
        }
    }
}

/// String do comando sai crua (e' relatorio pronto, como o `net` sem `--json`); o resto sai como
/// JSON indentado. Mesma regra da CLI.
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
                .with_title("Show aberto")
                .with_description("Resumo do .spell aberto: fps, duracao, saidas, tracks, cues.")
                .with_mime_type("application/json"),
            Resource::new(COMMANDS, "commands")
                .with_title("Registry")
                .with_description("Todo comando do produto: nome, doc e schema JSON dos parametros.")
                .with_mime_type("application/json"),
            Resource::new(GRAPH, "graph")
                .with_title("Graph do show")
                .with_description("Comportamento do show: nodes e edges da secao 10 do PRD.")
                .with_mime_type("application/json"),
            Resource::new(FACE, "face")
                .with_title("Face do show")
                .with_description("Superficie de operacao: faces/<nome>.face.json ou o objeto inline.")
                .with_mime_type("application/json"),
        ]))
    }

    async fn read_resource(
        &self,
        r: ReadResourceRequestParams,
        _c: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let txt = match r.uri.as_str() {
            SHOW => texto(&self.reg.call("show_get", Value::Object(Map::new())).map_err(
                |e| McpError::internal_error(format!("show_get: {}", e), None),
            )?),
            COMMANDS => texto(&self.reg.schema()),
            GRAPH => self.leia("graph_get")?,
            FACE => self.leia("face_get")?,
            u => return Err(McpError::resource_not_found(format!("resource {}", u), None)),
        };
        Ok(ReadResourceResult::new(vec![ResourceContents::text(txt, &r.uri)
            .with_mime_type("application/json")])
        .into())
    }
}

/// Servidor MCP em stdio: uma mensagem JSON-RPC por linha em stdin/stdout, log em stderr.
/// Bloqueia ate o cliente fechar o canal.
pub fn serve_stdio(reg: Registry) -> Result<(), String> {
    // ponytail: runtime current_thread montado aqui ; o rmcp e' async e o resto do spellcore nao —
    // trocar por multi_thread se algum dia uma tool precisar de concorrencia real dentro do MCP.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let s = Spell::new(reg)
            .serve(stdio())
            .await
            .map_err(|e| e.to_string())?;
        eprintln!("MCP stdio pronto");
        s.waiting().await.map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tools_saem_do_registry() {
        let s = Spell::new(engine::registry::base());
        let t = s.tools();
        for n in ["load", "show_get", "pause", "stop", "locate", "transport_state"] {
            assert!(t.iter().any(|x| x.name == n), "tool {} ausente", n);
        }
        let locate = t.iter().find(|x| x.name == "locate").unwrap();
        assert_eq!(locate.input_schema["type"], json!("object"));
        assert!(locate.input_schema["properties"]["t"].is_object());
        assert!(locate.description.as_deref().unwrap_or("").contains("instante"));
    }

    #[test]
    fn call_tool_devolve_erro_do_registry_sem_derrubar_o_servidor() {
        let s = Spell::new(engine::registry::base());
        let r = s.run("nao_existe", json!({}));
        assert_eq!(r.is_error, Some(true));
        // sem player vivo, o transporte responde erro de tool (o cliente le o texto), nao JSON-RPC
        let r = s.run("pause", json!({}));
        assert_eq!(r.is_error, Some(true));
        let r = s.run("show_get", json!({}));
        assert_eq!(r.is_error, Some(false));
    }

    #[test]
    fn texto_cru_para_string() {
        assert_eq!(texto(&json!("relatorio")), "relatorio");
        assert_eq!(texto(&json!({"a": 1})), "{\n  \"a\": 1\n}");
    }
}
