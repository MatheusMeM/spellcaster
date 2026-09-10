# Group of moving heads aimed as one unit: calibratable geometry, pan-wrap hysteresis and movement ramp.
import math, time

from ..core.registry import command
from .patch import PatchError

GROUPS = {}   # name -> Group, so the calibration commands can find the group by name


class Group:
    """Identical fixtures aimed by direction. Every calibration knob is an attribute, not a constant:
    centre pan/tilt, amp (tilt travel), side (sign of the x axis seen from the audience), zmin (dz floor),
    slew_pan/slew_tilt (ramp per frame), pan_deg (DMX units per degree) and zoom_min/zoom_max.
    `defaults` are the channels the group writes on every aim even when the show passes nothing."""

    def __init__(self, patch, names, name="", pan=128, tilt=128, amp=38, side=1, zmin=-1.2,
                 slew_pan=2.0, slew_tilt=0.03, pan_deg=255 / 540, zoom_min=0, zoom_max=255, defaults=None):
        self.patch, self.names, self.name = patch, list(names), name
        self.pan, self.tilt, self.amp, self.side, self.zmin = pan, tilt, amp, side, zmin
        self.slew_pan, self.slew_tilt, self.pan_deg = slew_pan, slew_tilt, pan_deg
        self.zoom_min, self.zoom_max = zoom_min, zoom_max
        self.defaults = dict(defaults or {})
        self.last = {}                       # name -> (phi, magnitude) of the previous frame
        if name:
            GROUPS[name] = self

    def aim(self, dz=0.0, dx=0.0, spread=0.0, t=0.0, wave=0.0, wph=0.8, wf=0.6, only=None, **params):
        """dz: -1 screen ... +1 audience; dx: -1 left ... +1 right (seen from the audience); spread: symmetric fan
        (spread<0 = crossed beams); wave: slow wave (wf Hz) of dz travelling across the units, phase wph.
        Pan/tilt come out in 16 bit; the pan wrap is picked frame by frame by continuity, without jumps."""
        p = {**self.defaults, **params}
        if "zoom" in p:
            p["zoom"] = int(max(self.zoom_min, min(self.zoom_max, p["zoom"])))
        names, last, sp, st = self.names, self.last, self.slew_pan, self.slew_tilt
        n = len(names)
        for i, a in enumerate(names):
            offs = (i - (n - 1) / 2) / max(1, (n - 1) / 2)             # -1 ... +1 from left to right
            x = dx + spread * offs
            z = max(self.zmin, dz + wave * math.sin(2 * math.pi * wf * t + i * wph))
            m = min(1.2, math.hypot(x, z))
            phi = math.degrees(math.atan2(z, self.side * x)) if m > 0.02 else 0.0
            lp, lm = last.get(a, (phi, m))                             # pan wrap closest to the previous frame...
            phi = min((phi + k for k in (-360, 0, 360) if abs(phi + k) <= 265), key=lambda c: abs(c - lp))
            phi = lp + max(-sp, min(sp, phi - lp)); m = lm + max(-st, min(st, m - lm))          # ...and a limited ramp
            last[a] = (phi, m)
            if only is None or only == a:   # ponytail: `only` writes a single unit but runs the hysteresis of all of them
                self.patch.set(a, pan=self.pan + phi * self.pan_deg, tilt=self.tilt + self.amp * m, **p)

    def __repr__(self):
        return f"<Group {self.name or '?'} {len(self.names)}x>"


def _group(name):
    g = GROUPS.get(name)
    if g is None:
        raise PatchError(f"unknown group {name!r}; it has {sorted(GROUPS)} (load the show first)")
    return g


def _hold(patch, seconds):
    """Holds the current patch state on sACN for N s."""
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
    """Holds a group at raw pan/tilt (dimmer 255, shutter open) and blacks out the other groups. Used to be bsw_hold.py."""
    g = _group(group)
    for other in GROUPS.values():
        for a in other.names:
            other.patch.set(a, dim=0, shutter="fechado")
    for a in g.names:
        g.patch.set(a, pan=pan, tilt=tilt, dim=255, shutter="aberto", zoom=zoom, focus=128)
    _hold(g.patch, seconds)


@command
def calib_sweep(group: str, channel: str, values: str, seconds: int = 600):
    """One value per group unit on the same channel ("0,64,128,255"), to find a colour/gobo wheel. Used to be bsw_multi.py."""
    g = _group(group)
    vals = [int(v) for v in values.replace(",", " ").split()]
    if len(vals) != len(g.names):
        raise PatchError(f"{group}: {len(g.names)} units and {len(vals)} values")
    for a, v in zip(g.names, vals):
        g.patch.set(a, pan=g.pan, tilt=g.tilt, dim=255, shutter="aberto", zoom=200, focus=128)
        g.patch.set(a, **{channel: v})
    _hold(g.patch, seconds)
