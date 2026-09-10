# .spell file: load/save, version, migration and the MED GRUPO show.
import json
import os
import pathlib
import tempfile
import unittest

from spellcaster import show as showfile
from spellcaster.cli import load_show
from spellcaster.timeline.model import Timeline

ROOT = pathlib.Path(__file__).resolve().parents[1]


class TestSpell(unittest.TestCase):
    def test_save_load_round_trip(self):
        sh = {"name": "t", "fps": 60, "duration": 2.0, "outputs": [],
              "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, [0]], [2, [255]]]}], "cues": []}
        d = tempfile.mkdtemp()
        try:
            p = showfile.save(os.path.join(d, "t.spell"), sh)
            got = showfile.load(p)
            self.assertEqual(got["version"], showfile.VERSION)
            self.assertEqual(got["_dir"], os.path.abspath(d))
            got.pop("_dir")
            got.pop("version")
            self.assertEqual(got, sh)
            showfile.save(p, got)                       # _dir does not go back into the file
            self.assertNotIn("_dir", json.loads(pathlib.Path(p).read_text(encoding="utf-8")))
        finally:
            for f in os.listdir(d):
                os.remove(os.path.join(d, f))
            os.rmdir(d)

    def test_migration_and_future_version(self):
        self.assertEqual(showfile.migrate({"tracks": []})["version"], showfile.VERSION)   # v0 -> v1
        with self.assertRaises(ValueError):
            showfile.migrate({"version": showfile.VERSION + 1})

    def test_medgrupo_spell(self):
        sh = showfile.load(str(ROOT / "shows" / "medgrupo.spell"))
        mod = load_show(str(ROOT / "shows" / "medgrupo.py"))
        self.assertAlmostEqual(sh["duration"], mod.DUR, delta=0.01)
        tl = Timeline(sh, sh["_dir"])
        self.assertEqual([k.type for k in tl.tracks], ["pyfx", "fx", "laser"])
        self.assertEqual(len(tl.dmx), 1)
        self.assertEqual(tl.dmx[0].fn(0.0), mod.look(0.0))         # pyfx points at the show look
        self.assertTrue((ROOT / "shows" / sh["tracks"][2]["clip"]).exists())


if __name__ == "__main__":
    unittest.main()
