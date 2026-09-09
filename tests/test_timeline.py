# Curvas e keyframes (valores nos instantes exatos e entre eles), tracks, cue list e markers.
import array
import os
import shutil
import subprocess
import tempfile
import unittest
import wave

from spellcaster.core.universe import Universes
from spellcaster.timeline import markers
from spellcaster.timeline.cues import CueList
from spellcaster.timeline.model import Keys, Timeline


class TestCurvas(unittest.TestCase):
    def test_instantes_exatos_e_entre(self):
        k = Keys([[0, 0.0], [2, 100.0]])
        self.assertEqual(k.value(0), 0.0)
        self.assertEqual(k.value(2), 100.0)
        self.assertEqual(k.value(1), 50.0)
        self.assertEqual(k.value(-5), 0.0)          # antes do primeiro: segura
        self.assertEqual(k.value(99), 100.0)        # depois do ultimo: segura
        self.assertIsNone(Keys([]).value(1))

    def test_ease_inout_no_meio(self):
        k = Keys([[0, 0.0], [1, 1.0, "inout"]])
        self.assertAlmostEqual(k.value(0.5), 0.5)
        self.assertAlmostEqual(k.value(0.25), 0.15625)
        self.assertEqual(k.value(0.0), 0.0)
        self.assertEqual(k.value(1.0), 1.0)

    def test_in_out_hold_bezier(self):
        self.assertAlmostEqual(Keys([[0, 0.0], [1, 1.0, "in"]]).value(0.5), 0.25)
        self.assertAlmostEqual(Keys([[0, 0.0], [1, 1.0, "out"]]).value(0.5), 0.75)
        h = Keys([[0, 10], [2, 20, "hold"], [4, 30, "hold"]])
        self.assertEqual(h.value(1.9), 10)          # "hold" no destino: degrau, segura o valor anterior
        self.assertEqual(h.value(2), 20)
        self.assertEqual(h.value(3.9), 20)
        self.assertEqual(h.value(4), 30)
        b = Keys([[0, 0.0], [1, 1.0, "bezier", [1 / 3, 2 / 3]]])
        self.assertAlmostEqual(b.value(0.3), 0.3)   # controles em 1/3 e 2/3 = reta

    def test_lista_por_canal_e_texto(self):
        k = Keys([[0, [0, 100, 200]], [2, [200, 100, 0]]])
        self.assertEqual(k.value(1), [100.0, 100.0, 100.0])
        s = Keys([[0, "stop"], [1, "play"]])
        self.assertEqual(s.value(0.5), "stop")      # texto nao interpola
        self.assertEqual(s.value(1.5), "play")

    def test_crossed(self):
        k = Keys([[1, "a"], [2, "b"], [3, "c"]])
        self.assertEqual([x.value for x in k.crossed(0.5, 2.0)], ["a", "b"])
        self.assertEqual([x.value for x in k.crossed(2.0, 2.5)], [])
        self.assertEqual([x.value for x in k.crossed(2.0, 3.0)], ["c"])


class TestTracks(unittest.TestCase):
    def test_dmx_e_media_capture(self):
        tl = Timeline({"tracks": [
            {"type": "dmx", "universe": 2, "address": 10, "keys": [[0, [0, 0]], [2, [200, 100]]]},
            {"type": "media", "player": "capture", "universe": 2, "address": 400, "clip": 3,
             "keys": [[0, "stop"], [1, "play"]]},
            {"type": "osc", "address": "/x", "keys": [[0, 0.0], [1, 1.0]]},
            {"type": "fixture", "universe": 2, "address": 1},
        ]})
        self.assertEqual((len(tl.dmx), len(tl.osc), len(tl.laser)), (3, 1, 0))
        u = Universes()
        tl.apply(u, 1.0)
        d = u.get_or_create(2).data
        self.assertEqual((d[9], d[10]), (100, 50))
        self.assertEqual((d[399], d[400]), (10, 3))      # play = 10, clipe 3
        tl.apply(u, 0.0)
        self.assertEqual(d[399], 28)                     # stop = 28

    def test_resolvedor_de_fixture_injetado(self):
        tl = Timeline({"tracks": [{"type": "fixture", "universe": 1, "address": 5}]})
        u = Universes()
        tl.apply(u, 0.0)
        self.assertEqual(u.get_or_create(1).data[4], 0)  # sem resolvedor: ignorado
        Timeline.resolvers["fixture"] = lambda tr, t, uni: uni.get_or_create(tr.universe).set(tr.address, [77])
        try:
            tl.apply(u, 0.0)
            self.assertEqual(u.get_or_create(1).data[4], 77)
        finally:
            del Timeline.resolvers["fixture"]


