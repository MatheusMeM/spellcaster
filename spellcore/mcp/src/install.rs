//! `spellcore mcp install`: grava a entrada "spellcaster" no config MCP do Claude Desktop
//! (`%APPDATA%\Claude\claude_desktop_config.json`) ou do Claude Code (`.mcp.json` do projeto).
//! Porte do `spellcaster/mcp/install.py`: mostra o que vai gravar, faz backup `.bak` e so' escreve
//! depois de um "s" no console (ou com `--yes`).
//!
// ponytail: nao entra no registry (o `mcp_install` do Python entra) ; uma sessao de IA nao deve
// reescrever a propria configuracao — quem instala e' o operador, pelo terminal.

use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Caminho do config para o alvo. `desktop` = Claude Desktop; `code` = `.mcp.json` do projeto.
pub fn config_path(target: &str) -> Result<PathBuf, String> {
    match target {
        "desktop" => {
            let base = std::env::var("APPDATA")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|_| "sem APPDATA nem HOME: passe --path".to_string())?;
            Ok(Path::new(&base)
                .join("Claude")
                .join("claude_desktop_config.json"))
        }
        "code" => Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(".mcp.json")),
        t => Err(format!("target {:?}: use desktop ou code", t)),
    }
}

/// A entrada gravada: o proprio binario, subcomando `mcp`. Sem `env`, ao contrario do Python — o
/// `spellcore` e' um executavel unico e nao precisa de PYTHONPATH.
fn entry() -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(json!({"command": exe.to_string_lossy(), "args": ["mcp"]}))
}

/// Config antigo + a entrada "spellcaster" em `mcpServers`. Devolve (novo, ja_existia_igual).
fn merge(old: &Value, e: Value) -> (Value, bool) {
    let mut new = old.clone();
    if !new.is_object() {
        new = Value::Object(Map::new());
    }
    let srv = new
        .as_object_mut()
        .expect("objeto")
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    if !srv.is_object() {
        *srv = Value::Object(Map::new());
    }
    let igual = srv.get("spellcaster") == Some(&e);
    srv.as_object_mut()
        .expect("objeto")
        .insert("spellcaster".into(), e);
    (new, igual)
}

pub fn install(target: &str, path: &str, yes: bool) -> Result<Value, String> {
    let p = if path.is_empty() {
        config_path(target)?
    } else {
        PathBuf::from(path)
    };
    let old: Value = match std::fs::read_to_string(&p) {
        Ok(t) if !t.trim().is_empty() => {
            serde_json::from_str(&t).map_err(|e| format!("{}: {}", p.display(), e))?
        }
        _ => Value::Object(Map::new()),
    };
    let (new, igual) = merge(&old, entry()?);
    // ponytail: sem diff unificado (a stdlib nao tem difflib) ; imprime o caminho e a entrada,
    // que e' a unica chave que este comando toca. O `.bak` cobre o resto.
    println!(
        "{}\n\"spellcaster\": {}",
        p.display(),
        serde_json::to_string_pretty(&new["mcpServers"]["spellcaster"]).unwrap_or_default()
    );
    if igual {
        println!("(sem mudanca)");
        return Ok(json!(p.to_string_lossy()));
    }
    if !yes {
        print!("gravar? [s/N] ");
        std::io::stdout().flush().ok();
        let mut l = String::new();
        // stdin fechado (pipe, chamada nao interativa) le 0 bytes: aborta em vez de travar.
        if std::io::stdin().lock().read_line(&mut l).unwrap_or(0) == 0
            || !l.trim().eq_ignore_ascii_case("s")
        {
            println!("abortado");
            return Ok(Value::Null);
        }
    }
    if p.exists() {
        let bak = p.with_extension("json.bak");
        std::fs::copy(&p, &bak).map_err(|e| format!("{}: {}", bak.display(), e))?;
        println!("backup: {}", bak.display());
    }
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {}", d.display(), e))?;
    }
    // ponytail: o serde_json ordena as chaves (sem a feature preserve_order) ; o config volta em
    // ordem alfabetica. Ligar preserve_order no workspace inteiro so' se alguem reclamar.
    let txt = serde_json::to_string_pretty(&new).map_err(|e| e.to_string())?;
    std::fs::write(&p, format!("{}\n", txt)).map_err(|e| format!("{}: {}", p.display(), e))?;
    println!("gravado: {}", p.display());
    Ok(json!(p.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_preserva_o_resto_e_detecta_igual() {
        let old = json!({"mcpServers": {"outro": {"command": "x"}}, "algo": 1});
        let e = json!({"command": "spellcore.exe", "args": ["mcp"]});
        let (new, igual) = merge(&old, e.clone());
        assert!(!igual);
        assert_eq!(new["algo"], json!(1));
        assert_eq!(new["mcpServers"]["outro"]["command"], json!("x"));
        assert_eq!(new["mcpServers"]["spellcaster"], e);
        // segunda passada: nada muda
        let (new2, igual2) = merge(&new, e);
        assert!(igual2);
        assert_eq!(new2, new);
    }

    #[test]
    fn merge_em_arquivo_vazio_ou_torto() {
        let e = json!({"command": "c", "args": ["mcp"]});
        for old in [json!({}), json!(null), json!({"mcpServers": 7})] {
            let (new, igual) = merge(&old, e.clone());
            assert!(!igual);
            assert_eq!(new["mcpServers"]["spellcaster"], e);
        }
    }

    #[test]
    fn target_desconhecido_e_erro() {
        assert!(config_path("nada").is_err());
        assert!(config_path("code").unwrap().ends_with(".mcp.json"));
    }

    /// Grava de verdade num arquivo temporario com `--yes`: e' o caminho que o operador usa.
    #[test]
    fn grava_com_yes_e_e_idempotente() {
        let p = std::env::temp_dir().join("spellcore_mcp_install_test.json");
        std::fs::write(&p, "{\"algo\": 1}\n").unwrap();
        let s = p.to_string_lossy().to_string();
        assert_eq!(install("desktop", &s, true).unwrap(), json!(s));
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(v["algo"], json!(1));
        assert_eq!(v["mcpServers"]["spellcaster"]["args"], json!(["mcp"]));
        // de novo: nao muda nada e nao pede confirmacao
        assert_eq!(install("desktop", &s, false).unwrap(), json!(s));
        std::fs::remove_file(&p).ok();
        std::fs::remove_file(p.with_extension("json.bak")).ok();
    }
}
