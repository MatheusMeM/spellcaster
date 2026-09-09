# `spell mcp_install`: grava a entrada "spellcaster" no config MCP do Claude Desktop ou do Claude Code.
# Mostra o diff, faz backup .bak e so escreve depois de um "s" no console.
import difflib
import json
import os
import shutil
import sys
from pathlib import Path

from ..core.registry import command

ROOT = str(Path(__file__).resolve().parents[2])   # pasta que contem o pacote spellcaster
ENTRY = {"command": sys.executable,
         "args": ["-m", "spellcaster.cli", "mcp"],
         "env": {"PYTHONPATH": ROOT}}             # o pacote nao esta instalado: PYTHONPATH acha ele


def config_path(target):
    if target == "desktop":
        return os.path.join(os.environ.get("APPDATA") or os.path.expanduser("~"),
                            "Claude", "claude_desktop_config.json")
    if target == "code":
        return os.path.join(os.getcwd(), ".mcp.json")
    raise ValueError(f"target {target!r}: use desktop ou code")


def _merge(old):
    new = json.loads(json.dumps(old))
    new.setdefault("mcpServers", {})["spellcaster"] = json.loads(json.dumps(ENTRY))
    return new


@command
def mcp_install(target: str = "desktop", path: str = "", yes: bool = False):
    """Registra o servidor MCP no claude_desktop_config.json (target desktop) ou no .mcp.json (target code)."""
    p = path or config_path(target)
    old = {}
    if os.path.exists(p):
        with open(p, encoding="utf-8") as f:
            old = json.load(f) or {}
    new = _merge(old)
    a = json.dumps(old, indent=2, ensure_ascii=False).splitlines(True)
    b = json.dumps(new, indent=2, ensure_ascii=False).splitlines(True)
    diff = "".join(difflib.unified_diff(a, b, fromfile=p, tofile=p + " (novo)"))
    print(f"{p}\n{diff or '(sem mudanca)'}")
    if not diff:
        return p
    if not yes:
        # ponytail: confirmacao so no console ; sem tty (pipe, ou chamada por MCP) aborta em vez de travar.
        if not sys.stdin.isatty():
            print("sem console para confirmar: rode `spell mcp_install` no terminal ou passe --yes")
            return None
        if input("gravar? [s/N] ").strip().lower() != "s":
            print("abortado")
            return None
    if os.path.exists(p):
        shutil.copyfile(p, p + ".bak")
        print(f"backup: {p}.bak")
    os.makedirs(os.path.dirname(os.path.abspath(p)), exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        json.dump(new, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"gravado: {p}")
    return p
