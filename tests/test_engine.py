import unittest

from spellcaster.core.engine import Engine
from spellcaster.core.universe import Universe


class FakeOut:
    def __init__(self): self.sent = []
    def send(self, universe, data): self.sent.append((universe, bytes(data)))
    def close(self): pass


class TestUniverse(unittest.TestCase):
    def test_set_clamp_e_limite(self):
        u = Universe(1); u.set(1, [300, -5, 7.9]); u.set(511, [1, 2, 3, 4])
        self.assertEqual(u.data[:3], bytes([255, 0, 7]))
        self.assertEqual(u.data[510:], bytes([1, 2]))
        self.assertEqual(len(u.data), 512)


class TestEngine(unittest.TestCase):
    def test_look_para_universos(self):
        out = FakeOut(); eng = Engine([out], fps=1000)

        def look(t):
            return {1: [10, 20], 400: [int(t * 100)], (2, 5): [99]}
        eng.run(look, duration=0.0)                                    # duration 0: nenhum tick
        self.assertEqual(out.sent, [])
        eng.tick(look, 0.5)
        sent = dict(out.sent)
        self.assertEqual(sent[1][:2], bytes([10, 20])); self.assertEqual(sent[1][399], 50)
        self.assertEqual(sent[2][4], 99); self.assertEqual(len(sent[1]), 512)


if __name__ == "__main__":
    unittest.main()
