# Patch: named fixture = profile + universe + address. Writes by channel name straight into the core Universes.
import json

from ..core.registry import command
from ..core.universe import Universes
from .profile import load

PATCH = None   # ponytail: the last Patch created is the process-wide current one (one show per process) ;
               # swap for an explicit selection when the GUI opens two shows at the same time.


class PatchError(ValueError):
    pass


class Fixture:
    """Resolved on add: `chans[name] = (index, fine_index, names)` already in universe coordinates."""
    __slots__ = ("name", "profile", "universe", "address", "data", "chans")

    def __init__(self, name, profile, universe, address, data):
        self.name, self.profile, self.universe, self.address, self.data = name, profile, universe, address, data
        b = address - 1
        self.chans = {n: (b + o, b + f if f >= 0 else -1, v) for n, (o, f, v) in profile.channels.items()}

    def __repr__(self):
        return f"<Fixture {self.name} [{self.profile.name}] U{self.universe}/{self.address}>"


class Patch:
    def __init__(self, universes=None):
        self.universes = Universes() if universes is None else universes
        self.fixtures = {}
        self._busy = {}                      # (universe, channel) -> fixture name
        global PATCH
        PATCH = self

    def add(self, name, profile, universe, address):
        """Fails right away if the profile footprint steps on another fixture or runs past 512."""
        if name in self.fixtures:
            raise PatchError(f"fixture {name!r} is already in the patch")
        p = load(profile)
        if not 1 <= address <= 512 or address + p.size - 1 > 512:
            raise PatchError(f"{name} [{p.name}]: address {address} + {p.size} ch runs past 512 (universe {universe})")
        for a in range(address, address + p.size):
            other = self._busy.get((universe, a))
            if other is not None:
                o = self.fixtures[other]
                raise PatchError(f"overlap on universe {universe} channel {a}: {other!r} "
                                 f"[{o.profile.name}, {o.profile.size} ch at {o.address}] and {name!r} "
                                 f"[{p.name}, {p.size} ch at {address}]")
        u = self.universes.get_or_create(universe)
        for a in range(address, address + p.size):
            self._busy[(universe, a)] = name
        fx = self.fixtures[name] = Fixture(name, p, universe, address, u.data)
        return fx

    def set(self, name, **params):
        """Value: int/float in 0-255, float in 0.0-1.0 treated as a fraction, range name or wheel name.
        A channel with `fine` goes to 16 bit on its own."""
        fx = self.fixtures.get(name)
        if fx is None:
            raise PatchError(f"fixture {name!r} is not in the patch")
        data, chans = fx.data, fx.chans
        for ch, v in params.items():
            c = chans.get(ch)
            if c is None:
                raise PatchError(f"{name} [{fx.profile.name}]: no channel {ch!r}; it has {sorted(chans)}")
            k = v.__class__
            if k is str:
                v = c[2].get(v)
                if v is None:
                    raise PatchError(f"{name}.{ch}: {params[ch]!r} is neither a range nor a wheel; it has {sorted(c[2])}")
            elif k is float and 0.0 <= v <= 1.0:
                v *= 255.0   # ponytail: a float in 0..1 is a fraction ; pass an int when the value may land in 0..1 as raw DMX
            if v < 0:
                v = 0
            elif v > 255:
                v = 255
            f = c[1]
            if f < 0:
                data[c[0]] = int(v)
            else:
                w = int(v * 257)
                data[c[0]] = w >> 8
                data[f] = w & 255

    def get(self, name, ch):
        """Raw channel value (a (hi, lo) tuple when the channel is 16 bit). For tests and for the GUI to read state."""
        fx = self.fixtures[name]
        i, f, _ = fx.chans[ch]
        return fx.data[i] if f < 0 else (fx.data[i], fx.data[f])

    def rows(self):
        return [{"name": f.name, "profile": f.profile.name, "universe": f.universe, "address": f.address,
                 "channels": f.profile.size} for f in self.fixtures.values()]


@command
def patch_list():
    """Lists the fixtures of the current patch (JSON): name, profile, universe, address, channels."""
    rows = PATCH.rows() if PATCH is not None else []
    print(json.dumps(rows, indent=1, ensure_ascii=False))
    return rows
