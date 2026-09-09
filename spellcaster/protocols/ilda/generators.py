"""Geradores de figura (migrado de seed/ilda_gen.py).
Coordenadas ILDA +-32767; cores (r, g, b) 0-255.
Uso: python -m spellcaster.protocols.ilda.generators saida.ild [SX SY]  (regenera o laser MED GRUPO)."""
import math
import sys

from .frame import Frame, Point

AMARELO = (255, 230, 0); BRANCO = (255, 255, 255); CIANO = (0, 200, 255)


def _p(x, y, col):
    return Point(x, y, *col)


def ellipse(cx, cy, rx, ry, col, n=60, ph=0.0):
    return [_p(cx + rx * math.cos(ph + 2 * math.pi * i / n), cy + ry * math.sin(ph + 2 * math.pi * i / n), col)
            for i in range(n + 1)]


def circle(cx, cy, r, col, n=60, ph=0.0):
    return ellipse(cx, cy, r, r, col, n, ph)


def polyline(verts, col, k=15, close=False):
    """Segmentos entre vertices com k passos cada; vertice repetido nas juncoes vira dwell natural."""
    if close:
        verts = list(verts) + [verts[0]]
    pts = []
    for (x0, y0), (x1, y1) in zip(verts, verts[1:]):
        pts += [_p(x0 + (x1 - x0) * j / k, y0 + (y1 - y0) * j / k, col) for j in range(k + 1)]
    return pts


def rect(cx, cy, w, h, col, k=15):
    """w, h = meia-largura e meia-altura."""
    return polyline([(cx - w, cy - h), (cx + w, cy - h), (cx + w, cy + h), (cx - w, cy + h)], col, k, close=True)


def line(x0, y0, x1, y1, col, k=15):
    return polyline([(x0, y0), (x1, y1)], col, k)


def blank_to(pts):
    """Ponto apagado no inicio da figura para o salto.
    Corrigido: o seed passava o alvo a mao e errava quando a fase (ph) girava."""
    return [Point(pts[0].x, pts[0].y, blank=True)] + pts if pts else pts


def ease(a, b, u):
    u = max(0, min(1, u)); u = u * u * (3 - 2 * u)
    return a + (b - a) * u


def medgrupo(t, sx=20000, sy=10000):
    """Laser da abertura MED GRUPO RJ (video 25 fps, 46,8 s). sx, sy = meia-largura/altura da tela em ILDA.
    0-10 contagem | 10-12,5 flash | 12,5-33,5 cidades | 33,5-40 logo | 40-44 fade."""
    pts = []
    if t < 10:
        beat = t % 1.0; r = sy * (0.55 + 0.12 * math.exp(-4 * beat))
        pts += circle(0, 0, r, AMARELO, ph=t * 0.8)
        if int(t) % 2 == 0:
            pts += blank_to(rect(0, 0, sx * 0.98, sy * 0.98, CIANO))
    elif t < 12.5:
        u = (t - 10) / 2.5; s = ease(0.1, 1.6, u)
        pts += rect(0, 0, sx * s, sy * s, BRANCO)
        pts += blank_to(rect(0, 0, sx * s * 0.6, sy * s * 0.6, BRANCO))
    elif t < 33.5:
        u = (t - 12.5) / 21
        y = sy * math.sin(u * 2 * math.pi * 3) * 0.6
        pts += rect(sx * 0.35, y, sx * 0.5, sy * 0.18, AMARELO)
        pts += blank_to(circle(-sx * 0.62, -sy * 0.1, sy * 0.35, CIANO, ph=t * 2))
    elif t < 40:
        u = (t - 33.5) / 6.5; r = ease(sy * 1.4, sy * 0.6, u)
        pts += circle(0, 0, r, BRANCO, ph=t * 3)
        pts += blank_to(circle(0, 0, r * 0.85, AMARELO, ph=-t * 3))
    elif t < 44:
        u = (t - 40) / 4; r = ease(sy * 0.6, 0, u)
        pts += circle(0, 0, r, BRANCO)
    return Frame(pts)


def render(fn, fps=25, dur=46.8, **kw):
    """Lista de Frame chamando fn(t, **kw) a cada 1/fps."""
    return [fn(i / fps, **kw) for i in range(int(dur * fps))]


if __name__ == "__main__":
    from . import ild
    out = sys.argv[1]
    sx = int(sys.argv[2]) if len(sys.argv) > 2 else 20000
    sy = int(sys.argv[3]) if len(sys.argv) > 3 else 10000
    frames = render(medgrupo, sx=sx, sy=sy)
    ild.write(out, frames, fmt=5, name="medgrupo", company="feitic.")
    print(out, len(frames), "frames", f"SX={sx} SY={sy}")
