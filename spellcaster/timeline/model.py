# Timeline: keyframes with curves, typed tracks, evaluation by bisect (no linear scan).
# The resolvers write straight into the Universes: no dict is allocated per frame, except the one the
# show itself returns in the pyfx track. Nothing here imports gui/mcp.
import bisect
import importlib.util
import os

# curve of the segment that ENDS at the keyframe (the way a .spell reads: "arrives at 255 with inout").
# c = controls (only bezier uses them).
CURVES = {
    "linear": lambda u, c: u,
    "hold": lambda u, c: 0.0,
    "in": lambda u, c: u * u,
    "out": lambda u, c: u * (2 - u),
    "inout": lambda u, c: u * u * (3 - 2 * u),
    # ponytail: cubic bezier with both controls on the value axis and x=u (it does not solve x(u) by Newton)
    # ; swap for a full CSS cubic-bezier once the GUI lets the handles be dragged along the time axis.
    "bezier": lambda u, c: 3 * (1 - u) ** 2 * u * c[0] + 3 * (1 - u) * u * u * c[1] + u ** 3,
}
BEZ = (0.42, 0.58)
MISS = object()                      # "never sent" sentinel (OSC tracks)
MEDIA = {"play": 10, "replay": 20, "stop": 28}       # Capture: ch1 controlling the media player
LASER_PARAMS = ("x", "y", "scale", "rot", "color")


class Keyframe:
    """(t, value, curve, controls). The curve applies to the segment that ARRIVES at this keyframe.
    Value: number, list of numbers or text (text = step)."""
    __slots__ = ("t", "value", "curve", "c")

    def __init__(self, t, value, curve="linear", c=BEZ):
        self.t, self.value, self.curve, self.c = float(t), value, curve, c

    def __repr__(self):
        return f"Keyframe({self.t}, {self.value!r}, {self.curve!r})"


def lerp(a, b, e):
    if isinstance(a, list) and isinstance(b, list):
        return [x + (y - x) * e for x, y in zip(a, b)]
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a + (b - a) * e
    return a                                          # text/None: holds the previous value


class Keys:
    """Sorted list of Keyframe. value(t) by bisect; crossed(t0, t1) for edge triggering."""

    def __init__(self, raw=()):
        ks = [k if isinstance(k, Keyframe) else Keyframe(*k) for k in raw]
        ks.sort(key=lambda k: k.t)
        self.keys = ks
        self.times = [k.t for k in ks]

    def __len__(self):
        return len(self.keys)

    def value(self, t, default=None):
        ks = self.keys
        if not ks:
            return default
        i = bisect.bisect_right(self.times, t) - 1
        if i < 0:
            return ks[0].value
        k = ks[i]
        if i + 1 == len(ks):
            return k.value
        n = ks[i + 1]
        dt = n.t - k.t
        if dt <= 0:
            return n.value
        return lerp(k.value, n.value, CURVES[n.curve]((t - k.t) / dt, n.c))

    def crossed(self, t0, t1):
        """Keyframes with t0 < t <= t1."""
        if t1 <= t0:
            return ()
        return self.keys[bisect.bisect_right(self.times, t0):bisect.bisect_right(self.times, t1)]


def load_fn(path, name="look"):
    """Imports a show .py and returns the f(t) function (pyfx track)."""
    spec = importlib.util.spec_from_file_location("spell_fx_" + os.path.basename(path)[:-3], path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, name, None)
    if not callable(fn):
        raise ValueError(f"{path}: does not define {name}(t)")
    return fn


class Track:
    """One .spell track. spec is the raw dict; keys/params are the already indexed keyframes."""

    def __init__(self, spec, base=""):
        self.spec = spec
        self.type = spec["type"]
        self.universe = int(spec.get("universe", 1))
        adr = spec.get("address", 1)
        self.address = int(adr) if isinstance(adr, (int, float)) else 0   # OSC uses a text address
        self.keys = Keys(spec.get("keys", ()))
        self.params = {p: Keys(spec[p]) for p in LASER_PARAMS if p in spec}
        self.last = MISS                                  # last value sent (OSC tracks)
        self.fparams = {k: Keys(v) for k, v in spec.items() if self.type == "fixture" and isinstance(v, list)}
        self.patch = ()                                   # the show's fixture list (Timeline fills it in)
        self.fn = load_fn(os.path.join(base, spec["file"]), spec.get("fn", "look")) if self.type == "pyfx" else None

    def value(self, t, default=None):
        return self.keys.value(t, default)

    def param(self, name, t, default=None):
        k = self.params.get(name)
        return k.value(t, default) if k else default

    def __repr__(self):
        return f"Track({self.type!r}, u{self.universe}@{self.address}, {len(self.keys)} keys)"


def _dmx(tr, t, uni):
    v = tr.keys.value(t)
    if v is not None:
        uni.get_or_create(tr.universe).set(tr.address, v if isinstance(v, list) else (v,))


def _media(tr, t, uni):
    """Capture: ch1 control (10 play / 20 replay / 28 stop), ch2 clip."""
    v = tr.keys.value(t)
    if v is not None:
        uni.get_or_create(tr.universe).set(tr.address, (MEDIA.get(v, 0), int(tr.spec.get("clip", 0))))


def _pyfx(tr, t, uni):
    for key, vals in tr.fn(t).items():
        u, a = key if isinstance(key, tuple) else (tr.universe, key)
        uni.get_or_create(u).set(a, vals)


def is_capture(tr):
    return tr.type == "media" and tr.spec.get("player", "capture") == "capture"


class Timeline:
    """A show's tracks, already split by destination. apply() writes into the Universes; the Player does the I/O."""

    # resolver per type: fn(track, t, universes). F2 injects its own with
    # Timeline.resolvers["fixture"] = fn -- a fixture track without a resolver is ignored.
    resolvers = {"dmx": _dmx, "artnet": _dmx, "media": _media, "pyfx": _pyfx}

    def __init__(self, show, base=""):
        self.show = show
        self.fps = int(show.get("fps", 30))
        self.duration = show.get("duration")
        self.tracks = [Track(s, base) for s in show.get("tracks", ())]
        for tr in self.tracks:
            tr.patch = show.get("patch", ())
        self.dmx = [k for k in self.tracks if k.type in ("dmx", "pyfx", "fixture") or is_capture(k)]
        self.artnet = [k for k in self.tracks if k.type == "artnet"]
        self.osc = [k for k in self.tracks if k.type == "osc" or (k.type == "media" and not is_capture(k))]
        self.cue = [k for k in self.tracks if k.type == "cue"]
        self.laser = [k for k in self.tracks if k.type == "laser"]

    def apply(self, universes, t, tracks=None):
        """Writes the DMX tracks (or the given list) into the Universes at instant t."""
        for tr in (self.dmx if tracks is None else tracks):
            r = self.resolvers.get(tr.type)
            if r is not None:
                r(tr, t, universes)


# ponytail: a show has a single sequence; Sequence and Timeline are the same class
# ; split them when sub-sequences exist (clips reused inside the same timeline).
Sequence = Timeline
