"""Generates the binary conformance fixtures of the `laser` crate from the Python package.

    C:\\Python313\\python.exe spellcore/laser/tests/gen_fixtures.py

For each case it writes `fixtures/<name>_in.ild` (input) and `fixtures/<name>_out.ild`
(the Python result) plus `fixtures/cases.json` with the operation and the parameters.
The `.ild` fmt 5 stores x, y, r, g, b and blank without loss, so it works as the fixture
format without inventing another one (the reader/writer itself is locked byte by byte by
the shows/medgrupo_laser.ild test).

Rust reads `_in.ild`, runs the same operation with the same parameters and must give
exactly the points of `_out.ild`.
"""
import json
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
sys.path.insert(0, ROOT)

from spellcaster.protocols.ilda import Frame, Point, generators, ild, optimize, safety  # noqa: E402

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures")
MEDGRUPO = os.path.join(ROOT, "shows", "medgrupo_laser.ild")


def salto():
    """Same frame as tests/test_ilda.py: 90 degree vertex, blanked jump, long step."""
    return Frame([Point(0, 0, 255, 0, 0), Point(3000, 0, 255, 0, 0), Point(3000, 3000, 255, 0, 0),
                  Point(-20000, -20000, blank=True), Point(-20000, -20000, 255, 0, 0),
                  Point(-19000, -20000, 255, 0, 0)], "lasr")


def quad(size, col=(255, 0, 0), k=4):
    return Frame(generators.rect(0, 0, size, size, col, k), "lasr")


def cases(med):
    """(name, input frame, operation, parameters)."""
    yield "opt_salto", salto(), "optimize", dict(dwell=2, blank_gap=4, max_step=1200, angle=25)
    yield "opt_quad", quad(10000), "optimize", dict(dwell=3, blank_gap=2, max_step=800, angle=40)
    yield "opt_med0", med[0], "optimize", dict(dwell=2, blank_gap=4, max_step=1200, angle=25)
    yield "opt_med300", med[300], "optimize", dict(dwell=1, blank_gap=6, max_step=2000, angle=10)
    yield "opt_med900", med[900], "optimize", dict(dwell=2, blank_gap=4, max_step=1200, angle=25)
    yield "saf_pequeno", quad(500), "safety", dict(min_size=2000, max_intensity=255, zone=None)
    yield "saf_zona", quad(5000), "safety", dict(min_size=2000, max_intensity=100,
                                                 zone=[-1000, -1000, 1000, 1000])
    yield "saf_ponto", Frame([Point(0, 0, 255, 255, 255)] * 5, "lasr"), "safety", \
        dict(min_size=2000, max_intensity=255, zone=None)
    yield "saf_med0", med[0], "safety", dict(min_size=20000, max_intensity=200,
                                             zone=[-18000, -9000, 18000, 9000])
    yield "optsaf_med600", med[600], "optimize+safety", dict(
        dwell=2, blank_gap=4, max_step=1200, angle=25,
        min_size=2000, max_intensity=180, zone=[-16000, -8000, 16000, 8000])


def apply(op, fr, p):
    if op in ("optimize", "optimize+safety"):
        fr = optimize(fr, p["dwell"], p["blank_gap"], p["max_step"], p["angle"])
    if op in ("safety", "optimize+safety"):
        z = p["zone"] and tuple(p["zone"])
        fr = safety(fr, p["min_size"], p["max_intensity"], z)
    return fr


def main():
    os.makedirs(OUT, exist_ok=True)
    med = ild.read(MEDGRUPO)
    index = []
    for name, src, op, params in cases(med):
        src = Frame(list(src.points), "lasr")
        assert src.points, name  # an empty frame does not survive the .ild
        pin = os.path.join(OUT, name + "_in.ild")
        ild.write(pin, [src], fmt=5, name="lasr", company="spell")
        # reload: Rust starts from exactly these bytes
        back = ild.read(pin)[0]
        out = apply(op, back, params)
        ild.write(os.path.join(OUT, name + "_out.ild"), [out], fmt=5, name="lasr", company="spell")
        index.append({"name": name, "op": op, "params": params,
                      "n_in": len(back), "n_out": len(out)})
        print(f"{name:16s} {op:16s} {len(back):5d} -> {len(out):5d} points")
    with open(os.path.join(OUT, "cases.json"), "w", encoding="ascii", newline="\n") as f:
        json.dump(index, f, indent=1)
        f.write("\n")
    print(len(index), "cases in", OUT)


if __name__ == "__main__":
    main()
