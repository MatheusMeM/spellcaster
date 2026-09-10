# Commands that only exist because of MCP (show summary, output monitor).
# They enter the registry like any other: the CLI and the GUI see them too.
import base64
import json

from ..core.registry import command

OPEN = {}        # ponytail: one open show per process, same as the PATCH in fixtures/patch.py
                 # ; swap for a session id when the GUI opens two shows at the same time.


def current():
    """Current show: the running player's; otherwise the last one opened by show_summary(file)."""
    from ..player import player as pl
    if pl.CURRENT is not None:
        return pl.CURRENT.show
    return OPEN or None


def summary(file=""):
    """Show summary without printing anything (used by the spell://show resource)."""
    from .. import show as showfile
    if file:
        OPEN.clear()
        OPEN.update(showfile.load(file))
    sh = current()
    if sh is None:
        return {"open": False, "hint": "call show_summary(file=...) or play_show(file=...)"}
    from ..player import player as pl
    p = pl.CURRENT
    return {"open": True,
            "name": sh.get("name", ""),
            "file": OPEN.get("_dir") or sh.get("_dir", ""),
            "fps": sh.get("fps", 30),
            "duration": sh.get("duration"),
            "playing": None if p is None else p.clock.state,
            "t": None if p is None else round(p.clock.time, 3),
            "outputs": [c.get("type") for c in sh.get("outputs", ())],
            "patch": [f.get("name") for f in sh.get("patch", ())],
            "tracks": [{"type": t.get("type"), "name": t.get("name") or t.get("fixture") or t.get("file", ""),
                        "universe": t.get("universe", 1)} for t in sh.get("tracks", ())],
            "cues": [{"name": c.get("name", ""), "fade": c.get("fade", 0.0), "wait": c.get("wait", 0.0),
                      "follow": bool(c.get("follow"))} for c in sh.get("cues", ())]}


@command
def show_summary(file: str = ""):
    """Summary of the open .spell (or of the given file): name, fps, duration, outputs, tracks, cues, patch."""
    d = summary(file)
    print(json.dumps(d, indent=1, ensure_ascii=False))
    return d


@command
def monitor(universe: int = 1):
    """512 bytes of the universe in the running player, in base64 (JSON with universe, t and b64)."""
    from ..player import player as pl
    p = pl.CURRENT
    # ponytail: reads the sACN engine only ; read the Art-Net engine too when someone asks for an Art-Net monitor.
    u = None if p is None else p.eng.universes.get(int(universe))
    d = {"universe": int(universe),
         "t": None if p is None else round(p.clock.time, 3),
         "playing": None if p is None else p.clock.state,
         "b64": base64.b64encode(bytes(u.data) if u is not None else bytes(512)).decode()}
    print(json.dumps(d))
    return d
