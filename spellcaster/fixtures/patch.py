# Patch: fixture nomeada = perfil + universo + endereco. Escreve por nome de canal direto no Universes do core.
import json

from ..core.registry import command
from ..core.universe import Universes
from .profile import load

PATCH = None   # ponytail: o ultimo Patch criado e o corrente do processo (um show por processo) ;
               # trocar por selecao explicita quando a GUI abrir dois shows ao mesmo tempo.


class PatchError(ValueError):
    pass


class Fixture:
    """Resolvida no add: `chans[nome] = (indice, indice_fine, nomes)` ja em coordenada de universo."""
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
        self._busy = {}                      # (universo, canal) -> nome da fixture
        global PATCH
        PATCH = self

    def add(self, name, profile, universe, address):
        """Erro na hora se o footprint do perfil pisa em outra fixture ou passa de 512."""
        if name in self.fixtures:
            raise PatchError(f"fixture {name!r} ja esta no patch")
        p = load(profile)
        if not 1 <= address <= 512 or address + p.size - 1 > 512:
            raise PatchError(f"{name} [{p.name}]: endereco {address} + {p.size} ch passa de 512 (universo {universe})")
        for a in range(address, address + p.size):
            other = self._busy.get((universe, a))
            if other is not None:
                o = self.fixtures[other]
                raise PatchError(f"sobreposicao no universo {universe} canal {a}: {other!r} "
                                 f"[{o.profile.name}, {o.profile.size} ch em {o.address}] e {name!r} "
                                 f"[{p.name}, {p.size} ch em {address}]")
        u = self.universes.get_or_create(universe)
        for a in range(address, address + p.size):
            self._busy[(universe, a)] = name
        fx = self.fixtures[name] = Fixture(name, p, universe, address, u.data)
        return fx

    def set(self, name, **params):
        """Valor: int/float em 0-255, float em 0.0-1.0 tratado como fracao, nome de faixa ou nome de roda.
        Canal com `fine` vai a 16 bit sozinho."""
        fx = self.fixtures.get(name)
        if fx is None:
            raise PatchError(f"fixture {name!r} nao esta no patch")
        data, chans = fx.data, fx.chans
        for ch, v in params.items():
            c = chans.get(ch)
            if c is None:
                raise PatchError(f"{name} [{fx.profile.name}]: sem canal {ch!r}; tem {sorted(chans)}")
            k = v.__class__
            if k is str:
                v = c[2].get(v)
                if v is None:
                    raise PatchError(f"{name}.{ch}: {params[ch]!r} nao e faixa nem roda; tem {sorted(c[2])}")
            elif k is float and 0.0 <= v <= 1.0:
                v *= 255.0   # ponytail: float em 0..1 e fracao ; passe int quando o valor puder cair em 0..1 em DMX cru
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
        """Valor cru do canal (tupla hi, lo quando o canal e 16 bit). Para teste e para a GUI ler o estado."""
        fx = self.fixtures[name]
        i, f, _ = fx.chans[ch]
        return fx.data[i] if f < 0 else (fx.data[i], fx.data[f])

    def rows(self):
        return [{"name": f.name, "profile": f.profile.name, "universe": f.universe, "address": f.address,
                 "channels": f.profile.size} for f in self.fixtures.values()]


@command
def patch_list():
    """Lista as fixtures do patch corrente (JSON): nome, perfil, universo, endereco, canais."""
    rows = PATCH.rows() if PATCH is not None else []
    print(json.dumps(rows, indent=1, ensure_ascii=False))
    return rows
