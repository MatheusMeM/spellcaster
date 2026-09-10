# Spellcaster MCP (Model Context Protocol) server.
#
# Methods: initialize, notifications/initialized, ping, tools/list, tools/call,
#          resources/list, resources/read, prompts/list, prompts/get.
# Transport: stdio (one JSON line per message, stderr for the log).
# ponytail: stdio only ; if some client demands streamable HTTP, it goes in here.
#
# The tools come from the registry: no product logic here (same contract as the GUI).
# Entry points: `spell mcp` or `python -m spellcaster.mcp.server`.
import contextlib
import io
import json
import sys
import threading
import time
from collections import deque

from .. import __version__
from ..core import registry
from ..core.registry import command
from . import tools as spelltools

PROTOCOL = "2025-06-18"
SUPPORTED = ("2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05")
# ponytail: no real negotiation, just an echo of the requested version when it is known ; implement
# server/discover and the 2026-07-28 _meta when some client demands it.

# ponytail: the decorator already accepts mcp=False, but marking command by command would mean
# editing player/fixtures/gui/cli ; the list lives here until the next pass over those files.
HIGH_LEVEL = {"play_show", "stop", "pause", "locate", "net", "markers", "patch_list",
              "show_summary", "monitor"}

# Commands that block until Ctrl+C: they run in a thread and the tool returns right away.
BACKGROUND = {"play_show", "play", "serve", "mcp", "calib_hold", "calib_sweep"}

_JSON_TYPE = {"str": "string", "int": "integer", "float": "number", "bool": "boolean"}
LOG = deque(maxlen=200)            # spell://log resource

INSTRUCTIONS = ("Spellcaster: show control (sACN, Art-Net, OSC, ILDA laser). Useful order: `net` to find "
                "the nodes on the network, `show_summary` to read the open show, `play_show` to play it, "
                "`monitor` to check the 512 bytes of a universe. Commands outside the tool list "
                "live in `run_command`; the resources spell://show, spell://patch, spell://net and "
                "spell://log give the current state without spending a tool call.")


class MethodNotFound(Exception):
    pass


def log(msg):
    """Logs to stderr (stdout is the JSON-RPC channel) and to the spell://log resource buffer."""
    line = time.strftime("%H:%M:%S ") + str(msg)
    LOG.append(line)
    print(line, file=sys.stderr, flush=True)


# ---------------------------------------------------------------- tools
def tool_list():
    """MCP tools from the registry: one per HIGH_LEVEL command + the generic run_command."""
    out, rest = [], []
    for c in registry.schema():
        if c["name"] not in HIGH_LEVEL:
            rest.append(c["name"])
            continue
        props, req = {}, []
        for p in c["params"]:
            s = props[p["name"]] = {"type": _JSON_TYPE.get(p["type"], "string")}
            if p["default"] is None:
                req.append(p["name"])
            else:
                s["default"] = p["default"]
        out.append({"name": c["name"], "description": c["doc"] or c["name"],
                    "inputSchema": {"type": "object", "properties": props, "required": req}})
    out.append({
        "name": "run_command",
        "description": "Runs any low-level registry command. Available: "
                       + ", ".join(sorted(rest)) + ". Use `commands` to see the parameters of each one.",
        "inputSchema": {"type": "object",
                        "properties": {"name": {"type": "string", "description": "command name"},
                                       "args": {"type": "object", "description": "arguments by name"}},
                        "required": ["name"]}})
    return out


def call_tool(name, args):
    """Runs the tool and returns text. The command stdout is captured (on stdio it would corrupt the JSON-RPC)."""
    if name == "run_command":
        name, args = args.get("name", ""), dict(args.get("args") or {})
    if name not in registry.REGISTRY:
        raise KeyError(f"command {name!r} does not exist")
    log(f"tool {name} {args}")
    if name in BACKGROUND:
        threading.Thread(target=registry.call, args=(name,), kwargs=args, daemon=True).start()
        return f"{name} started in the background; use stop/pause/monitor to follow it"
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        r = registry.call(name, **args)
    # ponytail: when the command prints, the printed text is the answer and the return value is
    # discarded (today the two say the same thing) ; return both if some command ever diverges.
    return buf.getvalue().strip() or json.dumps(r, ensure_ascii=False, default=str)


