# Grupo de movings mirado como unidade: geometria calibravel, histerese de volta de pan e rampa de movimento.
import math, time

from ..core.registry import command
from .patch import PatchError

GROUPS = {}   # nome -> Group, para os comandos de calibracao acharem o grupo pelo nome


class Group:
    """Fixtures iguais miradas por direcao. Todo knob de calibracao e atributo, nao constante:
    pan/tilt de centro, amp (curso de tilt), lado (sinal do eixo x visto da plateia), zmin (piso de dz),
    slew_pan/slew_tilt (rampa por frame), pan_deg (unidades DMX por grau) e zoom_min/zoom_max.
    `defaults` sao os canais que o grupo escreve em toda mirada mesmo quando o show nao passa nada."""

    def __init__(self, patch, names, name="", pan=128, tilt=128, amp=38, lado=1, zmin=-1.2,
                 slew_pan=2.0, slew_tilt=0.03, pan_deg=255 / 540, zoom_min=0, zoom_max=255, defaults=None):
        self.patch, self.names, self.name = patch, list(names), name
        self.pan, self.tilt, self.amp, self.lado, self.zmin = pan, tilt, amp, lado, zmin
        self.slew_pan, self.slew_tilt, self.pan_deg = slew_pan, slew_tilt, pan_deg
        self.zoom_min, self.zoom_max = zoom_min, zoom_max
        self.defaults = dict(defaults or {})
        self.last = {}                       # nome -> (phi, modulo) do frame anterior
        if name:
            GROUPS[name] = self

    def aim(self, dz=0.0, dx=0.0, spread=0.0, t=0.0, wave=0.0, wph=0.8, wf=0.6, only=None, **params):
        """dz: -1 tela ... +1 plateia; dx: -1 esq ... +1 dir (visto da plateia); spread: leque simetrico
        (spread<0 = raios cruzados); wave: onda lenta (wf Hz) de dz percorrendo as unidades, fase wph.
        Pan/tilt saem em 16 bit; a volta de pan e escolhida frame a frame pela continuidade, sem saltos."""
        p = {**self.defaults, **params}
        if "zoom" in p:
            p["zoom"] = int(max(self.zoom_min, min(self.zoom_max, p["zoom"])))
        names, last, sp, st = self.names, self.last, self.slew_pan, self.slew_tilt
        n = len(names)
        for i, a in enumerate(names):
            side = (i - (n - 1) / 2) / max(1, (n - 1) / 2)             # -1 ... +1 da esquerda para a direita
            x = dx + spread * side
            z = max(self.zmin, dz + wave * math.sin(2 * math.pi * wf * t + i * wph))
            m = min(1.2, math.hypot(x, z))
            phi = math.degrees(math.atan2(z, self.lado * x)) if m > 0.02 else 0.0
            lp, lm = last.get(a, (phi, m))                             # volta de pan mais perto do frame anterior...
            phi = min((phi + k for k in (-360, 0, 360) if abs(phi + k) <= 265), key=lambda c: abs(c - lp))
            phi = lp + max(-sp, min(sp, phi - lp)); m = lm + max(-st, min(st, m - lm))          # ...e rampa limitada
            last[a] = (phi, m)
            if only is None or only == a:   # ponytail: `only` escreve uma unidade so mas roda a histerese de todas
                self.patch.set(a, pan=self.pan + phi * self.pan_deg, tilt=self.tilt + self.amp * m, **p)

    def __repr__(self):
        return f"<Group {self.name or '?'} {len(self.names)}x>"


def _group(name):
    g = GROUPS.get(name)
    if g is None:
        raise PatchError(f"grupo {name!r} desconhecido; tem {sorted(GROUPS)} (carregue o show antes)")
    return g


def _hold(patch, seconds):
    """Segura o estado atual do patch em sACN por N s (o loop dos bsw_* do seed)."""
    from ..protocols.sacn import SacnOut
    out = SacnOut(universes=sorted(patch.universes))
    t0 = time.time()
    try:
        while time.time() - t0 < seconds:
            for u in patch.universes.values():
                out.send(u.number, bytes(u.data))
            time.sleep(1 / 30)
    except KeyboardInterrupt:
        pass
    finally:
        out.close()


@command
def calib_hold(group: str, pan: int, tilt: int, zoom: int = 255, seconds: int = 600):
    """Segura um grupo em pan/tilt cru (dimmer 255, shutter aberto) e apaga os outros grupos. Era o bsw_hold.py."""
    g = _group(group)
    for other in GROUPS.values():
        for a in other.names:
            other.patch.set(a, dim=0, shutter="fechado")
    for a in g.names:
        g.patch.set(a, pan=pan, tilt=tilt, dim=255, shutter="aberto", zoom=zoom, focus=128)
    _hold(g.patch, seconds)


@command
def calib_sweep(group: str, channel: str, values: str, seconds: int = 600):
    """Um valor por unidade do grupo no mesmo canal ("0,64,128,255"), para achar roda de cor/gobo. Era o bsw_multi.py."""
    g = _group(group)
    vals = [int(v) for v in values.replace(",", " ").split()]
    if len(vals) != len(g.names):
        raise PatchError(f"{group}: {len(g.names)} unidades e {len(vals)} valores")
    for a, v in zip(g.names, vals):
        g.patch.set(a, pan=g.pan, tilt=g.tilt, dim=255, shutter="aberto", zoom=200, focus=128)
        g.patch.set(a, **{channel: v})
    _hold(g.patch, seconds)
