# F2: perfil, patch (sobreposicao), Group e o aceite byte a byte do MED GRUPO reescrito em fixtures.
import time, unittest

from spellcaster.core.engine import Engine
from spellcaster.fixtures.group import Group
from spellcaster.fixtures.patch import Patch, PatchError
from spellcaster.fixtures.profile import Profile, ProfileError, load


class TestProfile(unittest.TestCase):
    def test_offsets_sobrepostos(self):
        with self.assertRaises(ProfileError):
            Profile({"name": "x", "channels": [{"name": "pan", "offset": 0, "fine": 1},
                                               {"name": "tilt", "offset": 1}]})

    def test_sem_offset_e_sem_canal(self):
        self.assertRaises(ProfileError, Profile, {"name": "x", "channels": [{"name": "a"}]})
        self.assertRaises(ProfileError, Profile, {"name": "x", "channels": []})

    def test_bsw_calibrado(self):
        p = load("bsw_scorpio_17")
        self.assertEqual(p.size, 17)                       # ch12 sem funcao, mas o footprint reserva os 17
        self.assertEqual(p.channels["pan"][:2], (0, 1))    # 16 bit
        self.assertEqual(p.channels["zoom"][2]["13deg"], 80)
        self.assertEqual(p.channels["color"][2]["amarelo"], 48)


class TestPatch(unittest.TestCase):
    def test_17ch_espacados_de_16_dao_erro_com_os_dois_nomes(self):
        p = Patch()
        p.add("mv1", "bsw_scorpio_17", 1, 300)
        with self.assertRaises(PatchError) as e:
            p.add("mv2", "bsw_scorpio_17", 1, 316)
        self.assertIn("mv1", str(e.exception))
        self.assertIn("mv2", str(e.exception))
        self.assertIn("316", str(e.exception))
        p.add("mv2", "bsw_scorpio_17", 1, 320)             # o repatch de 20 em 20 passa

    def test_limites(self):
        p = Patch()
        self.assertRaises(PatchError, p.add, "x", "bsw_scorpio_17", 1, 500)
        p.add("x", "bsw_scorpio_17", 1, 400)
        self.assertRaises(PatchError, p.add, "x", "dimmer_1", 1, 10)     # nome repetido
        p.add("y", "bsw_scorpio_17", 2, 400)                             # outro universo nao colide
        self.assertRaises(PatchError, p.set, "z", dim=1)
        self.assertRaises(PatchError, p.set, "x", inexistente=1)

    def test_valores_por_nome_fracao_e_16bit(self):
        p = Patch()
        p.add("mv", "bsw_scorpio_17", 1, 10)
        p.set("mv", color="amarelo", gobo="raios", shutter="aberto", zoom="13deg", dim=200)
        self.assertEqual(p.get("mv", "color"), 48)
        self.assertEqual(p.get("mv", "gobo"), 120)
        self.assertEqual(p.get("mv", "shutter"), 255)
        self.assertEqual(p.get("mv", "zoom"), 80)
        self.assertEqual(p.get("mv", "dim"), 200)
        self.assertRaises(PatchError, p.set, "mv", color="roxo")
        p.set("mv", pan=128.0)                             # 16 bit: valor cheio = 128 * 257
        self.assertEqual(p.get("mv", "pan"), (128, 128))
        p.set("mv", pan=128.5)                             # meio degrau de 8 bit, resolvido no canal fine
        self.assertEqual(p.get("mv", "pan"), (129, 0))
        p.set("mv", dim=0.5)                               # float em 0..1 = fracao
        self.assertEqual(p.get("mv", "dim"), 127)
        p.set("mv", dim=300)                               # clamp
        self.assertEqual(p.get("mv", "dim"), 255)

    def test_velocidade(self):
        """8 fixtures x 17 ch x 1000 frames: 23 ms nesta maquina. Limite com folga de 4x para nao piscar em CI."""
        p = Patch()
        nomes = [f"m{i}" for i in range(8)]
        for i, n in enumerate(nomes):
            p.add(n, "bsw_scorpio_17", 1, 1 + 20 * i)
        kw = dict(pan=128.0, tilt=200.0, color=48, gobo=104, grot=135, gobo2=0, shutter=255, dim=255,
                  focus=128, zoom=230, prism=0, frost=0, speed=0, function=0)
        t0 = time.perf_counter()
        for _ in range(1000):
            for n in nomes:
                p.set(n, **kw)
        dt = time.perf_counter() - t0
        self.assertLess(dt, 0.1, f"{dt * 1000:.1f} ms")


class TestGroup(unittest.TestCase):
    def test_knobs_de_calibracao(self):
        p = Patch()
        p.add("a", "bsw_scorpio_17", 1, 1)
        g = Group(p, ["a"], "g", amp=38, slew_tilt=1.0)
        g.aim(dz=1.0, dim=255)
        self.assertEqual(p.get("a", "tilt")[0], 166)       # 128 + 38 * 1,0
        g.amp = 76                                         # knob no ar, sem reeditar o show
        g.last.clear(); g.aim(dz=1.0, dim=255)
        self.assertEqual(p.get("a", "tilt")[0], 204)

    def test_only_escreve_um_e_anda_a_histerese_dos_outros(self):
        p = Patch()
        for i in range(2):
            p.add(f"a{i}", "bsw_scorpio_17", 1, 1 + 20 * i)
        g = Group(p, ["a0", "a1"], "g2", defaults={"dim": 0})
        g.aim(dz=1.0, dim=255, only="a1")
        self.assertEqual(p.get("a0", "dim"), 0)
        self.assertEqual(p.get("a1", "dim"), 255)
        self.assertIn("a0", g.last)


class TestShowFx(unittest.TestCase):
    def test_mesmos_bytes_do_universo_1(self):
        """Aceite F2: escrever por nome de fixture (par, fresnel e um moving mirado por Group) da os mesmos
        512 bytes do universo 1 que escrever por endereco cru no Engine."""
        p = Patch()
        p.add("par", "par_rgb_3", 1, 1)
        p.add("fresnel", "dimmer_1", 1, 29)
        p.add("mv", "bsw_scorpio_17", 1, 300)
        p.set("par", r=200, g=100, b=0)
        p.set("fresnel", dim="meio")
        Group(p, ["mv"], "g_aceite", defaults={"shutter": "aberto"}).aim(dz=1.0, dim=255, color="amarelo", zoom=200)

        eng = Engine([])
        eng.apply({1: [200, 100, 0], 29: [128],
                   # pan 128 + 90 * 255/540 = 170,5 e tilt 128 + 38 = 166, ambos em 16 bit
                   300: [171, 42, 166, 166, 48, 0, 0, 0, 255, 255, 0, 0, 200]})
        a, b = eng.universes[1].data, p.universes[1].data
        self.assertEqual(bytes(a), bytes(b), [(c + 1, a[c], b[c]) for c in range(512) if a[c] != b[c]][:8])


if __name__ == "__main__":
    unittest.main()