# ---------------------------------------------------------------- resources
RESOURCES = [
    {"uri": "spell://show", "name": "show", "title": "Open show",
     "description": "Summary of the open .spell: fps, duration, outputs, tracks, cues, patch.",
     "mimeType": "application/json"},
    {"uri": "spell://patch", "name": "patch", "title": "Patch",
     "description": "Fixtures of the current patch: name, profile, universe, address, channels.",
     "mimeType": "application/json"},
    {"uri": "spell://net", "name": "net", "title": "Network",
     "description": "Last network scan: interfaces, Art-Net nodes, sACN sources, Ether Dream DACs.",
     "mimeType": "application/json"},
    {"uri": "spell://log", "name": "log", "title": "Log",
     "description": "Last 200 log lines of the MCP server.", "mimeType": "text/plain"},
]


def read_resource(uri):
    if uri == "spell://show":
        return json.dumps(spelltools.summary(), indent=1, ensure_ascii=False, default=str)
    if uri == "spell://patch":
        from ..fixtures import patch as fxpatch
        rows = fxpatch.PATCH.rows() if fxpatch.PATCH is not None else []
        return json.dumps(rows, indent=1, ensure_ascii=False)
    if uri == "spell://net":
        # ponytail: no cache, it runs a short scan on the spot ; keep the last one if it proves expensive.
        with contextlib.redirect_stdout(io.StringIO()):    # `net` prints the report; here the dict is what counts
            d = registry.call("net", timeout=1)
        return json.dumps(d, indent=1, ensure_ascii=False, default=str)
    if uri == "spell://log":
        return "\n".join(LOG)
    raise KeyError(f"resource {uri!r} does not exist")


def _mime(uri):
    return next((r["mimeType"] for r in RESOURCES if r["uri"] == uri), "text/plain")


# ---------------------------------------------------------------- prompts
PROMPTS = [
    {"name": "build_show_from_video", "title": "Build a show from a video",
     "description": "Step by step to turn the cuts of a video into the cues of a show.",
     "arguments": [{"name": "video", "description": "path of the video or .wav", "required": True}]},
    {"name": "calibrate_group", "title": "Calibrate a group of moving heads",
     "description": "Step by step to find the pan/tilt, colour wheel and gobo of a group.",
     "arguments": [{"name": "group", "description": "name of the group in the patch", "required": True}]},
]

_P_VIDEO = """Build a Spellcaster show from the video {video}. Follow this order:

1. `net` (timeout 2) to see interfaces, Art-Net nodes and sACN sources; confirm the output interface with me.
2. `patch_list` to see the fixtures already patched. If it is empty, ask which fixtures exist
   before going on; do not invent a profile or an address.
3. `markers` with video={video} to get the list of scene cuts (seconds).
4. Build the .spell: one cue per cut, short fade (0.2 s) on a hard cut and a long fade on a slow transition.
   Use `run_command` for the low-level commands that are missing, and `commands` to see their signature.
5. `show_summary` to check tracks, cues and duration before playing.
6. `play_show` and, right after it, `monitor` on the main universe to confirm DMX is going out.
7. `stop` when you are done. Never leave the show playing without telling me."""

_P_GROUP = """Calibrate the group {group} of the current patch. Follow this order:

1. `patch_list` and the spell://patch resource to see which units make up the group {group} and which profile they use.
2. `net` to confirm the output is on the right interface before sending light out.
3. Pan/tilt: `run_command` with name=calib_hold and args {{"group": "{group}", "pan": ..., "tilt": ...}}.
   Go by bisection, one axis at a time, and ask me what shows up on stage at every step.
4. Colour wheel and gobo: `run_command` with name=calib_sweep, channel=color (then gobo) and one value per
   unit, e.g. values="0,64,128,255". Write down which value gave which colour/gobo.
5. `monitor` on the group universe to confirm the bytes that are going out.
6. Close with a summary table: unit, pan, tilt, colour and gobo values. Do not write anything
   into the .spell without my confirmation."""


def get_prompt(name, args):
    if name == "build_show_from_video":
        txt = _P_VIDEO.format(video=args.get("video", "<video>"))
    elif name == "calibrate_group":
        txt = _P_GROUP.format(group=args.get("group", "<group>"))
    else:
        raise KeyError(f"prompt {name!r} does not exist")
    d = next(p for p in PROMPTS if p["name"] == name)
    return {"description": d["description"],
            "messages": [{"role": "user", "content": {"type": "text", "text": txt}}]}


