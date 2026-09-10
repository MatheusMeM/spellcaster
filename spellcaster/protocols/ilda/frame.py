"""ILDA point/frame, scan optimisation and safety."""
import math
from dataclasses import dataclass, replace

LIM = 32767


def clamp(v):
    return max(-LIM, min(LIM, int(v)))


@dataclass(slots=True)
class Point:
    x: int
    y: int
    r: int = 0
    g: int = 0
    b: int = 0
    blank: bool = False

    def __post_init__(self):
        self.x, self.y = clamp(self.x), clamp(self.y)
        self.r, self.g, self.b = (max(0, min(255, int(c))) for c in (self.r, self.g, self.b))

    @property
    def lit(self):
        return not self.blank and bool(self.r or self.g or self.b)


class Frame:
    def __init__(self, points=None, name=""):
        self.points = list(points or [])
        self.name = name  # .ild frame name (8 chars)

    def __len__(self):
        return len(self.points)

    def __iter__(self):
        return iter(self.points)

    def bbox(self, lit_only=True):
        """(x0, y0, x1, y1) of the lit points; None if there are none."""
        pts = [p for p in self.points if p.lit] if lit_only else self.points
        if not pts:
            return None
        xs = [p.x for p in pts]; ys = [p.y for p in pts]
        return min(xs), min(ys), max(xs), max(ys)


def _lerp(p, q, u, blank):
    return Point(p.x + (q.x - p.x) * u, p.y + (q.y - p.y) * u, q.r, q.g, q.b, blank)


def optimize(frame, dwell=2, blank_gap=4, max_step=1200, angle=25):
    """Dwell on the vertices, blanked points on the lit<->blanked transitions,
    interpolation of steps larger than max_step (ILDA units)."""
    src = frame.points
    if not src:
        return Frame([], frame.name)
    cos_lim = math.cos(math.radians(angle))
    out = [src[0]]
    for i in range(1, len(src)):
        p, q = src[i - 1], src[i]
        # jump: repeats p blanked before leaving, q blanked before lighting up
        if p.lit and not q.lit:
            out += [replace(p, blank=True)] * blank_gap
        step = max(abs(q.x - p.x), abs(q.y - p.y))
        if step > max_step:
            n = math.ceil(step / max_step)
            out += [_lerp(p, q, k / n, not q.lit) for k in range(1, n)]
        if not p.lit and q.lit:
            out += [replace(q, blank=True)] * blank_gap
        out.append(q)
        # vertex: change of direction (or start of a segment) -> dwell
        if q.lit and i + 1 < len(src) and src[i + 1].lit:
            ax, ay = q.x - p.x, q.y - p.y
            bx, by = src[i + 1].x - q.x, src[i + 1].y - q.y
            na, nb = math.hypot(ax, ay), math.hypot(bx, by)
            if na == 0 or nb == 0 or (ax * bx + ay * by) / (na * nb) < cos_lim:
                out += [replace(q)] * dwell
    return Frame(out, frame.name)


def safety(frame, min_size=2000, max_intensity=255, zone=None):
    """Dims a figure smaller than min_size (a stationary point burns), caps intensity,
    blanks points outside zone=(x0, y0, x1, y1)."""
    bb = frame.bbox()
    gain = 1.0
    if bb:
        size = max(bb[2] - bb[0], bb[3] - bb[1])
        if size < min_size:
            gain = size / min_size  # single point -> 0
    out = []
    for p in frame.points:
        q = replace(p, r=min(p.r * gain, max_intensity), g=min(p.g * gain, max_intensity),
                    b=min(p.b * gain, max_intensity))
        if zone and not (zone[0] <= p.x <= zone[2] and zone[1] <= p.y <= zone[3]):
            q.blank = True
        out.append(q)
    return Frame(out, frame.name)
