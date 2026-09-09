# Comandos que so existem por causa do MCP (resumo do show, monitor de saida).
# Entram no registry como qualquer outro: a CLI e a GUI tambem os enxergam.
import base64
import json

from ..core.registry import command

OPEN = {}        # ponytail: um show aberto por processo, igual ao PATCH de fixtures/patch.py
                 # ; trocar por id de sessao quando a GUI abrir dois shows ao mesmo tempo.


def current():
    """Show corrente: o do player rodando; senao o ultimo aberto por show_summary(file)."""
    from ..player import player as pl
    if pl.CURRENT is not None:
        return pl.CURRENT.show
    return OPEN or None


def summary(file=""):
    """Resumo do show sem imprimir nada (usado pelo resource spell://show)."""
    from .. import show as showfile
    if file:
        OPEN.clear()
        OPEN.update(showfile.load(file))
    sh = current()
    if sh is None:
        return {"aberto": False, "dica": "chame show_summary(file=...) ou play_show(file=...)"}
    from ..player import player as pl
    p = pl.CURRENT
    return {"aberto": True,
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
    """Resumo do .spell aberto (ou do arquivo dado): nome, fps, duracao, saidas, tracks, cues, patch."""
    d = summary(file)
    print(json.dumps(d, indent=1, ensure_ascii=False))
    return d


@command
def monitor(universe: int = 1):
    """512 bytes do universo no player em execucao, em base64 (JSON com universe, t e b64)."""
    from ..player import player as pl
    p = pl.CURRENT
    # ponytail: le so o engine sACN ; ler tambem o engine Art-Net quando alguem pedir monitor de Art-Net.
    u = None if p is None else p.eng.universes.get(int(universe))
    d = {"universe": int(universe),
         "t": None if p is None else round(p.clock.time, 3),
         "playing": None if p is None else p.clock.state,
         "b64": base64.b64encode(bytes(u.data) if u is not None else bytes(512)).decode()}
    print(json.dumps(d))
    return d
