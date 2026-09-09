"""Gera as fixtures binarias de conformidade do crate `laser` a partir do pacote Python.

    C:\\Python313\\python.exe spellcore/laser/tests/gen_fixtures.py

Para cada caso grava `fixtures/<nome>_in.ild` (entrada) e `fixtures/<nome>_out.ild`
(resultado do Python) mais `fixtures/cases.json` com a operacao e os parametros.
O `.ild` fmt 5 guarda x, y, r, g, b e blank sem perda, entao serve de formato de fixture
sem precisar inventar outro (o proprio leitor/escritor esta preso byte a byte pelo
teste do shows/medgrupo_laser.ild).

O Rust le `_in.ild`, roda a mesma operacao com os mesmos parametros e tem que dar
exatamente os pontos de `_out.ild`.
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
    """Mesmo frame do tests/test_ilda.py: vertice de 90 graus, salto apagado, passo longo."""
    return Frame([Point(0, 0, 255, 0, 0), Point(3000, 0, 255, 0, 0), Point(3000, 3000, 255, 0, 0),
                  Point(-20000, -20000, blank=True), Point(-20000, -20000, 255, 0, 0),
                  Point(-19000, -20000, 255, 0, 0)], "lasr")


def quad(size, col=(255, 0, 0), k=4):
    return Frame(generators.rect(0, 0, size, size, col, k), "lasr")


def cases(med):
    """(nome, frame de entrada, operacao, parametros)."""
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
        assert src.points, name  # frame vazio nao sobrevive ao .ild
        pin = os.path.join(OUT, name + "_in.ild")
        ild.write(pin, [src], fmt=5, name="lasr", company="spell")
        # recarrega: o Rust parte exatamente destes bytes
        back = ild.read(pin)[0]
        out = apply(op, back, params)
        ild.write(os.path.join(OUT, name + "_out.ild"), [out], fmt=5, name="lasr", company="spell")
        index.append({"name": name, "op": op, "params": params,
                      "n_in": len(back), "n_out": len(out)})
        print(f"{name:16s} {op:16s} {len(back):5d} -> {len(out):5d} pontos")
    with open(os.path.join(OUT, "cases.json"), "w", encoding="ascii", newline="\n") as f:
        json.dump(index, f, indent=1)
        f.write("\n")
    print(len(index), "casos em", OUT)


if __name__ == "__main__":
    main()
