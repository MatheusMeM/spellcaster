# paths.py nos dois modos (fonte e congelado) e config.json ao lado do executavel.
import importlib
import json
import os
import pathlib
import sys
import tempfile
import unittest

from spellcaster import config, paths


class TestPaths(unittest.TestCase):
    def tearDown(self):
        for a in ("frozen", "_MEIPASS"):
            if hasattr(sys, a):
                delattr(sys, a)
        importlib.reload(paths)

    def test_source(self):
        """Rodando do fonte: ROOT = raiz do repo (pai de spellcaster/)."""
        self.assertFalse(paths.FROZEN)
        self.assertEqual(paths.ROOT, pathlib.Path(paths.__file__).resolve().parents[1])
        self.assertTrue((paths.PROFILES / "bsw_scorpio_17.json").is_file())
        self.assertTrue((paths.WEB / "index.html").is_file())
        self.assertEqual(paths.CONFIG.name, "config.json")

    def test_frozen(self):
        """Congelado: ROOT = pasta do exe; assets vem do bundle (_MEIPASS)."""
        with tempfile.TemporaryDirectory() as d:
            app, internal = pathlib.Path(d) / "app", pathlib.Path(d) / "app" / "_internal"
            (internal / "profiles").mkdir(parents=True)
            sys.frozen = True
            sys._MEIPASS = str(internal)
            old, sys.executable = sys.executable, str(app / "Spellcaster.exe")
            try:
                importlib.reload(paths)
                self.assertTrue(paths.FROZEN)
                self.assertEqual(paths.ROOT, app.resolve())
                self.assertEqual(paths.CONFIG, app.resolve() / "config.json")
                self.assertEqual(paths.WEB, internal / "spellcaster" / "gui" / "web")
                # sem shows/ ao lado do exe cai no que veio no bundle; profiles/ existe no bundle
                self.assertEqual(paths.PROFILES, internal / "profiles")
                self.assertEqual(paths.SHOWS, internal / "shows")
                (app / "shows").mkdir()
                importlib.reload(paths)
                self.assertEqual(paths.SHOWS, app.resolve() / "shows")   # o do usuario ganha
            finally:
                sys.executable = old


class TestConfig(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.old, config.CONFIG = config.CONFIG, pathlib.Path(self.tmp) / "config.json"

    def tearDown(self):
        config.CONFIG = self.old

    def test_defaults_and_roundtrip(self):
        self.assertEqual(config.load()["port"], 8000)          # arquivo ausente = defaults
        config.save(port=9001, last_show="medgrupo.spell")
        self.assertEqual(json.loads(config.CONFIG.read_text())["port"], 9001)
        d = config.load()
        self.assertEqual((d["port"], d["last_show"], d["skin"]), (9001, "medgrupo.spell", "feiticaria"))
        config.CONFIG.write_text("{ nao e json", encoding="utf-8")
        self.assertEqual(config.load(), config.DEFAULTS)       # quebrado = defaults, sem excecao
        os.remove(config.CONFIG)


if __name__ == "__main__":
    unittest.main()
