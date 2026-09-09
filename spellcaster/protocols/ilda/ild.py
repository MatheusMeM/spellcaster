"""Leitura/escrita de arquivos .ild (ILDA Image Data Transfer Format).
Formatos: 0 3D indexado, 1 2D indexado, 2 paleta, 4 3D true color, 5 2D true color.
Header 32 bytes big-endian; status bit7 = ultimo ponto, bit6 = apagado."""
import struct

from .frame import Frame, Point

HDR = struct.Struct(">4s3xB8s8sHHHBx")
REC = {0: struct.Struct(">hhhBB"), 1: struct.Struct(">hhBB"), 2: struct.Struct(">BBB"),
       4: struct.Struct(">hhhBBBB"), 5: struct.Struct(">hhBBBB")}
LAST, BLANK = 0x80, 0x40

# paleta padrao ILDA (64 cores)
DEFAULT_PALETTE = [
    (255, 0, 0), (255, 16, 0), (255, 32, 0), (255, 48, 0), (255, 64, 0), (255, 80, 0), (255, 96, 0), (255, 112, 0),
    (255, 128, 0), (255, 144, 0), (255, 160, 0), (255, 176, 0), (255, 192, 0), (255, 208, 0), (255, 224, 0), (255, 240, 0),
    (255, 255, 0), (224, 255, 0), (192, 255, 0), (160, 255, 0), (128, 255, 0), (96, 255, 0), (64, 255, 0), (32, 255, 0),
    (0, 255, 0), (0, 255, 36), (0, 255, 73), (0, 255, 109), (0, 255, 146), (0, 255, 182), (0, 255, 219), (0, 255, 255),
    (0, 227, 255), (0, 198, 255), (0, 170, 255), (0, 142, 255), (0, 113, 255), (0, 85, 255), (0, 56, 255), (0, 28, 255),
    (0, 0, 255), (32, 0, 255), (64, 0, 255), (96, 0, 255), (128, 0, 255), (160, 0, 255), (192, 0, 255), (224, 0, 255),
    (255, 0, 255), (255, 32, 255), (255, 64, 255), (255, 96, 255), (255, 128, 255), (255, 160, 255), (255, 192, 255), (255, 224, 255),
    (255, 255, 255), (255, 224, 224), (255, 192, 192), (255, 160, 160), (255, 128, 128), (255, 96, 96), (255, 64, 64), (255, 32, 32),
]


def _index(col, palette):
    return min(range(len(palette)), key=lambda i: sum((a - b) ** 2 for a, b in zip(col, palette[i])))


def read(path):
    """Le .ild -> lista de Frame. Secao de paleta (fmt 2) troca a paleta dos frames indexados seguintes."""
    with open(path, "rb") as f:
        data = f.read()
    frames, palette, pos = [], DEFAULT_PALETTE, 0
    while pos + HDR.size <= len(data):
        magic, fmt, name, _company, n, _idx, _total, _proj = HDR.unpack_from(data, pos)
        pos += HDR.size
        if magic != b"ILDA" or fmt not in REC:
            raise ValueError(f"header invalido em {pos - HDR.size}: {magic!r} fmt={fmt}")
        if n == 0:
            break  # frame final vazio = fim
        rec = REC[fmt]
        recs = [rec.unpack_from(data, pos + i * rec.size) for i in range(n)]
        pos += n * rec.size
        if fmt == 2:
            palette = [tuple(r) for r in recs]
            continue
        pts = []
        for r in recs:
            if fmt in (0, 1):
                *xyz, st, ci = r
                col = palette[ci] if ci < len(palette) else (255, 255, 255)
            else:
                *xyz, st, b, g, rr = r
                col = (rr, g, b)
            pts.append(Point(xyz[0], xyz[1], *col, blank=bool(st & BLANK)))
        frames.append(Frame(pts, name.rstrip(b"\0 ").decode("ascii", "replace")))
    return frames


def _section(fmt, name, company, recs, idx, total):
    hdr = HDR.pack(b"ILDA", fmt, name.encode("ascii", "replace")[:8].ljust(8),
                   company.encode("ascii", "replace")[:8].ljust(8), len(recs), idx, total, 0)
    return hdr + b"".join(recs)


def write(path, frames, fmt=5, name="", company="spell", palette=None):
    """Grava frames em .ild. fmt 0/1 indexado: usa palette (gravada antes como secao fmt 2) ou a padrao."""
    if fmt not in (0, 1, 4, 5):
        raise ValueError("fmt deve ser 0, 1, 4 ou 5")
    rec, total = REC[fmt], len(frames)
    pal = palette or DEFAULT_PALETTE
    out = []
    if palette and fmt in (0, 1):
        out.append(_section(2, name, company, [REC[2].pack(*c) for c in palette], 0, total))
    for idx, fr in enumerate(frames):
        pts = fr.points or [Point(0, 0, blank=True)]  # frame vazio: 1 ponto apagado (n=0 seria fim de arquivo)
        recs = []
        for i, p in enumerate(pts):
            st = (LAST if i == len(pts) - 1 else 0) | (BLANK if p.blank else 0)
            z = (0,) if fmt in (0, 4) else ()
            if fmt in (0, 1):
                recs.append(rec.pack(p.x, p.y, *z, st, _index((p.r, p.g, p.b), pal)))
            else:
                recs.append(rec.pack(p.x, p.y, *z, st, p.b, p.g, p.r))
        out.append(_section(fmt, fr.name or name, company, recs, idx, total))
    out.append(_section(fmt, name, company, [], total, total))  # terminador
    with open(path, "wb") as f:
        f.write(b"".join(out))
