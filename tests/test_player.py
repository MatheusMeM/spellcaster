# Player: equivalencia byte a byte com Engine.run(look), transporte local e por OSC, loop,
# laser sincronizado no emulador do Ether Dream e desempenho a 60 fps com 8 universos + OSC.
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
    """Saida de teste: guarda os bytes de cada universo (contrato send(universe, data) / close())."""

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
    def test_medgrupo_spell_igual_ao_engine(self):
        """Aceite 1: o track pyfx do .spell da os mesmos bytes do universo 1 que Engine.run(look)."""
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


class TestTransporte(unittest.TestCase):
    def test_play_pause_locate_play_e_osc(self):
        """Aceite 4: transporte local e remoto por OSC em loopback."""
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
            self.assertEqual(p.clock.time, t)                    # pausado congela
            rc.send("/spellcaster/locate", 0.5)
            self.assertTrue(wait_until(lambda: abs(p.clock.time - 0.5) < 1e-6))
            n = len(cap.frames)
            rc.send("/spellcaster/play")
            self.assertTrue(wait_until(lambda: p.clock.state == "play"))
            self.assertTrue(wait_until(lambda: len(cap.frames) > n + 5))
            self.assertGreater(cap.frames[-1][1][0], 120)        # depois do locate o valor esta acima da metade
            self.assertTrue(p.wait(3))                           # duracao atingida -> stop
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
            self.assertEqual(p.clock.state, "play")              # com loop nao para no fim
            self.assertLess(p.clock.time, 0.25)
            self.assertGreater(len(cap.frames), 30)
        finally:
            p.close()

    def test_saida_sacn_em_loopback(self):
        rx = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        rx.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            rx.bind(("127.0.0.1", sacn.PORT))
        except OSError as e:
            rx.close()
            self.skipTest(f"porta 5568 ocupada: {e}")
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

    def test_cue_disparada_pela_timeline(self):
        """Aceite 3 dentro do player: track cue dispara GO e o fade sai nos universos."""
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
            self.assertTrue(any(60 < v < 200 for v in vals), "fade nao foi amostrado")
        finally:
            p.close()


class TestLaser(unittest.TestCase):
    def test_frame_150_em_t5_no_emulador(self):
        """Aceite 5: t = 5 s a 30 fps = frame 150 do .ild, com safety() antes de sair."""
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
            p.pause()                                            # tempo congelado em 5 s
            self.assertTrue(wait_until(lambda: len(em.received) >= len(exp)))
        finally:
            p.close()
            em.stop()
        got = em.received[:len(exp)]
        self.assertEqual([(r[1], r[2]) for r in got], [(q.x, q.y) for q in exp])
        self.assertEqual([r[3] // 257 for r in got], [(0 if q.blank else q.r) for q in exp])

    def test_safety_apaga_figura_pequena(self):
        sh = {"fps": 30, "outputs": [{"type": "laser", "dac": None, "safety": {"min_size": 2000}}],
              "tracks": [{"type": "laser", "gen": "medgrupo"}]}
        p = Player(sh, outputs=[])
        p.clip = [Frame([Point(0, 0, 255, 255, 255), Point(100, 100, 255, 255, 255)])]
        fr = p.laser_frame(0.0)
        self.assertLess(max(q.r for q in fr), 20)                # 100/2000 do brilho: praticamente apagado
        p.clip = [Frame([Point(-8000, -8000, 255, 0, 0), Point(8000, 8000, 255, 0, 0)])]
        self.assertEqual(max(q.r for q in p.laser_frame(0.0)), 255)


class TestDesempenho(unittest.TestCase):
    def test_60fps_8_universos_mais_osc(self):
        port = free_port()
        ouvinte = OscIn(port)
        tracks = [{"type": "dmx", "universe": u, "address": 1, "keys": [[0, [0] * 24], [5, [255] * 24]]}
                  for u in range(1, 9)]
        tracks.append({"type": "osc", "address": "/spell/x", "keys": [[0, 0.0], [5, 1.0]]})
        sh = {"fps": 60, "duration": 5.0, "tracks": tracks,
              "outputs": [{"type": "sacn", "universes": list(range(1, 9)), "interfaces": ["127.0.0.1"]},
                          {"type": "osc", "host": "127.0.0.1", "port": port}]}
        p = Player(sh)
        marcas = []
        tick = p._tick

        def medido(t):
            marcas.append((time.perf_counter(), t))
            tick(t)
        p._tick = medido
        try:
            p.start()
            p.play()
            self.assertTrue(p.wait(15))
        finally:
            p.close()
            ouvinte.close()
        dts = [b[0] - a[0] for a, b in zip(marcas, marcas[1:])]
        jitter = max(abs(d - 1 / 60) for d in dts)
        deriva = (marcas[-1][0] - marcas[0][0]) - (marcas[-1][1] - marcas[0][1])
        print(f"\nPERF 60fps 8 universos sACN + 1 OSC: ticks={len(marcas)} "
              f"jitter_max={jitter * 1000:.1f}ms deriva={deriva * 1000:.1f}ms", file=sys.stderr)
        self.assertGreater(len(marcas), 280)                     # ~300 ticks em 5 s
        self.assertLess(abs(deriva), 0.05)
        self.assertLess(jitter, 0.020)


if __name__ == "__main__":
    unittest.main()