class TestCues(unittest.TestCase):
    def test_go_fade_e_follow_com_wait(self):
        cl = CueList([{"name": "1", "values": {"1/1": [0, 0]}},
                      {"name": "2", "fade": 1.0, "follow": True, "values": {"1/1": [100, 200]}},
                      {"name": "3", "wait": 0.5, "values": {"1/1": [10, 20]}}])
        cl.go(0.0)
        self.assertEqual(cl.update(0.0)[(1, 1)], [0, 0])
        cl.go(1.0)                                        # GO na cue 2, fade de 1 s
        cl.update(1.0)
        self.assertEqual(cl.update(1.25)[(1, 1)], [25.0, 50.0])
        self.assertEqual(cl.update(1.5)[(1, 1)], [50.0, 100.0])
        self.assertEqual(cl.update(2.0)[(1, 1)], [100, 200])
        self.assertEqual(cl.index, 1)
        self.assertEqual(cl.update(2.2)[(1, 1)], [100, 200])   # follow disparou, mas wait = 0,5 s
        self.assertEqual(cl.update(2.5)[(1, 1)], [10, 20])     # cue 3 entra
        self.assertEqual(cl.index, 2)
        cl.reset()
        self.assertEqual((cl.index, cl.state), (-1, {}))


class TestMarkers(unittest.TestCase):
    def test_video_tres_cortes(self):
        if not shutil.which("ffmpeg"):
            self.skipTest("ffmpeg fora do PATH")
        d = tempfile.mkdtemp()
        try:
            f = os.path.join(d, "cortes.mkv")
            cmd = ["ffmpeg", "-y", "-loglevel", "error"]
            for c in ("black", "red", "lime", "blue"):
                cmd += ["-f", "lavfi", "-i", f"color=c={c}:s=320x240:r=10:d=1"]
            cmd += ["-filter_complex", "[0:v][1:v][2:v][3:v]concat=n=4:v=1[v]", "-map", "[v]", "-c:v", "ffv1", f]
            subprocess.run(cmd, check=True, capture_output=True)
            ts = markers.video(f, 0.3)
            self.assertEqual(len(ts), 3, ts)
            for got, want in zip(ts, (1.0, 2.0, 3.0)):
                self.assertAlmostEqual(got, want, delta=0.1)
        finally:
            shutil.rmtree(d, ignore_errors=True)

    def test_audio_tres_batidas(self):
        d = tempfile.mkdtemp()
        try:
            f = os.path.join(d, "batidas.wav")
            sr = 8000
            a = array.array("h", [0]) * (4 * sr)
            for t in (1.0, 2.0, 3.0):
                for i in range(int(0.05 * sr)):
                    a[int(t * sr) + i] = 20000 if (i // 4) % 2 else -20000
            with wave.open(f, "wb") as w:
                w.setnchannels(1)
                w.setsampwidth(2)
                w.setframerate(sr)
                w.writeframes(a.tobytes())
            ts = markers.find(f)
            self.assertEqual(len(ts), 3, ts)
            for got, want in zip(ts, (1.0, 2.0, 3.0)):
                self.assertAlmostEqual(got, want, delta=0.05)
        finally:
            shutil.rmtree(d, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
