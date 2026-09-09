# Monitor de texto: transporte, universos em barras, fontes de rede, laser e log.
# Com curses (Pi/Linux) redesenha a tela; sem curses (Windows) imprime UMA linha com \r.
import threading
import time

from .core.registry import command

GLYPHS = " .:-=+*#%@"                # 0..255 -> 10 niveis
SOURCES = []                         # fontes vistas na rede; so o --scan preenche


def bar(data, cols=48):
    """Barra de um universo: cada coluna e o maior valor do seu bloco de canais."""
    n = max(1, len(data) // cols)
    return "".join(GLYPHS[min(9, max(data[i:i + n], default=0) * 10 // 256)] for i in range(0, cols * n, n))


def snapshot(log=()):
    """Estado deste processo: player em execucao (transporte, universos, laser) e fontes de rede."""
    from .player.player import CURRENT
    st = {"state": "--", "t": 0.0, "universes": {}, "laser": "-", "sources": list(SOURCES), "log": list(log)}
    if CURRENT is not None:
        cfg = CURRENT.laser_cfg
        st["state"] = CURRENT.clock.state
        st["t"] = CURRENT.clock.time
        st["universes"] = {u.number: u.data for u in CURRENT.eng.universes.values()}
        st["laser"] = f"{cfg.get('pps', 25000)} pps dac {cfg.get('dac') or '(nenhum)'}" if cfg else "-"
    return st


def render(st, cols=48):
    """Estado -> linhas ASCII (o console e cp1252: nada fora de ASCII sai daqui)."""
    ln = [f"spellcaster  transporte {st['state']:5}  t={st['t']:8.2f}s"]
    for n, d in sorted(st["universes"].items()):
        ln.append(f"U{n:<3}|{bar(d, cols)}| max {max(d, default=0):3}")
    ln.append("laser: " + str(st["laser"]))
    ln.append("rede : " + (", ".join(st["sources"]) or "(sem fontes; spell tui --scan)"))
    ln += ["log  : " + s for s in st["log"][-5:]]
    return [x.encode("ascii", "replace").decode() for x in ln]


def draw(st, width=118):
    """Fallback sem curses: tudo numa linha so, reescrita com \r."""
    print(("  ".join(render(st, 24)) + " " * width)[:width], end="\r", flush=True)


def _scan_bg(timeout=2):
    from .protocols import netscan

    def go():
        try:
            d = netscan.scan_all(timeout)
            SOURCES[:] = ([f"sacn {s['source_name']}" for s in d["sacn"]]
                          + [f"artnet {n['short_name']}" for n in d["artnet"]])
        except Exception as e:                     # scan e diagnostico: falhar nele nao derruba o monitor
            SOURCES[:] = [f"scan falhou: {type(e).__name__}"]
    threading.Thread(target=go, daemon=True).start()


@command
def tui(fps: int = 4, seconds: float = 0, scan: bool = False):
    """Monitor de texto (curses, ou uma linha com \r sem curses). --seconds limita; Ctrl+C encerra."""
    if scan:
        _scan_bg()
    log, prev = [], None
    end = time.perf_counter() + seconds if seconds else None

    def step():
        nonlocal prev
        st = snapshot(log)
        if st["state"] != prev:
            prev = st["state"]
            log.append(f"{time.strftime('%H:%M:%S')} transporte {prev}")
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
        import curses                              # ponytail: sem teclas, so leitura ; ler tecla quando houver transporte no tui
    except ImportError:
        curses = None
    try:
        curses.wrapper(loop) if curses else loop()
    except KeyboardInterrupt:
        pass
    print()
