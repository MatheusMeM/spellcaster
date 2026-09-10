# `spell` CLI: every verb is a registry command.
import argparse, importlib.util, json, sys

from . import __version__
from .core.registry import command, schema, call
from .core.engine import Engine
from . import fixtures, player, gui, tui, config, mcp  # noqa: F401  (they register their commands in the registry)


def load_show(path):
    """Imports a show .py (compatibility mode): needs look(t); DUR is optional."""
    spec = importlib.util.spec_from_file_location("show", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    if not callable(getattr(mod, "look", None)):
        sys.exit(f"{path}: does not define look(t)")
    return mod


@command
def play(file: str, fps: int = 30, loop: bool = False, universes: str = "1"):
    """Plays a .py show (look(t), DUR) over sACN."""
    from .protocols.sacn import SacnOut
    mod = load_show(file)
    dur = getattr(mod, "DUR", None)
    out = SacnOut(universes=[int(u) for u in universes.split(",")])
    eng = Engine([out], fps=fps)
    print(f"sACN universes {out.universes} via {out.ifaces}; {dur}s per pass; {fps} fps", flush=True)
    try:
        while True:
            eng.run(mod.look, dur)
            if not loop or dur is None:
                break
    except KeyboardInterrupt:
        pass
    finally:
        out.close()


@command
def net(timeout: int = 2):
    """Scans the network: interfaces, Art-Net nodes, sACN sources, Ether Dream DACs, suggestions.
    Returns the scan dict (GUI and MCP read from here); `report` is the text that gets printed."""
    from .protocols import netscan
    d = netscan.scan_all(timeout)
    d["report"] = netscan.report(d)
    print(d["report"])
    return d


@command
def commands():
    """Lists the registry commands (JSON)."""
    print(json.dumps(schema(), indent=1, ensure_ascii=False))


def main(argv=None):
    ap = argparse.ArgumentParser(prog="spell")
    ap.add_argument("--version", action="version", version=f"spell {__version__}")
    sub = ap.add_subparsers(dest="cmd", required=True)
    for c in schema():                              # one subcommand per registry entry
        p = sub.add_parser(c["name"], help=c["doc"].split("\n")[0])
        for prm in c["params"]:
            if prm["default"] is None:
                p.add_argument(prm["name"])
            elif prm["type"] == "bool":
                p.add_argument(f"--{prm['name']}", action="store_true")
            else:
                p.add_argument(f"--{prm['name']}", default=prm["default"])
    a = vars(ap.parse_args(argv))
    return call(a.pop("cmd"), **a)


if __name__ == "__main__":
    main()
