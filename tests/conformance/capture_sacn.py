# Validacao de conformidade: roda o binario Rust `spellcore play shows/medgrupo_r0.spell`,
# captura o sACN em 127.0.0.1 com o SacnIn do Python e compara byte a byte com medgrupo_u1.bin.
#
# O indice do frame vem da SEQUENCIA do E1.31, nao do relogio: o mesmo frame chega varias vezes
# (uma copia multicast por interface + a copia unicast em 127.0.0.1) com a mesma sequencia.
# Duplicata e descartada; salto de sequencia avanca o indice na mesma medida.
#
# Uso: C:\Python313\python.exe tests/conformance/capture_sacn.py [--secs 3] [--exe <spellcore.exe>]
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
    """SacnIn que guarda (seq, data) de cada pacote em vez de so o ultimo frame."""

    def _loop(self):
        self.got = []
        while self._run:
            try:
                pk, _ = self._sock.recvfrom(2048)
            except socket.timeout:          # timeout e' OSError: tem que vir antes
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
        print(f"FALTA o binario: {a.exe}")
        return 2

    ref = fixture(os.path.join(HERE, "medgrupo_u1.bin"))
    rec = Recorder(universes=(1,))
    rec.got = []
    time.sleep(0.3)                                   # deixa o socket entrar no grupo multicast
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
            continue                                  # copia do mesmo frame (multicast + unicast)
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

    print(f"pacotes recebidos: {len(got)} ; frames distintos comparados: {ok + bad}")
    print(f"frames iguais ao fixture: {ok} ; diferentes: {bad}")
    if first_bad:
        print(f"primeiro erro: frame {first_bad[0]} canal {first_bad[1] + 1} "
              f"recebido {first_bad[2]} esperado {first_bad[3]}")
    if err:
        print("stderr do spellcore:", err[:400])
    return 0 if ok and not bad else 1


if __name__ == "__main__":
    sys.exit(main())
