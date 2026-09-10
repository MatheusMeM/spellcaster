import time, unittest

from spellcaster.core.clock import Clock


class TestClock(unittest.TestCase):
    def test_30_ticks_at_60fps(self):
        c = Clock(fps=60); ticks = []

        def fn(t):
            ticks.append(t)
            if len(ticks) == 30:
                c.stop()
        t0 = time.perf_counter(); c.run(fn); dt = time.perf_counter() - t0
        self.assertEqual(len(ticks), 30)
        self.assertAlmostEqual(dt, 0.5, delta=0.1)          # ±20 %
        self.assertAlmostEqual(ticks[-1], 29 / 60, delta=0.05)

    def test_transport(self):
        c = Clock(); c.locate(5.0); c.play(); time.sleep(0.05); c.pause()
        self.assertAlmostEqual(c.time, 5.05, delta=0.03)
        t = c.time; time.sleep(0.02); self.assertEqual(c.time, t)   # paused freezes
        c.stop(); self.assertEqual(c.time, 0.0)

    def test_duration(self):
        c = Clock(fps=100); n = []
        c.run(lambda t: n.append(t), duration=0.1)
        self.assertTrue(8 <= len(n) <= 12, n)


if __name__ == "__main__":
    unittest.main()