# ---------------------------------------------------------------- JSON-RPC
def method(name, p):
    if name == "initialize":
        v = p.get("protocolVersion")
        return {"protocolVersion": v if v in SUPPORTED else PROTOCOL,
                "capabilities": {"tools": {}, "resources": {}, "prompts": {}, "logging": {}},
                "serverInfo": {"name": "spellcaster", "title": "Spellcaster", "version": __version__},
                "instructions": INSTRUCTIONS}
    if name == "ping":
        return {}
    if name == "tools/list":
        return {"tools": tool_list()}
    if name == "tools/call":
        try:
            txt = call_tool(p.get("name", ""), dict(p.get("arguments") or {}))
        except Exception as e:                       # a runtime error goes in the result, not in the JSON-RPC
            log(f"tools/call: {type(e).__name__}: {e}")
            return {"content": [{"type": "text", "text": f"{type(e).__name__}: {e}"}], "isError": True}
        return {"content": [{"type": "text", "text": txt}], "isError": False}
    if name == "resources/list":
        return {"resources": RESOURCES}
    if name == "resources/read":
        uri = p.get("uri", "")
        return {"contents": [{"uri": uri, "mimeType": _mime(uri), "text": read_resource(uri)}]}
    if name == "prompts/list":
        return {"prompts": PROMPTS}
    if name == "prompts/get":
        return get_prompt(p.get("name", ""), dict(p.get("arguments") or {}))
    raise MethodNotFound(name)


def handle(m):
    """One JSON-RPC message -> response (dict) or None (notification / client response)."""
    mid = m.get("id")
    name = m.get("method")
    if name is None:                                  # a client answer to a request of ours: ignored
        return None
    if name.startswith("notifications/"):
        log(f"<- {name}")
        return None
    try:
        r = method(name, dict(m.get("params") or {}))
    except MethodNotFound:
        return None if mid is None else _err(mid, -32601, f"unknown method: {name}")
    except KeyError as e:
        return None if mid is None else _err(mid, -32602, str(e))
    except Exception as e:
        log(f"{name}: {type(e).__name__}: {e}")
        return None if mid is None else _err(mid, -32603, f"{type(e).__name__}: {e}")
    return None if mid is None else {"jsonrpc": "2.0", "id": mid, "result": r}


def _err(mid, code, msg):
    return {"jsonrpc": "2.0", "id": mid, "error": {"code": code, "message": msg}}


def notification(level, data):
    return {"jsonrpc": "2.0", "method": "notifications/message",
            "params": {"level": level, "logger": "spellcaster", "data": data}}


# ---------------------------------------------------------------- player events
def watch(send, period=0.5, stop=None):
    """ponytail: the Player has no event hooks ; state polling at 2 Hz on this thread.
    Swap for a callback once Player exposes on_cue/on_end."""
    from ..player import player as pl
    last = None
    while stop is None or not stop.is_set():
        time.sleep(period)
        try:
            p = pl.CURRENT
            now = (p is not None, "" if p is None else p.clock.state, -1 if p is None else p.cues.index)
            if last is None or now == last:
                last = now
                continue
            if last[0] and not now[0]:
                send(notification("info", {"event": "show_end"}))
            elif now[2] != last[2] and now[2] >= 0:
                c = pl.CURRENT.cues.cues[now[2]]
                send(notification("info", {"event": "cue", "index": now[2], "name": c.name,
                                           "t": round(pl.CURRENT.clock.time, 3)}))
            elif now[1] != last[1]:
                send(notification("info", {"event": "transport", "state": now[1]}))
            last = now
        except Exception as e:                        # the watcher never takes the server down
            send(notification("error", {"event": "error", "text": f"{type(e).__name__}: {e}"}))
            last = None


# ---------------------------------------------------------------- stdio transport
def serve_stdio(inp=None, out=None):
    """One JSON line per message on stdin/stdout; log on stderr. Returns on EOF."""
    if inp is None:
        inp = sys.stdin
    if out is None:
        out = sys.stdout
        with contextlib.suppress(Exception):
            out.reconfigure(encoding="utf-8", newline="\n")
    lock = threading.Lock()

    def send(msg):
        with lock:                                    # ensure_ascii: a cp1252 console does not break the channel
            out.write(json.dumps(msg, default=str) + "\n")
            out.flush()

    threading.Thread(target=watch, args=(send,), daemon=True).start()
    log("MCP stdio ready")
    for line in inp:
        line = line.strip()
        if not line:
            continue
        try:
            m = json.loads(line)
        except ValueError as e:
            send(_err(None, -32700, f"parse error: {e}"))
            continue
        r = handle(m)
        if r is not None:
            send(r)
    log("MCP stdio: EOF")


@command
def mcp():
    """MCP server over stdio (Claude Desktop / Claude Code): one JSON line per message."""
    return serve_stdio()


if __name__ == "__main__":
    from .. import cli  # noqa: F401   (importing the CLI registers every command in the registry)
    serve_stdio()
