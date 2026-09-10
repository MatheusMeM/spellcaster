# Player: byte-for-byte equivalence with Engine.run(look), local and OSC transport, loop,
# laser synced on the Ether Dream emulator and performance at 60 fps with 8 universes + OSC.
import pathlib
import socket
import sys
import time
import unittest

from spellcaster import show as showfile
from spellcaster.cli import load_show
from spellcaster.core.engine import Engine
from spellcaster.core.universe import Universes
from spellcaster.player.player import Player
from spellcaster.protocols import sacn
from spellcaster.protocols.ilda.etherdream import Emulator
from spellcaster.protocols.ilda.frame import Frame, Point, safety
from spellcaster.protocols.osc import OscIn, OscOut
from spellcaster.timeline.model import Timeline

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPELL = str(ROOT / "shows" / "medgrupo.spell")


class Cap:
    """Test output: keeps the bytes of every universe (send(universe, data) / close() contract)."""

    def __init__(self):
        self.frames = []

    def send(self, universe, data):
        self.frames.append((universe, data))

    def close(self):
        pass


def free_port():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def wait_until(fn, timeout=3.0):
    end = time.perf_counter() + timeout
    while time.perf_counter() < end:
        if fn():
            return True
        time.sleep(0.01)
    return False


class TestPyfx(unittest.TestCase):
    def test_medgrupo_spell_equals_engine(self):
        """Acceptance 1: the .spell pyfx track gives the same universe 1 bytes as Engine.run(look)."""
        sh = showfile.load(SPELL)
        ts = [i / 30 for i in range(0, int(sh["duration"] * 30), 13)]
        self.assertGreaterEqual(len(ts), 30)
        tl = Timeline(sh, sh["_dir"])
        uni = Universes()
        a = []
        for t in ts:
            tl.apply(uni, t)
            a.append(bytes(uni.get_or_create(1).data))
        cap = Cap()
        eng = Engine([cap])
        mod = load_show(str(ROOT / "shows" / "medgrupo.py"))
        for t in ts:
            eng.tick(mod.look, t)
        b = [d for u, d in cap.frames if u == 1]
        self.assertEqual(len(a), len(b))
        self.assertEqual(a, b)


