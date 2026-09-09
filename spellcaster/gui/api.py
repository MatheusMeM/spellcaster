# Comandos do registry que a GUI precisa e o resto do produto ainda nao tinha: show corrente em
# memoria (abrir/salvar/editar), transporte com player nao-bloqueante e leitura de patch/perfis/rede.
# A GUI e cliente: nenhuma logica de produto vive no JS, tudo entra por aqui.
#
# Um show corrente por processo (SHOW), mesma convencao do PATCH de fixtures/patch.py.
# ponytail: show unico global ; passar id de sessao quando a GUI abrir dois shows ao mesmo tempo.
import copy
import json
import os

from .. import player as playerpkg
from .. import show as showfile
from ..core.registry import command
from ..fixtures import Patch, PatchError
from ..fixtures.profile import names as profile_names
from ..player import player as playermod
from ..timeline.model import CURVES
from . import server as gs

NEW = {"version": 1, "name": "novo show", "fps": 30, "duration": 60.0,
       "outputs": [{"type": "sacn", "universes": [1]}],
       "patch": [], "tracks": [], "cues": [], "markers": [], "in": 0.0, "out": 60.0}

SHOW = copy.deepcopy(NEW)          # deepcopy: listas de NEW nao podem ser as do show vivo
SHOW["_dir"] = os.getcwd()
PATH = ""


def _clean(sh):
    return {k: v for k, v in sh.items() if not k.startswith("_")}


def _track(i):
    tracks = SHOW.setdefault("tracks", [])
    i = int(i)
    if not 0 <= i < len(tracks):
        raise IndexError(f"track {i}: o show tem {len(tracks)}")
    return tracks[i]


def _value(v):
    """Valor de keyframe vindo como texto: JSON quando der (255, [255,0,0]), senao o proprio texto."""
    if not isinstance(v, str):
        return v
    try:
        return json.loads(v)
    except ValueError:
        return v


# ---------------------------------------------------------------- show
@command
def show_get():
    """Show corrente (JSON). E daqui que a GUI desenha timeline, patch e transporte."""
    return _clean(SHOW)


@command
def show_set(data: str):
    """Substitui o show corrente pelo JSON dado (texto ou dict). Devolve o show corrente."""
    sh = json.loads(data) if isinstance(data, str) else data
    if not isinstance(sh, dict):
        raise ValueError("show_set: esperava um objeto JSON")
    if not isinstance(sh.get("tracks", []), list):
        raise ValueError("show_set: 'tracks' precisa ser lista")
    base = sh.get("_dir") or SHOW.get("_dir", "")
    SHOW.clear()
    SHOW.update(showfile.migrate(sh))
    SHOW["_dir"] = base
    return _clean(SHOW)


@command
def show_open(file: str):
    """Abre um .spell e passa a ser o show corrente."""
    global PATH
    sh = showfile.load(file)
    SHOW.clear()
    SHOW.update(sh)
    PATH = os.path.abspath(file)
    return _clean(SHOW)


@command
def show_save(file: str = ""):
    """Grava o show corrente (sem argumento, no caminho do ultimo show_open/show_save)."""
    global PATH
    p = file or PATH
    if not p:
        raise ValueError("show_save: sem caminho (passe file=)")
    showfile.save(p, SHOW)
    PATH = os.path.abspath(p)
    SHOW["_dir"] = os.path.dirname(PATH)
    return PATH


@command
def show_new():
    """Zera o show corrente."""
    global PATH
    SHOW.clear()
    SHOW.update(copy.deepcopy(NEW))
    SHOW["_dir"] = os.getcwd()
    PATH = ""
    return _clean(SHOW)


# ---------------------------------------------------------------- tracks e keyframes
@command
def track_add(type: str = "dmx", universe: int = 1, address: int = 1, label: str = ""):
    """Acrescenta um track vazio ao show corrente. Devolve o indice do track.
    O nome do track chama-se `label` porque `registry.call(name, **kwargs)` ja usa `name` de posicional."""
    tr = {"type": type, "universe": universe, "address": address, "keys": []}
    if label:
        tr["name"] = label
    SHOW.setdefault("tracks", []).append(tr)
    return len(SHOW["tracks"]) - 1


@command
def track_del(index: int):
    """Remove o track de indice dado. Devolve o track removido."""
    _track(index)
    return SHOW["tracks"].pop(int(index))


