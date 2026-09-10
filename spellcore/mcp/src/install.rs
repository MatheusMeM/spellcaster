//! `spellcore mcp install`: writes the "spellcaster" entry into the MCP config of Claude Desktop
//! (`%APPDATA%\Claude\claude_desktop_config.json`) or of Claude Code (the project `.mcp.json`).
//! A port of `spellcaster/mcp/install.py`: it shows what it is going to write, makes a `.bak`
//! backup and only writes after a "y" on the console (or with `--yes`).
//!
// ponytail: it does not go into the registry (the Python `mcp_install` does) ; an AI session must
// not rewrite its own configuration — the one who installs is the operator, from the terminal.

use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Path of the config for the target. `desktop` = Claude Desktop; `code` = the project `.mcp.json`.
pub fn config_path(target: &str) -> Result<PathBuf, String> {
    match target {
        "desktop" => {
            let base = std::env::var("APPDATA")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|_| "no APPDATA and no HOME: pass --path".to_string())?;
            Ok(Path::new(&base)
                .join("Claude")
                .join("claude_desktop_config.json"))
        }
        "code" => Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(".mcp.json")),
        t => Err(format!("target {:?}: use desktop or code", t)),
    }
}

/// The entry written: the binary itself, subcommand `mcp`. With no `env`, unlike Python — the
/// `spellcore` is a single executable and does not need a PYTHONPATH.
fn entry() -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(json!({"command": exe.to_string_lossy(), "args": ["mcp"]}))
}

/// The old config + the "spellcaster" entry in `mcpServers`. It returns (new, was_already_equal).
fn merge(old: &Value, e: Value) -> (Value, bool) {
    let mut new = old.clone();
    if !new.is_object() {
        new = Value::Object(Map::new());
    }
    let srv = new
        .as_object_mut()
        .expect("object")
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    if !srv.is_object() {
        *srv = Value::Object(Map::new());
    }
    let igual = srv.get("spellcaster") == Some(&e);
    srv.as_object_mut()
        .expect("object")
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
    // ponytail: no unified diff (the stdlib has no difflib) ; it prints the path and the entry,
    // which is the only key this command touches. The `.bak` covers the rest.
    println!(
        "{}\n\"spellcaster\": {}",
        p.display(),
        serde_json::to_string_pretty(&new["mcpServers"]["spellcaster"]).unwrap_or_default()
    );
    if igual {
        println!("(no change)");
        return Ok(json!(p.to_string_lossy()));
    }
    if !yes {
        print!("write? [y/N] ");
        std::io::stdout().flush().ok();
        let mut l = String::new();
        // a closed stdin (a pipe, a non-interactive call) reads 0 bytes: it aborts instead of
        // hanging. "s" still answers yes: it is what the operator typed before this round.
        if std::io::stdin().lock().read_line(&mut l).unwrap_or(0) == 0
            || !matches!(
                l.trim().to_ascii_lowercase().as_str(),
                "y" | "yes" | "s" | "sim"
            )
        {
            println!("aborted");
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
    // ponytail: serde_json sorts the keys (without the preserve_order feature) ; the config comes
    // back in alphabetical order. Turn preserve_order on for the whole workspace only if someone
    // complains.
    let txt = serde_json::to_string_pretty(&new).map_err(|e| e.to_string())?;
    std::fs::write(&p, format!("{}\n", txt)).map_err(|e| format!("{}: {}", p.display(), e))?;
    println!("written: {}", p.display());
    Ok(json!(p.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_the_rest_and_detects_an_equal_entry() {
        let old = json!({"mcpServers": {"other": {"command": "x"}}, "thing": 1});
        let e = json!({"command": "spellcore.exe", "args": ["mcp"]});
        let (new, igual) = merge(&old, e.clone());
        assert!(!igual);
        assert_eq!(new["thing"], json!(1));
        assert_eq!(new["mcpServers"]["other"]["command"], json!("x"));
        assert_eq!(new["mcpServers"]["spellcaster"], e);
        // a second pass: nothing changes
        let (new2, igual2) = merge(&new, e);
        assert!(igual2);
        assert_eq!(new2, new);
    }

    #[test]
    fn merge_on_an_empty_or_crooked_file() {
        let e = json!({"command": "c", "args": ["mcp"]});
        for old in [json!({}), json!(null), json!({"mcpServers": 7})] {
            let (new, igual) = merge(&old, e.clone());
            assert!(!igual);
            assert_eq!(new["mcpServers"]["spellcaster"], e);
        }
    }

    #[test]
    fn an_unknown_target_is_an_error() {
        assert!(config_path("nothing").is_err());
        assert!(config_path("code").unwrap().ends_with(".mcp.json"));
    }

    /// It really writes into a temporary file with `--yes`: it is the path the operator uses.
    #[test]
    fn it_writes_with_yes_and_is_idempotent() {
        let p = std::env::temp_dir().join("spellcore_mcp_install_test.json");
        std::fs::write(&p, "{\"thing\": 1}\n").unwrap();
        let s = p.to_string_lossy().to_string();
        assert_eq!(install("desktop", &s, true).unwrap(), json!(s));
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(v["thing"], json!(1));
        assert_eq!(v["mcpServers"]["spellcaster"]["args"], json!(["mcp"]));
        // again: it changes nothing and asks for no confirmation
        assert_eq!(install("desktop", &s, false).unwrap(), json!(s));
        std::fs::remove_file(&p).ok();
        std::fs::remove_file(p.with_extension("json.bak")).ok();
    }
}
