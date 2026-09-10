# Conformance check: runs the Rust binary `spellcore play shows/medgrupo_r0.spell`, captures sACN
# on 127.0.0.1 with the Python SacnIn and compares it byte for byte with medgrupo_u1.bin.
#
# The frame index comes from the E1.31 SEQUENCE, not from the clock: the same frame arrives several
# times (one multicast copy per interface + the unicast copy on 127.0.0.1) with the same sequence.
# A duplicate is dropped; a sequence jump advances the index by the same amount.
#
# Usage: C:\Python313\python.exe tests/conformance/capture_sacn.py [--secs 3] [--exe <spellcore.exe>]
import argparse
import os
import socket
import struct
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, ROOT)

from spellcaster.protocols import sacn  # noqa: E402

DEFAULT_EXE = os.path.join(os.environ.get("TEMP", ""), "spellcore_target", "release", "spellcore.exe")


class Recorder(sacn.SacnIn):
    """SacnIn that keeps (seq, data) of every packet instead of only the last frame."""

    def _loop(self):
        self.got = []
        while self._run:
            try:
                pk, _ = self._sock.recvfrom(2048)
            except socket.timeout:          # timeout is an OSError: it has to come first
                continue
            except OSError:
                break
            p = sacn.parse(pk)
            if p and p["kind"] == "data" and p["universe"] == 1:
                self.got.append((p["seq"], p["data"]))


def fixture(path):
    with open(path, "rb") as f:
        n = struct.unpack("<I", f.read(4))[0]
        return [f.read(512) for _ in range(n)]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--secs", type=float, default=3.0)
    ap.add_argument("--exe", default=DEFAULT_EXE)
    ap.add_argument("--show", default=os.path.join(ROOT, "shows", "medgrupo_r0.spell"))
    a = ap.parse_args()
    if not os.path.exists(a.exe):
        print(f"MISSING binary: {a.exe}")
        return 2

    ref = fixture(os.path.join(HERE, "medgrupo_u1.bin"))
    rec = Recorder(universes=(1,))
    rec.got = []
    time.sleep(0.3)                                   # let the socket join the multicast group
    proc = subprocess.Popen([a.exe, "play", a.show], cwd=ROOT,
                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    time.sleep(a.secs)
    proc.terminate()
    try:
        proc.wait(3)
    except subprocess.TimeoutExpired:
        proc.kill()
    time.sleep(0.2)
    rec.close()
    err = proc.stderr.read().decode("utf-8", "replace").strip()

    got, idx, last = rec.got, 0, None
    ok = bad = 0
    first_bad = None
    for seq, data in got:
        if last is None:
            idx = 0
        elif seq == last:
            continue                                  # copy of the same frame (multicast + unicast)
        else:
            idx += (seq - last) % 256
        last = seq
        if idx >= len(ref):
            break
        if len(data) == 512 and data == ref[idx]:
            ok += 1
        else:
            bad += 1
            if first_bad is None:
                d = next((i for i in range(min(len(data), 512)) if data[i:i + 1] != ref[idx][i:i + 1]), -1)
                first_bad = (idx, d, data[d] if 0 <= d < len(data) else None, ref[idx][d] if d >= 0 else None)

    print(f"packets received: {len(got)} ; distinct frames compared: {ok + bad}")
    print(f"frames equal to the fixture: {ok} ; different: {bad}")
    if first_bad:
        print(f"first mismatch: frame {first_bad[0]} channel {first_bad[1] + 1} "
              f"got {first_bad[2]} expected {first_bad[3]}")
    if err:
        print("spellcore stderr:", err[:400])
    return 0 if ok and not bad else 1


if __name__ == "__main__":
    sys.exit(main())
