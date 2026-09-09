# shows/medgrupo.py reproduz look(t) do seed (mesma sequência de frames => mesma histerese de pan).
import pathlib, unittest

from spellcaster.cli import load_show

ROOT = pathlib.Path(__file__).resolve().parents[1]


class TestShow(unittest.TestCase):
    def test_frames_iguais_ao_seed(self):
        seed = {}
        exec(ROOT.joinpath("seed", "show_medgrupo.py").read_text(encoding="utf-8").split("# ---- sACN ----")[0], seed)
        show = load_show(str(ROOT / "shows" / "medgrupo.py"))
        self.assertEqual(show.DUR, seed["DUR"] + show.PRE)
        ts = [i / 30 for i in range(0, int(seed["DUR"] * 30), 7)]
        for t in ts:
            a, b = seed["look"](t), show.look(t + show.PRE)
            self.assertEqual(b.pop(show.VIDEO), [10 if t < seed["VIDEO_END"] else 28, 0])
            self.assertEqual(a, b, t)


if __name__ == "__main__":
    unittest.main()
