# Timeline: keyframes com curvas, tracks tipadas, avaliacao por bisect (sem varredura linear).
# Os resolvedores escrevem direto nos Universes: nenhum dict e alocado por frame, exceto o que o
# proprio show devolve no track pyfx. Nada aqui importa gui/mcp.
import bisect
import importlib.util
import os

# curva do segmento que TERMINA no keyframe (como se le num .spell: "chega em 255 com inout").
# c = controles (so o bezier usa).
CURVES = {
    "linear": lambda u, c: u,
    "hold": lambda u, c: 0.0,
    "in": lambda u, c: u * u,
    "out": lambda u, c: u * (2 - u),
    "inout": lambda u, c: u * u * (3 - 2 * u),
    # ponytail: bezier cubico com os dois controles no eixo do valor e x=u (nao resolve x(u) por Newton)
    # ; trocar por cubic-bezier CSS completo quando a GUI deixar arrastar as alcas no eixo do tempo.
    "bezier": lambda u, c: 3 * (1 - u) ** 2 * u * c[0] + 3 * (1 - u) * u * u * c[1] + u ** 3,
}
BEZ = (0.42, 0.58)
MISS = object()                      # sentinela de "nunca enviado" (tracks OSC)
MEDIA = {"play": 10, "replay": 20, "stop": 28}       # Capture: ch1 de controle do reprodutor de midia
LASER_PARAMS = ("x", "y", "scale", "rot", "color")


class Keyframe:
    """(t, valor, curva, controles). A curva vale para o segmento que CHEGA neste keyframe.
    Valor: numero, lista de numeros ou texto (texto = degrau)."""
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
    return a                                          # texto/None: segura o valor anterior


class Keys:
    """Lista ordenada de Keyframe. value(t) por bisect; crossed(t0, t1) para disparo por borda."""

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
        """Keyframes com t0 < t <= t1."""
        if t1 <= t0:
            return ()
        return self.keys[bisect.bisect_right(self.times, t0):bisect.bisect_right(self.times, t1)]


def load_fn(path, name="look"):
    """Importa um .py de show e devolve a funcao f(t) (track pyfx)."""
    spec = importlib.util.spec_from_file_location("spell_fx_" + os.path.basename(path)[:-3], path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, name, None)
    if not callable(fn):
        raise ValueError(f"{path}: nao define {name}(t)")
    return fn


class Track:
    """Um track do .spell. spec e o dict cru; keys/params sao os keyframes ja indexados."""

    def __init__(self, spec, base=""):
        self.spec = spec
        self.type = spec["type"]
        self.universe = int(spec.get("universe", 1))
        adr = spec.get("address", 1)
        self.address = int(adr) if isinstance(adr, (int, float)) else 0   # OSC usa address de texto
        self.keys = Keys(spec.get("keys", ()))
        self.params = {p: Keys(spec[p]) for p in LASER_PARAMS if p in spec}
        self.last = MISS                                  # ultimo valor enviado (tracks OSC)
        self.fparams = {k: Keys(v) for k, v in spec.items() if self.type == "fixture" and isinstance(v, list)}
        self.patch = ()                                   # lista de fixtures do show (Timeline preenche)
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
    """Capture: ch1 controle (10 play / 20 replay / 28 stop), ch2 clipe."""
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
    """Tracks de um show, ja separados por destino. apply() escreve nos Universes; o Player faz a I/O."""

    # resolvedor por tipo: fn(track, t, universes). A F2 injeta o dela com
    # Timeline.resolvers["fixture"] = fn — track fixture sem resolvedor e ignorado.
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
        """Escreve os tracks DMX (ou a lista dada) nos Universes no instante t."""
        for tr in (self.dmx if tracks is None else tracks):
            r = self.resolvers.get(tr.type)
            if r is not None:
                r(tr, t, universes)


# ponytail: um show tem uma sequencia so; Sequence e Timeline sao a mesma classe
# ; separar quando existirem sub-sequencias (clipes reaproveitados na mesma timeline).
Sequence = Timeline
