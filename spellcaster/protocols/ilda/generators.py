"""Figure generators.
ILDA coordinates +-32767; colours (r, g, b) 0-255.
Usage: python -m spellcaster.protocols.ilda.generators output.ild [SX SY]  (regenerates the MED GRUPO laser)."""
import math
import sys

from .frame import Frame, Point

YELLOW = (255, 230, 0); WHITE = (255, 255, 255); CYAN = (0, 200, 255)


def _p(x, y, col):
    return Point(x, y, *col)


def ellipse(cx, cy, rx, ry, col, n=60, ph=0.0):
    return [_p(cx + rx * math.cos(ph + 2 * math.pi * i / n), cy + ry * math.sin(ph + 2 * math.pi * i / n), col)
            for i in range(n + 1)]


def circle(cx, cy, r, col, n=60, ph=0.0):
    return ellipse(cx, cy, r, r, col, n, ph)


def polyline(verts, col, k=15, close=False):
    """Segments between vertices with k steps each; a vertex repeated at the joints becomes a natural dwell."""
    if close:
        verts = list(verts) + [verts[0]]
    pts = []
    for (x0, y0), (x1, y1) in zip(verts, verts[1:]):
        pts += [_p(x0 + (x1 - x0) * j / k, y0 + (y1 - y0) * j / k, col) for j in range(k + 1)]
    return pts


def rect(cx, cy, w, h, col, k=15):
    """w, h = half-width and half-height."""
    return polyline([(cx - w, cy - h), (cx + w, cy - h), (cx + w, cy + h), (cx - w, cy + h)], col, k, close=True)


def blank_to(pts):
    """Blanked point at the start of the figure, for the jump.
    The target comes from here, not from the caller: passed by hand it gets it wrong when the phase (ph) turns."""
    return [Point(pts[0].x, pts[0].y, blank=True)] + pts if pts else pts


def ease(a, b, u):
    u = max(0, min(1, u)); u = u * u * (3 - 2 * u)
    return a + (b - a) * u


def medgrupo(t, sx=20000, sy=10000):
    """MED GRUPO RJ opening laser (video 25 fps, 46.8 s). sx, sy = half-width/height of the screen in ILDA units.
    0-10 countdown | 10-12.5 flash | 12.5-33.5 cities | 33.5-40 logo | 40-44 fade."""
    pts = []
    if t < 10:
        beat = t % 1.0; r = sy * (0.55 + 0.12 * math.exp(-4 * beat))
        pts += circle(0, 0, r, YELLOW, ph=t * 0.8)
        if int(t) % 2 == 0:
            pts += blank_to(rect(0, 0, sx * 0.98, sy * 0.98, CYAN))
    elif t < 12.5:
        u = (t - 10) / 2.5; s = ease(0.1, 1.6, u)
        pts += rect(0, 0, sx * s, sy * s, WHITE)
        pts += blank_to(rect(0, 0, sx * s * 0.6, sy * s * 0.6, WHITE))
    elif t < 33.5:
        u = (t - 12.5) / 21
        y = sy * math.sin(u * 2 * math.pi * 3) * 0.6
        pts += rect(sx * 0.35, y, sx * 0.5, sy * 0.18, YELLOW)
        pts += blank_to(circle(-sx * 0.62, -sy * 0.1, sy * 0.35, CYAN, ph=t * 2))
    elif t < 40:
        u = (t - 33.5) / 6.5; r = ease(sy * 1.4, sy * 0.6, u)
        pts += circle(0, 0, r, WHITE, ph=t * 3)
        pts += blank_to(circle(0, 0, r * 0.85, YELLOW, ph=-t * 3))
    elif t < 44:
        u = (t - 40) / 4; r = ease(sy * 0.6, 0, u)
        pts += circle(0, 0, r, WHITE)
    return Frame(pts)


def render(fn, fps=25, dur=46.8, **kw):
    """List of Frame calling fn(t, **kw) every 1/fps."""
    return [fn(i / fps, **kw) for i in range(int(dur * fps))]


if __name__ == "__main__":
    from . import ild
    out = sys.argv[1]
    sx = int(sys.argv[2]) if len(sys.argv) > 2 else 20000
    sy = int(sys.argv[3]) if len(sys.argv) > 3 else 10000
    frames = render(medgrupo, sx=sx, sy=sy)
    ild.write(out, frames, fmt=5, name="medgrupo", company="feitic.")
    print(out, len(frames), "frames", f"SX={sx} SY={sy}")
