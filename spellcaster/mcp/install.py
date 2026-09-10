# `spell mcp_install`: writes the "spellcaster" entry into the MCP config of Claude Desktop or Claude Code.
# It shows the diff, makes a .bak backup and only writes after a "y" on the console.
import difflib
import json
import os
import shutil
import sys
from pathlib import Path

from ..core.registry import command

ROOT = str(Path(__file__).resolve().parents[2])   # folder that contains the spellcaster package
ENTRY = {"command": sys.executable,
         "args": ["-m", "spellcaster.cli", "mcp"],
         "env": {"PYTHONPATH": ROOT}}             # the package is not installed: PYTHONPATH finds it


def config_path(target):
    if target == "desktop":
        return os.path.join(os.environ.get("APPDATA") or os.path.expanduser("~"),
                            "Claude", "claude_desktop_config.json")
    if target == "code":
        return os.path.join(os.getcwd(), ".mcp.json")
    raise ValueError(f"target {target!r}: use desktop or code")


def _merge(old):
    new = json.loads(json.dumps(old))
    new.setdefault("mcpServers", {})["spellcaster"] = json.loads(json.dumps(ENTRY))
    return new


@command
def mcp_install(target: str = "desktop", path: str = "", yes: bool = False):
    """Registers the MCP server in claude_desktop_config.json (target desktop) or in .mcp.json (target code)."""
    p = path or config_path(target)
    old = {}
    if os.path.exists(p):
        with open(p, encoding="utf-8") as f:
            old = json.load(f) or {}
    new = _merge(old)
    a = json.dumps(old, indent=2, ensure_ascii=False).splitlines(True)
    b = json.dumps(new, indent=2, ensure_ascii=False).splitlines(True)
    diff = "".join(difflib.unified_diff(a, b, fromfile=p, tofile=p + " (new)"))
    print(f"{p}\n{diff or '(no change)'}")
    if not diff:
        return p
    if not yes:
        # ponytail: confirmation on the console only ; with no tty (pipe, or an MCP call) it aborts instead of hanging.
        if not sys.stdin.isatty():
            print("no console to confirm on: run `spell mcp_install` in the terminal or pass --yes")
            return None
        if input("write? [y/N] ").strip().lower() != "y":
            print("aborted")
            return None
    if os.path.exists(p):
        shutil.copyfile(p, p + ".bak")
        print(f"backup: {p}.bak")
    os.makedirs(os.path.dirname(os.path.abspath(p)), exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        json.dump(new, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"written: {p}")
    return p
