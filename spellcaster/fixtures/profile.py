# Perfil de aparelho: JSON com canais por nome, faixas nomeadas, rodas nomeadas e canal fine (16 bit).
import json, pathlib

DIR = pathlib.Path(__file__).resolve().parents[2] / "profiles"


class ProfileError(ValueError):
    pass


class Profile:
    """{"name", "channels": [{"name", "offset", "fine": offset|null, "ranges": {...}, "wheel": {...}}]}.

    `offset` e 0-based dentro do aparelho. `channels[nome] = (offset, fine, nomes)`, com fine = -1 quando
    o canal e de 8 bit e `nomes` juntando faixas e rodas num dicionario so. `size` = footprint em canais.
    """

    def __init__(self, d):
        self.name = d.get("name") or "?"
        self.channels = {}
        used = {}
        for c in d.get("channels") or ():
            n = c.get("name")
            if not n:
                raise ProfileError(f"{self.name}: canal sem nome")
            if n in self.channels:
                raise ProfileError(f"{self.name}: canal {n!r} repetido")
            if "offset" not in c:
                raise ProfileError(f"{self.name}.{n}: sem offset")
            fine = c.get("fine")
            for o in (c["offset"], fine):
                if o is None:
                    continue
                if type(o) is not int or not 0 <= o <= 511:
                    raise ProfileError(f"{self.name}.{n}: offset {o!r} fora de 0..511")
                if o in used:
                    raise ProfileError(f"{self.name}: offset {o} usado por {used[o]!r} e {n!r}")
                used[o] = n
            names = dict(c.get("ranges") or ())
            names.update(c.get("wheel") or ())   # ponytail: faixa e roda no mesmo dicionario ; separar se algum
            self.channels[n] = (c["offset"], -1 if fine is None else fine, names)   # perfil repetir um nome nos dois
        if not used:
            raise ProfileError(f"{self.name}: sem canais")
        self.size = max(used) + 1

    def __repr__(self):
        return f"<Profile {self.name} {self.size}ch>"


_CACHE = {}


def load(p):
    """Perfil por nome de arquivo em profiles/ (sem .json), por caminho, por dict ou ja pronto."""
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
            raise ProfileError(f"perfil {p!r} nao encontrado ({f})")
        got = _CACHE[p] = Profile(json.loads(f.read_text(encoding="utf-8")))
    return got


def names():
    """Perfis disponiveis em profiles/."""
    return sorted(f.stem for f in DIR.glob("*.json"))
