# Monitor de texto: barras dos universos, render e o fallback com \r (Windows, sem curses).
import io
import unittest
from contextlib import redirect_stdout

from spellcaster import tui

STATE = {"state": "play", "t": 12.5, "laser": "25000 pps dac 10.0.0.9",
         "universes": {1: bytearray(512), 2: bytearray(b"\xff" * 512)},
         "sources": ["sacn Consola", "artnet Node \xe9"], "log": ["00:00:01 transporte play"]}


class TestTui(unittest.TestCase):
    def test_bar(self):
        self.assertEqual(tui.bar(bytearray(512), 8), " " * 8)
        self.assertEqual(tui.bar(bytearray(b"\xff" * 512), 8), "@" * 8)
        self.assertEqual(len(tui.bar(bytearray(512), 48)), 48)
        meio = bytearray(512)
        meio[0:64] = b"\xff" * 64                       # so o primeiro bloco aceso
        self.assertEqual(tui.bar(meio, 8), "@" + " " * 7)

    def test_render(self):
        ln = tui.render(STATE, 16)
        self.assertIn("play", ln[0])
        self.assertIn("12.50", ln[0])
        self.assertTrue(ln[1].startswith("U1  |") and ln[1].endswith("max   0"))
        self.assertIn("@" * 16, ln[2])                  # universo 2 cheio
        self.assertIn("25000 pps", "\n".join(ln))
        self.assertIn("transporte play", ln[-1])
        for x in ln:
            x.encode("ascii")                           # console cp1252: nada fora de ASCII

    def test_draw_uma_linha(self):
        buf = io.StringIO()
        with redirect_stdout(buf):
            tui.draw(STATE, 100)
        out = buf.getvalue()
        self.assertEqual(len(out), 101)                 # 100 colunas + \r
        self.assertTrue(out.endswith("\r"))
        self.assertNotIn("\n", out)
        self.assertIn("play", out)
        out.encode("ascii")

    def test_snapshot_sem_player(self):
        st = tui.snapshot(["x"])
        self.assertEqual((st["state"], st["universes"], st["log"]), ("--", {}, ["x"]))
        tui.render(st)


if __name__ == "__main__":
    unittest.main()