@command
def key_set(track: int, t: float, value: str = "0", curve: str = "linear"):
    """Cria ou move o keyframe do track em t. value e JSON (255, [255,0,0]) ou texto ("amarelo")."""
    if curve not in CURVES:
        raise ValueError(f"curva {curve!r}: use {sorted(CURVES)}")
    ks = _track(track).setdefault("keys", [])
    v, t = _value(value), float(t)
    for i, k in enumerate(ks):
        if abs(float(k[0]) - t) < 1e-6:
            ks[i] = [t, v, curve]
            break
    else:
        ks.append([t, v, curve])
    ks.sort(key=lambda k: k[0])
    return ks


@command
def key_del(track: int, t: float):
    """Apaga o keyframe do track em t (tolerancia 1 ms). Devolve quantos sairam."""
    ks = _track(track).setdefault("keys", [])
    n = len(ks)
    ks[:] = [k for k in ks if abs(float(k[0]) - float(t)) > 1e-3]
    return n - len(ks)


# ---------------------------------------------------------------- transporte
class _Tap:
    """Saida DMX falsa: publica cada frame do player no topico binario 'dmx' da GUI.
    Cumpre o contrato send/close, entao entra na lista de saidas do Engine como qualquer protocolo."""

    def send(self, universe, data):
        srv = gs.SERVER
        if srv is not None and srv.clients:
            srv.push("dmx", data, universe)

    def close(self):
        pass


def _live(sh):
    """Show sem os tracks mudos (ou so os em solo): mute/solo da GUI valem no que toca de verdade."""
    tracks = sh.get("tracks") or []
    solo = [t for t in tracks if t.get("solo")]
    return {**sh, "tracks": solo or [t for t in tracks if not t.get("mute")]}


@command
def transport_state():
    """Estado do transporte: state, t, dur, loop."""
    p = playermod.CURRENT
    return {"state": p.clock.state if p else "stop", "t": p.clock.time if p else 0.0,
            "dur": SHOW.get("duration") or 60.0, "loop": bool(p.loop) if p else False}


@command
def transport(state: str = "play", t: float = -1.0, loop: bool = False):
    """play | pause | stop do show corrente, sem bloquear (o `play_show` da CLI bloqueia).
    Cada frame DMX vai para o topico 'dmx' da GUI. t >= 0 localiza antes de tocar."""
    if state not in ("play", "pause", "stop"):
        raise ValueError(f"transport: {state!r} nao e play/pause/stop")
    p = playermod.CURRENT
    if state == "play":
        if p is None:
            p = playermod.CURRENT = playermod.Player(_live(SHOW), loop=loop)
            p.eng.outputs.append(_Tap())
            if p.art:
                p.art.outputs.append(_Tap())
            p.start()
        p.loop = loop
        if t >= 0:
            p.locate(t)
        p.play()
    elif p is not None:
        if state == "pause":
            p.pause()
        else:
            p.close()                       # stop: fecha sockets e threads; o proximo play cria outro
            playermod.CURRENT = None
    return transport_state()


# ---------------------------------------------------------------- patch, perfis, rede
@command
def patch_check():
    """Monta o patch do show corrente e devolve as linhas para a grade + o erro de sobreposicao."""
    p, err = Patch(), None
    for f in SHOW.get("patch", ()) or ():
        try:
            p.add(f["name"], f["profile"], int(f.get("universe", 1)), int(f["address"]))
        except (PatchError, ValueError, KeyError, OSError) as e:
            err = f"{type(e).__name__}: {e}"
            break
    return {"rows": p.rows(), "error": err}


@command
def profiles():
    """Nomes dos perfis disponiveis em profiles/."""
    return profile_names()


@command
def net_report(timeout: int = 2):
    """Analise de rede como dict (o comando `net` so imprime): relatorio, sugestoes e listas cruas."""
    from ..protocols import netscan
    d = netscan.scan_all(timeout)
    # ponytail: scan sincrono trava a conexao ws do cliente por ~timeout+1 s ; jogar numa thread
    # com push("net", ...) se alguem reclamar da GUI congelada durante o Rescan.
    return {"report": netscan.report(d), **d}


_ = playerpkg  # noqa: F401  (importar o pacote player registra play_show/stop/pause/locate/markers)
