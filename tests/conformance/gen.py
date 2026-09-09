# Gera os fixtures de conformidade que o spellcore (Rust) tem que reproduzir byte a byte.
# Fonte da verdade: o pacote Python spellcaster + shows/medgrupo.spell.
#
#   medgrupo_u1.bin   u32 LE nframes, depois nframes * 512 bytes do universo 1 (t = n/fps)
#   sacn_packet.bin   um pacote E1.31 completo: universo 1, seq 0, CID 00..0f, prio 100
#   artnet_packet.bin um ArtDmx completo: universo 1, seq 0
#   ../../shows/medgrupo_r0.spell   o mesmo show, mas com os valores ja assados em keyframes
#
# ponytail: o track pyfx do medgrupo e Python puro e usa estado entre frames (histerese de pan
# nos movings). Em Rust nao existe pyfx na R0; o gen.py assa o resultado frame a frame em tracks
# dmx com curva "hold" (degrau exato) ; trocar por track fx em rhai quando a R1 entrar.
# Uso: C:\Python313\python.exe tests/conformance/gen.py
import json
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, ROOT)

from spellcaster import show as showfile                       # noqa: E402
from spellcaster.core.universe import Universes                # noqa: E402
from spellcaster.protocols import artnet, sacn                 # noqa: E402
from spellcaster.timeline.model import Timeline                # noqa: E402

CID = bytes(range(16))                                         # 00 01 02 ... 0f
DATA = bytes(range(256)) * 2                                   # 512 bytes: [0..255, 0..255]


def frames(spell_path):
    """Roda a timeline do .spell frame a frame e devolve (fps, [bytes(512) por frame])."""
    sh = showfile.load(spell_path)
    tl = Timeline(sh, sh["_dir"])
    fps = tl.fps
    n = int(round(float(sh["duration"]) * fps))
    uni = Universes()
    out = []
    for i in range(n):
        tl.apply(uni, i / fps)
        out.append(bytes(uni.get_or_create(1).data))
    return fps, out


def write_bin(path, fr):
    with open(path, "wb") as f:
        f.write(struct.pack("<I", len(fr)))
        for b in fr:
            f.write(b)
    return path


def bake(src, dst, fps, fr):
    """Reescreve o show trocando os tracks nao-DMX por um track dmx por canal com curva hold."""
    sh = showfile.load(src)
    tracks = []
    for ch in range(512):
        keys, prev = [], None
        for i, b in enumerate(fr):
            v = b[ch]
            if v != prev:
                # tempo truncado (nunca arredondado para cima) em 1 us: cai sempre antes de i/fps
                keys.append([int(i / fps * 1e6) / 1e6, v, "hold"])
                prev = v
        if len(keys) == 1 and keys[0][1] == 0:
            continue                                           # canal sempre 0: nao precisa de track
        if keys and keys[0][0] != 0.0:
            keys.insert(0, [0.0, 0, "hold"])
        if keys:
            tracks.append({"type": "dmx", "universe": 1, "address": ch + 1, "keys": keys})
    sh["tracks"] = tracks
    sh["outputs"] = [o for o in sh.get("outputs", ()) if o.get("type") in ("sacn", "artnet")]
    sh["name"] = sh.get("name", "") + " (R0 assado)"
    out = {k: v for k, v in sh.items() if not k.startswith("_")}
    with open(dst, "w", encoding="utf-8", newline="\n") as f:
        json.dump(out, f, separators=(",", ":"), ensure_ascii=False)
        f.write("\n")
    return tracks


def main():
    src = os.path.join(ROOT, "shows", "medgrupo.spell")
    fps, fr = frames(src)
    write_bin(os.path.join(HERE, "medgrupo_u1.bin"), fr)
    with open(os.path.join(HERE, "sacn_packet.bin"), "wb") as f:
        f.write(sacn.packet(1, DATA, CID, 0, "Spellcaster", 100))
    with open(os.path.join(HERE, "artnet_packet.bin"), "wb") as f:
        f.write(artnet.artdmx(1, DATA, 0))
    baked = os.path.join(ROOT, "shows", "medgrupo_r0.spell")
    tracks = bake(src, baked, fps, fr)
    nkeys = sum(len(t["keys"]) for t in tracks)
    print(f"medgrupo_u1.bin: {len(fr)} frames a {fps} fps")
    print(f"medgrupo_r0.spell: {len(tracks)} tracks, {nkeys} keyframes, "
          f"{os.path.getsize(baked) // 1024} KiB")

    # auto-check: o show assado reproduz o .bin em todos os frames
    fps2, fr2 = frames(baked)
    bad = [i for i, (a, b) in enumerate(zip(fr, fr2)) if a != b]
    assert fps2 == fps and len(fr2) == len(fr) and not bad, f"assado difere em {len(bad)} frames: {bad[:5]}"
    print("assado confere com o original em todos os frames")


if __name__ == "__main__":
    main()