class TestTransport(unittest.TestCase):
    def test_play_pause_locate_play_and_osc(self):
        """Acceptance 4: local and remote transport over OSC on loopback."""
        port = free_port()
        sh = {"fps": 60, "duration": 1.0, "transport": {"osc_port": port}, "outputs": [],
              "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, [0]], [1.0, [255]]]}]}
        cap = Cap()
        p = Player(sh, outputs=[cap]).start()
        rc = OscOut("127.0.0.1", port)
        try:
            rc.send("/spellcaster/play")
            self.assertTrue(wait_until(lambda: p.clock.state == "play"))
            time.sleep(0.1)
            rc.send("/spellcaster/pause")
            self.assertTrue(wait_until(lambda: p.clock.state == "pause"))
            t = p.clock.time
            self.assertGreater(t, 0.02)
            time.sleep(0.1)
            self.assertEqual(p.clock.time, t)                    # paused freezes
            rc.send("/spellcaster/locate", 0.5)
            self.assertTrue(wait_until(lambda: abs(p.clock.time - 0.5) < 1e-6))
            n = len(cap.frames)
            rc.send("/spellcaster/play")
            self.assertTrue(wait_until(lambda: p.clock.state == "play"))
            self.assertTrue(wait_until(lambda: len(cap.frames) > n + 5))
            self.assertGreater(cap.frames[-1][1][0], 120)        # after the locate the value is above half
            self.assertTrue(p.wait(3))                           # duration reached -> stop
            self.assertEqual(p.clock.state, "stop")
        finally:
            rc.close()
            p.close()

    def test_loop(self):
        sh = {"fps": 60, "duration": 0.2, "outputs": [],
              "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, [0]], [0.2, [255]]]}]}
        cap = Cap()
        p = Player(sh, loop=True, outputs=[cap]).start()
        try:
            p.play()
            time.sleep(0.7)
            self.assertEqual(p.clock.state, "play")              # with loop it does not stop at the end
            self.assertLess(p.clock.time, 0.25)
            self.assertGreater(len(cap.frames), 30)
        finally:
            p.close()

    def test_sacn_output_on_loopback(self):
        rx = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        rx.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            rx.bind(("127.0.0.1", sacn.PORT))
        except OSError as e:
            rx.close()
            self.skipTest(f"port 5568 busy: {e}")
        rx.settimeout(2.0)
        sh = {"fps": 30, "duration": 0.5,
              "outputs": [{"type": "sacn", "universes": [1], "interfaces": ["127.0.0.1"]}],
              "tracks": [{"type": "dmx", "universe": 1, "address": 10, "keys": [[0, [7, 8, 9]]]}]}
        p = Player(sh).start()
        try:
            p.play()
            pk, _ = rx.recvfrom(2048)
        finally:
            p.close()
            rx.close()
        self.assertEqual(tuple(sacn.parse(pk)["data"][9:12]), (7, 8, 9))

    def test_cue_fired_by_the_timeline(self):
        """Acceptance 3 inside the player: the cue track fires GO and the fade goes out on the universes."""
        sh = {"fps": 60, "duration": 1.2, "outputs": [],
              "tracks": [{"type": "cue", "keys": [[0.05, "GO"]]}],
              "cues": [{"name": "q1", "fade": 0.5, "values": {"1/10": [255]}}]}
        cap = Cap()
        p = Player(sh, outputs=[cap]).start()
        try:
            p.play()
            self.assertTrue(p.wait(3))
            vals = [d[9] for u, d in cap.frames]
            self.assertEqual(vals[0], 0)
            self.assertEqual(vals[-1], 255)
            self.assertTrue(any(60 < v < 200 for v in vals), "fade was not sampled")
        finally:
            p.close()


class TestLaser(unittest.TestCase):
    def test_frame_150_at_t5_on_the_emulator(self):
        """Acceptance 5: t = 5 s at 30 fps = frame 150 of the .ild, with safety() before it goes out."""
        em = Emulator().start()
        sh = showfile.load(SPELL)
        sh["outputs"] = [{"type": "laser", "dac": "127.0.0.1", "port": em.port, "pps": 25000,
                          "safety": {"min_size": 2000, "max_intensity": 255}}]
        sh["tracks"] = [t for t in sh["tracks"] if t["type"] == "laser"]
        p = Player(sh, outputs=[])
        try:
            self.assertEqual(len(p.clip), 1404)
            exp = p.laser_frame(5.0)
            ref = safety(p.clip[150], min_size=2000, max_intensity=255)
            self.assertEqual([(q.x, q.y, q.r) for q in exp], [(q.x, q.y, q.r) for q in ref])
            p.start()
            p.locate(5.0)
            p.pause()                                            # time frozen at 5 s
            self.assertTrue(wait_until(lambda: len(em.received) >= len(exp)))
        finally:
            p.close()
            em.stop()
        got = em.received[:len(exp)]
        self.assertEqual([(r[1], r[2]) for r in got], [(q.x, q.y) for q in exp])
        self.assertEqual([r[3] // 257 for r in got], [(0 if q.blank else q.r) for q in exp])

    def test_safety_dims_small_figure(self):
        sh = {"fps": 30, "outputs": [{"type": "laser", "dac": None, "safety": {"min_size": 2000}}],
              "tracks": [{"type": "laser", "gen": "medgrupo"}]}
        p = Player(sh, outputs=[])
        p.clip = [Frame([Point(0, 0, 255, 255, 255), Point(100, 100, 255, 255, 255)])]
        fr = p.laser_frame(0.0)
        self.assertLess(max(q.r for q in fr), 20)                # 100/2000 of the brightness: practically dark
        p.clip = [Frame([Point(-8000, -8000, 255, 0, 0), Point(8000, 8000, 255, 0, 0)])]
        self.assertEqual(max(q.r for q in p.laser_frame(0.0)), 255)


class TestPerformance(unittest.TestCase):
    def test_60fps_8_universes_plus_osc(self):
        port = free_port()
        listener = OscIn(port)
        tracks = [{"type": "dmx", "universe": u, "address": 1, "keys": [[0, [0] * 24], [5, [255] * 24]]}
                  for u in range(1, 9)]
        tracks.append({"type": "osc", "address": "/spell/x", "keys": [[0, 0.0], [5, 1.0]]})
        sh = {"fps": 60, "duration": 5.0, "tracks": tracks,
              "outputs": [{"type": "sacn", "universes": list(range(1, 9)), "interfaces": ["127.0.0.1"]},
                          {"type": "osc", "host": "127.0.0.1", "port": port}]}
        p = Player(sh)
        marks = []
        tick = p._tick

        def measured(t):
            marks.append((time.perf_counter(), t))
            tick(t)
        p._tick = measured
        try:
            p.start()
            p.play()
            self.assertTrue(p.wait(15))
        finally:
            p.close()
            listener.close()
        dts = [b[0] - a[0] for a, b in zip(marks, marks[1:])]
        jitter = max(abs(d - 1 / 60) for d in dts)
        drift = (marks[-1][0] - marks[0][0]) - (marks[-1][1] - marks[0][1])
        print(f"\nPERF 60fps 8 sACN universes + 1 OSC: ticks={len(marks)} "
              f"jitter_max={jitter * 1000:.1f}ms drift={drift * 1000:.1f}ms", file=sys.stderr)
        self.assertGreater(len(marks), 280)                      # ~300 ticks in 5 s
        self.assertLess(abs(drift), 0.05)
        self.assertLess(jitter, 0.020)


if __name__ == "__main__":
    unittest.main()
