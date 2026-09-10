# Text monitor: transport, universes as bars, network sources, laser and log.
# With curses (Pi/Linux) it redraws the screen; without curses (Windows) it prints ONE line with \r.
import threading
import time

from .core.registry import command

GLYPHS = " .:-=+*#%@"                # 0..255 -> 10 levels
SOURCES = []                         # sources seen on the network; only --scan fills it


def bar(data, cols=48):
    """Bar of one universe: each column is the highest value of its channel block."""
    n = max(1, len(data) // cols)
    return "".join(GLYPHS[min(9, max(data[i:i + n], default=0) * 10 // 256)] for i in range(0, cols * n, n))


def snapshot(log=()):
    """State of this process: running player (transport, universes, laser) and network sources."""
    from .player.player import CURRENT
    st = {"state": "--", "t": 0.0, "universes": {}, "laser": "-", "sources": list(SOURCES), "log": list(log)}
    if CURRENT is not None:
        cfg = CURRENT.laser_cfg
        st["state"] = CURRENT.clock.state
        st["t"] = CURRENT.clock.time
        st["universes"] = {u.number: u.data for u in CURRENT.eng.universes.values()}
        st["laser"] = f"{cfg.get('pps', 25000)} pps dac {cfg.get('dac') or '(none)'}" if cfg else "-"
    return st


def render(st, cols=48):
    """State -> ASCII lines (the console is cp1252: nothing outside ASCII leaves here)."""
    ln = [f"spellcaster  transport {st['state']:5}  t={st['t']:8.2f}s"]
    for n, d in sorted(st["universes"].items()):
        ln.append(f"U{n:<3}|{bar(d, cols)}| max {max(d, default=0):3}")
    ln.append("laser: " + str(st["laser"]))
    ln.append("net  : " + (", ".join(st["sources"]) or "(no sources; spell tui --scan)"))
    ln += ["log  : " + s for s in st["log"][-5:]]
    return [x.encode("ascii", "replace").decode() for x in ln]


def draw(st, width=118):
    """Fallback without curses: everything on a single line, rewritten with \r."""
    print(("  ".join(render(st, 24)) + " " * width)[:width], end="\r", flush=True)


def _scan_bg(timeout=2):
    from .protocols import netscan

    def go():
        try:
            d = netscan.scan_all(timeout)
            SOURCES[:] = ([f"sacn {s['source_name']}" for s in d["sacn"]]
                          + [f"artnet {n['short_name']}" for n in d["artnet"]])
        except Exception as e:                     # the scan is diagnostics: failing there does not take the monitor down
            SOURCES[:] = [f"scan failed: {type(e).__name__}"]
    threading.Thread(target=go, daemon=True).start()


@command
def tui(fps: int = 4, seconds: float = 0, scan: bool = False):
    """Text monitor (curses, or a single line with \r without curses). --seconds caps it; Ctrl+C quits."""
    if scan:
        _scan_bg()
    log, prev = [], None
    end = time.perf_counter() + seconds if seconds else None

    def step():
        nonlocal prev
        st = snapshot(log)
        if st["state"] != prev:
            prev = st["state"]
            log.append(f"{time.strftime('%H:%M:%S')} transport {prev}")
        return st

    def loop(scr=None):
        while end is None or time.perf_counter() < end:
            st = step()
            if scr is None:
                draw(st)
            else:
                w = scr.getmaxyx()[1]
                scr.erase()
                for i, line in enumerate(render(st, max(8, w - 24))):
                    scr.addnstr(i, 0, line, w - 1)
                scr.refresh()
            time.sleep(1.0 / max(1, fps))

    try:
        import curses                              # ponytail: no keys, read-only ; read keys once the tui has transport
    except ImportError:
        curses = None
    try:
        curses.wrapper(loop) if curses else loop()
    except KeyboardInterrupt:
        pass
    print()
