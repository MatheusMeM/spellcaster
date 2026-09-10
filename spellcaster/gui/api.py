# Registry commands the GUI needs and the rest of the product did not have yet: current show in
# memory (open/save/edit), transport with a non-blocking player and reading of patch/profiles/network.
# The GUI is a client: no product logic lives in the JS, everything comes in through here.
#
# One current show per process (SHOW), same convention as the PATCH in fixtures/patch.py.
# ponytail: a single global show ; pass a session id when the GUI opens two shows at the same time.
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

NEW = {"version": 1, "name": "new show", "fps": 30, "duration": 60.0,
       "outputs": [{"type": "sacn", "universes": [1]}],
       "patch": [], "tracks": [], "cues": [], "markers": [], "in": 0.0, "out": 60.0}

SHOW = copy.deepcopy(NEW)          # deepcopy: the lists in NEW must not be the ones of the live show
SHOW["_dir"] = os.getcwd()
PATH = ""


def _clean(sh):
    return {k: v for k, v in sh.items() if not k.startswith("_")}


def _track(i):
    tracks = SHOW.setdefault("tracks", [])
    i = int(i)
    if not 0 <= i < len(tracks):
        raise IndexError(f"track {i}: the show has {len(tracks)}")
    return tracks[i]


def _value(v):
    """Keyframe value arriving as text: JSON when it parses (255, [255,0,0]), otherwise the text itself."""
    if not isinstance(v, str):
        return v
    try:
        return json.loads(v)
    except ValueError:
        return v


# ---------------------------------------------------------------- show
@command
def show_get():
    """Current show (JSON). This is what the GUI draws the timeline, patch and transport from."""
    return _clean(SHOW)


@command
def show_set(data: str):
    """Replaces the current show with the given JSON (text or dict). Returns the current show."""
    sh = json.loads(data) if isinstance(data, str) else data
    if not isinstance(sh, dict):
        raise ValueError("show_set: expected a JSON object")
    if not isinstance(sh.get("tracks", []), list):
        raise ValueError("show_set: 'tracks' must be a list")
    base = sh.get("_dir") or SHOW.get("_dir", "")
    SHOW.clear()
    SHOW.update(showfile.migrate(sh))
    SHOW["_dir"] = base
    return _clean(SHOW)


@command
def show_open(file: str):
    """Opens a .spell and makes it the current show."""
    global PATH
    sh = showfile.load(file)
    SHOW.clear()
    SHOW.update(sh)
    PATH = os.path.abspath(file)
    return _clean(SHOW)


@command
def show_save(file: str = ""):
    """Saves the current show (with no argument, to the path of the last show_open/show_save)."""
    global PATH
    p = file or PATH
    if not p:
        raise ValueError("show_save: no path (pass file=)")
    showfile.save(p, SHOW)
    PATH = os.path.abspath(p)
    SHOW["_dir"] = os.path.dirname(PATH)
    return PATH


@command
def show_new():
    """Resets the current show."""
    global PATH
    SHOW.clear()
    SHOW.update(copy.deepcopy(NEW))
    SHOW["_dir"] = os.getcwd()
    PATH = ""
    return _clean(SHOW)


# ---------------------------------------------------------------- tracks and keyframes
@command
def track_add(type: str = "dmx", universe: int = 1, address: int = 1, label: str = ""):
    """Appends an empty track to the current show. Returns the track index.
    The track name is called `label` because `registry.call(name, **kwargs)` already uses `name` positionally."""
    tr = {"type": type, "universe": universe, "address": address, "keys": []}
    if label:
        tr["name"] = label
    SHOW.setdefault("tracks", []).append(tr)
    return len(SHOW["tracks"]) - 1


@command
def track_del(index: int):
    """Removes the track at the given index. Returns the removed track."""
    _track(index)
    return SHOW["tracks"].pop(int(index))


@command
def key_set(track: int, t: float, value: str = "0", curve: str = "linear"):
    """Creates or moves the track keyframe at t. value is JSON (255, [255,0,0]) or text ("amarelo")."""
    if curve not in CURVES:
        raise ValueError(f"curve {curve!r}: use {sorted(CURVES)}")
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
    """Deletes the track keyframe at t (1 ms tolerance). Returns how many were removed."""
    ks = _track(track).setdefault("keys", [])
    n = len(ks)
    ks[:] = [k for k in ks if abs(float(k[0]) - float(t)) > 1e-3]
    return n - len(ks)


# ---------------------------------------------------------------- transport
class _Tap:
    """Fake DMX output: publishes every player frame on the GUI binary topic 'dmx'.
    It honours the send/close contract, so it joins the Engine output list like any protocol."""

    def send(self, universe, data):
        srv = gs.SERVER
        if srv is not None and srv.clients:
            srv.push("dmx", data, universe)

    def close(self):
        pass


def _live(sh):
    """Show without the muted tracks (or only the soloed ones): the GUI mute/solo apply to what actually plays."""
    tracks = sh.get("tracks") or []
    solo = [t for t in tracks if t.get("solo")]
    return {**sh, "tracks": solo or [t for t in tracks if not t.get("mute")]}


@command
def transport_state():
    """Transport state: state, t, dur, loop."""
    p = playermod.CURRENT
    return {"state": p.clock.state if p else "stop", "t": p.clock.time if p else 0.0,
            "dur": SHOW.get("duration") or 60.0, "loop": bool(p.loop) if p else False}


@command
def transport(state: str = "play", t: float = -1.0, loop: bool = False):
    """play | pause | stop of the current show, without blocking (the CLI `play_show` blocks).
    Every DMX frame goes to the GUI 'dmx' topic. t >= 0 locates before playing."""
    if state not in ("play", "pause", "stop"):
        raise ValueError(f"transport: {state!r} is not play/pause/stop")
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
            p.close()                       # stop: closes sockets and threads; the next play creates another one
            playermod.CURRENT = None
    return transport_state()


# ---------------------------------------------------------------- patch, profiles, network
@command
def patch_check():
    """Builds the patch of the current show and returns the rows for the grid + the overlap error."""
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
    """Names of the profiles available in profiles/."""
    return profile_names()


_ = playerpkg  # noqa: F401  (importing the player package registers play_show/stop/pause/locate/markers)
