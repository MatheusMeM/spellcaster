# Fixture profile: JSON with channels by name, named ranges, named wheels and a fine channel (16 bit).
import json, pathlib

from ..paths import PROFILES as DIR


class ProfileError(ValueError):
    pass


class Profile:
    """{"name", "channels": [{"name", "offset", "fine": offset|null, "ranges": {...}, "wheel": {...}}]}.

    `offset` is 0-based inside the fixture. `channels[name] = (offset, fine, names)`, with fine = -1 when
    the channel is 8 bit and `names` merging ranges and wheels into a single dict. `size` = channel footprint.
    """

    def __init__(self, d):
        self.name = d.get("name") or "?"
        self.channels = {}
        used = {}
        for c in d.get("channels") or ():
            n = c.get("name")
            if not n:
                raise ProfileError(f"{self.name}: channel without a name")
            if n in self.channels:
                raise ProfileError(f"{self.name}: channel {n!r} repeated")
            if "offset" not in c:
                raise ProfileError(f"{self.name}.{n}: no offset")
            fine = c.get("fine")
            for o in (c["offset"], fine):
                if o is None:
                    continue
                if type(o) is not int or not 0 <= o <= 511:
                    raise ProfileError(f"{self.name}.{n}: offset {o!r} outside 0..511")
                if o in used:
                    raise ProfileError(f"{self.name}: offset {o} used by {used[o]!r} and {n!r}")
                used[o] = n
            names = dict(c.get("ranges") or ())
            names.update(c.get("wheel") or ())   # ponytail: range and wheel in the same dict ; split them if some
            self.channels[n] = (c["offset"], -1 if fine is None else fine, names)   # profile repeats a name in both
        if not used:
            raise ProfileError(f"{self.name}: no channels")
        self.size = max(used) + 1

    def __repr__(self):
        return f"<Profile {self.name} {self.size}ch>"


_CACHE = {}


def load(p):
    """Profile by file name in profiles/ (without .json), by path, by dict or already built."""
    if isinstance(p, Profile):
        return p
    if isinstance(p, dict):
        return Profile(p)
    got = _CACHE.get(p)
    if got is None:
        f = pathlib.Path(p)
        if not f.suffix:
            f = DIR / f"{p}.json"
        if not f.is_file():
            raise ProfileError(f"profile {p!r} not found ({f})")
        got = _CACHE[p] = Profile(json.loads(f.read_text(encoding="utf-8")))
    return got


def names():
    """Profiles available in profiles/."""
    return sorted(f.stem for f in DIR.glob("*.json"))
