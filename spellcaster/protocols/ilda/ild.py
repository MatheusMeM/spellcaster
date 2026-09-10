"""Reading/writing .ild files (ILDA Image Data Transfer Format).
Formats: 0 indexed 3D, 1 indexed 2D, 2 palette, 4 true colour 3D, 5 true colour 2D.
32-byte big-endian header; status bit7 = last point, bit6 = blanked."""
import struct

from .frame import Frame, Point

HDR = struct.Struct(">4s3xB8s8sHHHBx")
REC = {0: struct.Struct(">hhhBB"), 1: struct.Struct(">hhBB"), 2: struct.Struct(">BBB"),
       4: struct.Struct(">hhhBBBB"), 5: struct.Struct(">hhBBBB")}
LAST, BLANK = 0x80, 0x40

# default ILDA palette (64 colours)
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
    """Reads .ild -> list of Frame. A palette section (fmt 2) swaps the palette of the indexed frames that follow."""
    with open(path, "rb") as f:
        data = f.read()
    frames, palette, pos = [], DEFAULT_PALETTE, 0
    while pos + HDR.size <= len(data):
        magic, fmt, name, _company, n, _idx, _total, _proj = HDR.unpack_from(data, pos)
        pos += HDR.size
        if magic != b"ILDA" or fmt not in REC:
            raise ValueError(f"invalid header at {pos - HDR.size}: {magic!r} fmt={fmt}")
        if n == 0:
            break  # empty final frame = end of file
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
    """Writes frames to .ild. fmt 0/1 indexed: uses palette (written first as a fmt 2 section) or the default one."""
    if fmt not in (0, 1, 4, 5):
        raise ValueError("fmt must be 0, 1, 4 or 5")
    rec, total = REC[fmt], len(frames)
    pal = palette or DEFAULT_PALETTE
    out = []
    if palette and fmt in (0, 1):
        out.append(_section(2, name, company, [REC[2].pack(*c) for c in palette], 0, total))
    for idx, fr in enumerate(frames):
        pts = fr.points or [Point(0, 0, blank=True)]  # empty frame: 1 blanked point (n=0 would mean end of file)
        recs = []
        for i, p in enumerate(pts):
            st = (LAST if i == len(pts) - 1 else 0) | (BLANK if p.blank else 0)
            z = (0,) if fmt in (0, 4) else ()
            if fmt in (0, 1):
                recs.append(rec.pack(p.x, p.y, *z, st, _index((p.r, p.g, p.b), pal)))
            else:
                recs.append(rec.pack(p.x, p.y, *z, st, p.b, p.g, p.r))
        out.append(_section(fmt, fr.name or name, company, recs, idx, total))
    out.append(_section(fmt, name, company, [], total, total))  # terminator
    with open(path, "wb") as f:
        f.write(b"".join(out))
