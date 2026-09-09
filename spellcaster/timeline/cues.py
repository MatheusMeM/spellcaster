# Cue list: GO manual, wait antes de comecar, follow automatico ao terminar, fade linear entre snapshots.
# Snapshot = {"universo/endereco": [valores]} no .spell; vira {(universo, endereco): [valores]}.


def key(s):
    """'1/100' -> (1, 100); '100' -> (1, 100)."""
    u, _, a = str(s).partition("/")
    return (int(u), int(a)) if a else (1, int(u))


class Cue:
    def __init__(self, spec):
        self.name = spec.get("name", "")
        self.fade = float(spec.get("fade", 0.0))
        self.wait = float(spec.get("wait", 0.0))          # atraso entre o gatilho e o inicio do fade
        self.follow = bool(spec.get("follow", False))     # ao terminar, dispara a proxima
        self.values = {key(k): (list(v) if isinstance(v, list) else [v]) for k, v in spec.get("values", {}).items()}

    def __repr__(self):
        return f"Cue({self.name!r}, fade={self.fade}, wait={self.wait}, follow={self.follow})"


class CueList:
    """update(t) devolve o snapshot corrente {(universo, endereco): [valores]}; o Player aplica nos Universes."""

    def __init__(self, cues=()):
        self.cues = [c if isinstance(c, Cue) else Cue(c) for c in cues]
        self.state = {}
        self.index = -1
        self._cue = None
        self._t0 = 0.0
        self._from = {}
        self._pending = None                              # (indice, instante do gatilho)

    def go(self, t, index=None):
        """Dispara a proxima cue (ou a de indice dado). O fade comeca depois do wait dela."""
        i = self.index + 1 if index is None else int(index)
        if not (0 <= i < len(self.cues)):
            return None
        self._pending = (i, t)
        return self.cues[i]

    def update(self, t):
        if self._pending is not None:
            i, t0 = self._pending
            if t >= t0 + self.cues[i].wait:
                self._pending = None
                self.index, self._cue, self._t0 = i, self.cues[i], t
                self._from = {k: list(v) for k, v in self.state.items()}
        c = self._cue
        if c is not None:
            u = 1.0 if c.fade <= 0 else max(0.0, min(1.0, (t - self._t0) / c.fade))
            for k, v in c.values.items():
                if u >= 1.0:
                    self.state[k] = list(v)
                else:
                    a = self._from.get(k, ())
                    a = list(a) + [0.0] * (len(v) - len(a))
                    self.state[k] = [x + (y - x) * u for x, y in zip(a, v)]
            if u >= 1.0:
                self._cue = None
                if c.follow:
                    self.go(t)
        return self.state

    def reset(self):
        self.state.clear()
        self.index, self._cue, self._pending = -1, None, None
        self._from = {}
