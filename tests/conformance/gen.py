# Generates the conformance fixtures that spellcore (Rust) has to reproduce byte for byte.
# Source of truth: the Python package spellcaster + shows/medgrupo.spell.
#
#   medgrupo_u1.bin   u32 LE nframes, then nframes * 512 bytes of universe 1 (t = n/fps)
#   sacn_packet.bin   one complete E1.31 packet: universe 1, seq 0, CID 00..0f, prio 100
#   artnet_packet.bin one complete ArtDmx: universe 1, seq 0
#   ../../shows/medgrupo_r0.spell   the same show, but with the values already baked into keyframes
#
# ponytail: the medgrupo pyfx track is pure Python and keeps state between frames (pan hysteresis
# on the movings). Rust has no pyfx in R0; gen.py bakes the result frame by frame into dmx tracks
# with a "hold" curve (exact step) ; swap for an fx track in rhai when R1 lands.
# Usage: C:\Python313\python.exe tests/conformance/gen.py
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
    """Runs the .spell timeline frame by frame and returns (fps, [bytes(512) per frame])."""
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
    """Rewrites the show, swapping the non-DMX tracks for one dmx track per channel with a hold curve."""
    sh = showfile.load(src)
    tracks = []
    for ch in range(512):
        keys, prev = [], None
        for i, b in enumerate(fr):
            v = b[ch]
            if v != prev:
                # time truncated (never rounded up) to 1 us: always lands before i/fps
                keys.append([int(i / fps * 1e6) / 1e6, v, "hold"])
                prev = v
        if len(keys) == 1 and keys[0][1] == 0:
            continue                                           # channel always 0: no track needed
        if keys and keys[0][0] != 0.0:
            keys.insert(0, [0.0, 0, "hold"])
        if keys:
            tracks.append({"type": "dmx", "universe": 1, "address": ch + 1, "keys": keys})
    sh["tracks"] = tracks
    sh["outputs"] = [o for o in sh.get("outputs", ()) if o.get("type") in ("sacn", "artnet")]
    # kept in Portuguese on purpose: this string goes into the bytes of the generated fixture
    sh["name"] = sh.get("name", "") + " (R0 baked)"
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
    print(f"medgrupo_u1.bin: {len(fr)} frames at {fps} fps")
    print(f"medgrupo_r0.spell: {len(tracks)} tracks, {nkeys} keyframes, "
          f"{os.path.getsize(baked) // 1024} KiB")

    # auto-check: the baked show reproduces the .bin on every frame
    fps2, fr2 = frames(baked)
    bad = [i for i, (a, b) in enumerate(zip(fr, fr2)) if a != b]
    assert fps2 == fps and len(fr2) == len(fr) and not bad, f"baked differs on {len(bad)} frames: {bad[:5]}"
    print("baked matches the original on every frame")


if __name__ == "__main__":
    main()
