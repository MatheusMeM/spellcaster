# Gera animação ILDA (formato 5, 2D true color) sincronizada ao vídeo abertura_medgrupo.mp4 (25 fps, 46,8 s).
# Coordenadas ILDA ±32767; a tela ocupa o quadro [-SX,SX]x[-SY,SY] (calibrar no Capture).
# Uso: python ilda_gen.py saida.ild [SX SY]
import math, struct, sys

FPS = 25; DUR = 46.8; N = int(DUR * FPS)
SX = int(sys.argv[2]) if len(sys.argv) > 2 else 20000   # meia-largura da tela em unidades ILDA
SY = int(sys.argv[3]) if len(sys.argv) > 3 else 10000   # meia-altura (tela 2:1)
AMARELO = (255, 230, 0); BRANCO = (255, 255, 255); CIANO = (0, 200, 255)

def clamp(v): return max(-32767, min(32767, int(v)))
def circle(cx, cy, r, col, n=60, ph=0.0):
    return [(cx + r * math.cos(ph + 2 * math.pi * i / n), cy + r * math.sin(ph + 2 * math.pi * i / n), col) for i in range(n + 1)]
def rect(cx, cy, w, h, col, k=15):
    c = [(cx - w, cy - h), (cx + w, cy - h), (cx + w, cy + h), (cx - w, cy + h), (cx - w, cy - h)]
    pts = []
    for (x0, y0), (x1, y1) in zip(c, c[1:]):
        pts += [(x0 + (x1 - x0) * j / k, y0 + (y1 - y0) * j / k, col) for j in range(k + 1)]
    return pts
def blank_to(pts, x, y):  # ponto apagado para pular entre figuras
    return [(x, y, None)] + pts
def ease(a, b, u): u = max(0, min(1, u)); u = u * u * (3 - 2 * u); return a + (b - a) * u

def frame_at(t):
    """Conteúdo por fase do vídeo. 0-10 contagem | 10-12,5 flash | 12,5-33,5 cidades | 33,5-40 logo | 40+ fade."""
    pts = []
    if t < 10:                                             # contagem: círculo pulsando ao redor do número, batida 1 Hz
        beat = (t % 1.0); r = SY * (0.55 + 0.12 * math.exp(-4 * beat))
        pts += circle(0, 0, r, AMARELO, ph=t * 0.8)
        pts += blank_to(rect(0, 0, SX * 0.98, SY * 0.98, CIANO), SX * 0.98, -SY * 0.98) if int(t) % 2 == 0 else []
    elif t < 12.5:                                         # flash: quadrado explode até fora da tela
        u = (t - 10) / 2.5; s = ease(0.1, 1.6, u)
        pts += rect(0, 0, SX * s, SY * s, BRANCO)
        pts += blank_to(rect(0, 0, SX * s * 0.6, SY * s * 0.6, BRANCO), -SX * s * 0.6, -SY * s * 0.6)
    elif t < 33.5:                                         # cidades: retângulo varre a lista (coluna direita) + círculo no logo (esquerda)
        u = (t - 12.5) / 21
        y = SY * math.sin(u * 2 * math.pi * 3) * 0.6
        pts += rect(SX * 0.35, y, SX * 0.5, SY * 0.18, AMARELO)
        pts += blank_to(circle(-SX * 0.62, -SY * 0.1, SY * 0.35, CIANO, ph=t * 2), -SX * 0.62 + SY * 0.35, -SY * 0.1)
    elif t < 40:                                           # logo: círculo fecha no logo central e gira
        u = (t - 33.5) / 6.5; r = ease(SY * 1.4, SY * 0.6, u)
        pts += circle(0, 0, r, BRANCO, ph=t * 3)
        pts += blank_to(circle(0, 0, r * 0.85, AMARELO, ph=-t * 3), r * 0.85, 0)
    elif t < 44:                                           # fade: círculo encolhe até sumir
        u = (t - 40) / 4; r = ease(SY * 0.6, 0, u)
        pts += circle(0, 0, r, BRANCO)
    return pts

def frame_bytes(pts, idx, total):
    if not pts: pts = [(0, 0, None)]
    recs = b""
    for i, (x, y, col) in enumerate(pts):
        status = (0x80 if i == len(pts) - 1 else 0) | (0x40 if col is None else 0)
        r, g, b = col or (0, 0, 0)
        recs += struct.pack(">hhBBBB", clamp(x), clamp(y), status, b, g, r)   # formato 5: X Y status B G R
    hdr = b"ILDA" + b"\0\0\0" + bytes([5]) + b"medgrupo".ljust(8) + b"feitic. ".ljust(8) \
          + struct.pack(">HHHBB", len(pts), idx, total, 0, 0)
    return hdr + recs

out = sys.argv[1]
with open(out, "wb") as f:
    for i in range(N): f.write(frame_bytes(frame_at(i / FPS), i, N))
    f.write(b"ILDA" + b"\0\0\0" + bytes([5]) + b"".ljust(16) + struct.pack(">HHHBB", 0, N, N, 0, 0))  # frame final vazio
print(out, N, "frames", f"SX={SX} SY={SY}")
