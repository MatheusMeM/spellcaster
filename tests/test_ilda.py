import os
import struct
import tempfile
import unittest

from spellcaster.protocols.ilda import Frame, Point, optimize, safety, generators, ild
from spellcaster.protocols.ilda import etherdream as ed


def square(size=10000, col=(255, 0, 0)):
    return Frame(generators.rect(0, 0, size, size, col, k=4), name="square")


class TestIld(unittest.TestCase):
    def roundtrip(self, fmt, **kw):
        frames = [square(), Frame(generators.blank_to(generators.circle(0, 0, 5000, (0, 255, 0), n=8)), "circ"), Frame([])]
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "t.ild")
            ild.write(path, frames, fmt=fmt, **kw)
            back = ild.read(path)
        self.assertEqual(len(back), 3)
        self.assertEqual([f.name for f in back][:2], ["square", "circ"])
        for a, b in zip(frames[:2], back):
            self.assertEqual([(p.x, p.y, p.blank) for p in a], [(p.x, p.y, p.blank) for p in b])
        self.assertEqual(len(back[2]), 1)  # empty frame becomes 1 blanked point
        self.assertTrue(back[2].points[0].blank)
        return frames, back

    def test_fmt5_true_color(self):
        a, b = self.roundtrip(5)
        self.assertEqual((b[0].points[0].r, b[0].points[0].g, b[0].points[0].b), (255, 0, 0))
        self.assertEqual((b[1].points[1].r, b[1].points[1].g, b[1].points[1].b), (0, 255, 0))

    def test_fmt1_indexed_default_palette(self):
        _, b = self.roundtrip(1)
        self.assertEqual((b[0].points[0].r, b[0].points[0].g, b[0].points[0].b), (255, 0, 0))

    def test_fmt1_with_palette_section(self):
        _, b = self.roundtrip(1, palette=[(1, 2, 3), (255, 0, 0), (0, 255, 0)])
        self.assertEqual((b[1].points[1].r, b[1].points[1].g, b[1].points[1].b), (0, 255, 0))

    def test_fmt0_and_4(self):
        for fmt in (0, 4):
            self.roundtrip(fmt)

    def test_seed_layout_matches(self):
        # header and format-5 record identical to seed/ilda_gen.py: 32 bytes + 8 per point, X Y status B G R
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "s.ild")
            ild.write(path, [Frame([Point(1, -2, 10, 20, 30), Point(3, 4, blank=True)])], name="medgrupo", company="feitic.")
            with open(path, "rb") as f:
                raw = f.read()
        self.assertEqual(raw[:4], b"ILDA")
        self.assertEqual(raw[7], 5)
        self.assertEqual(raw[8:16], b"medgrupo")
        self.assertEqual(raw[32:48], struct.pack(">hhBBBB", 1, -2, 0, 30, 20, 10) + struct.pack(">hhBBBB", 3, 4, 0xC0, 0, 0, 0))
        self.assertEqual(len(raw), 32 + 16 + 32)  # + terminator


class TestOptimize(unittest.TestCase):
    def test_inserts_blanks_dwell_and_interpolates(self):
        f = Frame([Point(0, 0, 255, 0, 0), Point(3000, 0, 255, 0, 0), Point(3000, 3000, 255, 0, 0),
                   Point(-20000, -20000, blank=True), Point(-20000, -20000, 255, 0, 0), Point(-19000, -20000, 255, 0, 0)])
        o = optimize(f, dwell=2, blank_gap=4, max_step=1200)
        pts = o.points
        # blanked jump: 4 blanked copies of (3000,3000) before leaving + 4 blanked at the destination
        self.assertEqual(sum(1 for p in pts if p.blank and (p.x, p.y) == (3000, 3000)), 4)
        self.assertGreaterEqual(sum(1 for p in pts if p.blank and (p.x, p.y) == (-20000, -20000)), 5)
        # interpolation: no step larger than max_step
        self.assertTrue(all(max(abs(b.x - a.x), abs(b.y - a.y)) <= 1200 for a, b in zip(pts, pts[1:])))
        # dwell at the (3000,0) corner: 90 degrees -> 1 original + 2 lit repeats
        self.assertEqual(sum(1 for p in pts if p.lit and (p.x, p.y) == (3000, 0)), 3)
        self.assertEqual(optimize(Frame([])).points, [])


class TestSafety(unittest.TestCase):
    def test_small_figure_darkens(self):
        small = safety(square(500), min_size=2000)
        self.assertTrue(all(p.r == 127 for p in small.points))  # 1000/2000 * 255, truncated
        big = safety(square(5000), min_size=2000)
        self.assertTrue(all(p.r == 255 for p in big.points))
        dot = safety(Frame([Point(0, 0, 255, 255, 255)] * 5))
        self.assertTrue(all((p.r, p.g, p.b) == (0, 0, 0) for p in dot.points))

    def test_max_intensity_and_zone(self):
        f = safety(square(5000), max_intensity=100, zone=(-1000, -1000, 1000, 1000))
        self.assertTrue(all(p.r == 100 for p in f.points))
        self.assertTrue(all(p.blank for p in f.points if abs(p.x) == 5000))


class TestEtherDream(unittest.TestCase):
    def test_encode_data(self):
        raw = ed.encode_data([Point(-1, 2, 255, 0, 128), Point(5, 6, 255, 255, 255, blank=True)])
        self.assertEqual(raw[:3], b"d" + struct.pack("<H", 2))
        self.assertEqual(raw[3:21], struct.pack("<HhhHHHHHH", 0, -1, 2, 65535, 0, 32896, 65535, 0, 0))
        self.assertEqual(raw[21:39], struct.pack("<HhhHHHHHH", 0, 5, 6, 0, 0, 0, 0, 0, 0))

    def test_parse_beacon(self):
        emu = ed.Emulator()
        b = ed.parse_beacon(emu.beacon())
        self.assertEqual(b["mac"], "01:02:03:04:05:06")
        self.assertEqual(b["buffer_capacity"], 1800)
        self.assertEqual(b["status"]["playback_state"], 0)
        emu._srv.close()

    def test_loopback_with_emulator(self):
        emu = ed.Emulator(capacity=1800).start()
        try:
            dac = ed.EtherDream("127.0.0.1", emu.port, capacity=1800)
            r = dac.connect()
            self.assertEqual(r["ack"], "a")
            frames = [Frame([Point(i * 10, -i * 10, 255, 0, 0) for i in range(100)]) for _ in range(6)]
            t = dac.play(frames, pps=20000)
            t.join(5)
            self.assertFalse(t.is_alive())
            dac.close()
        finally:
            emu.stop()
        self.assertEqual(len(emu.received), 600)
        self.assertEqual(emu.received[1][1:6], (10, -10, 65535, 0, 0))
        self.assertEqual(emu.commands[:3], [b"p", b"d", b"b"])  # prepare, data, begin
        self.assertEqual(emu.commands[-1], b"s")
        self.assertTrue(all(c in (b"p", b"d", b"b", b"?", b"s") for c in emu.commands))


if __name__ == "__main__":
    unittest.main()
