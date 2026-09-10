# GUI commands (spellcaster/gui/api.py) in memory: open, edit track/keyframe, save and reopen.
# No network here: transport is only tested in the stopped state.
import json
import os
import pathlib
import tempfile
import unittest

from spellcaster.core import registry
from spellcaster.gui import api

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPELL = str(ROOT / "shows" / "medgrupo.spell")


class TestGuiApi(unittest.TestCase):
    def setUp(self):
        api.show_new()

    def test_show_open_brings_medgrupo(self):
        sh = registry.call("show_open", file=SPELL)
        self.assertEqual(sh["name"], "MED GRUPO RJ - plenary 09/2026")
        self.assertEqual(sh["fps"], 30)
        self.assertEqual(len(sh["tracks"]), 3)
        self.assertNotIn("_dir", sh)                      # internal keys do not leak into the GUI
        self.assertTrue(api.SHOW["_dir"].endswith("shows"))

    def test_edit_and_save_round_trip(self):
        registry.call("show_open", file=SPELL)
        i = registry.call("track_add", type="dmx", universe="2", address="10", label="test")
        self.assertEqual(i, 3)
        registry.call("key_set", track=i, t="0", value="[0, 0, 0]")
        registry.call("key_set", track=i, t="2.5", value="[255, 128, 0]", curve="inout")
        registry.call("key_set", track=i, t="1", value="[10, 10, 10]")
        keys = api.SHOW["tracks"][i]["keys"]
        self.assertEqual([k[0] for k in keys], [0.0, 1.0, 2.5])     # sorted by time
        self.assertEqual(keys[2], [2.5, [255, 128, 0], "inout"])

        out = os.path.join(tempfile.gettempdir(), "spellcaster_test_gui.spell")
        saved = registry.call("show_save", file=out)
        self.assertEqual(saved, os.path.abspath(out))
        before = registry.call("show_get")
        try:
            api.show_new()
            self.assertEqual(registry.call("show_get")["tracks"], [])
            after = registry.call("show_open", file=out)
            self.assertEqual(json.dumps(after, sort_keys=True), json.dumps(before, sort_keys=True))
        finally:
            os.remove(out)

    def test_key_set_moves_and_key_del_removes(self):
        i = registry.call("track_add")
        registry.call("key_set", track=i, t="1", value="255")
        registry.call("key_set", track=i, t="1", value="10", curve="hold")   # same t = replaces
        self.assertEqual(api.SHOW["tracks"][i]["keys"], [[1.0, 10, "hold"]])
        self.assertEqual(registry.call("key_del", track=i, t="1"), 1)
        self.assertEqual(registry.call("key_del", track=i, t="1"), 0)
        with self.assertRaises(ValueError):
            registry.call("key_set", track=i, t="0", value="0", curve="spring")
        with self.assertRaises(IndexError):
            registry.call("key_set", track="9", t="0", value="0")

    def test_track_del(self):
        registry.call("show_open", file=SPELL)
        tr = registry.call("track_del", index="0")
        self.assertEqual(tr["type"], "pyfx")
        self.assertEqual(len(registry.call("show_get")["tracks"]), 2)

    def test_show_set_validates_and_replaces(self):
        sh = {"name": "x", "fps": 25, "duration": 3.0, "tracks": [{"type": "dmx", "keys": [[0, 1]]}]}
        got = registry.call("show_set", data=json.dumps(sh))
        self.assertEqual(got["fps"], 25)
        self.assertEqual(got["version"], 1)
        with self.assertRaises(ValueError):
            registry.call("show_set", data='{"tracks": 3}')
        with self.assertRaises(ValueError):
            registry.call("show_set", data='[1, 2]')

    def test_patch_check_reports_overlap(self):
        registry.call("show_set", data=json.dumps({
            "tracks": [], "patch": [{"name": "bsw_1", "profile": "bsw_scorpio_17", "universe": 1, "address": 300},
                                    {"name": "bsw_2", "profile": "bsw_scorpio_17", "universe": 1, "address": 316}]}))
        r = registry.call("patch_check")
        self.assertEqual(len(r["rows"]), 1)                          # the second one did not go in
        self.assertIn("overlap", r["error"])
        self.assertIn("316", r["error"])
        registry.call("show_set", data=json.dumps({
            "tracks": [], "patch": [{"name": "bsw_1", "profile": "bsw_scorpio_17", "universe": 1, "address": 300},
                                    {"name": "bsw_2", "profile": "bsw_scorpio_17", "universe": 1, "address": 317}]}))
        r = registry.call("patch_check")
        self.assertIsNone(r["error"])
        self.assertEqual([x["address"] for x in r["rows"]], [300, 317])
        self.assertEqual(r["rows"][0]["channels"], 17)

    def test_transport_state_stopped(self):
        registry.call("show_open", file=SPELL)
        st = registry.call("transport_state")
        self.assertEqual(st["state"], "stop")
        self.assertEqual(st["dur"], 85.9)
        with self.assertRaises(ValueError):
            registry.call("transport", state="fly")

    def test_profiles_lists_the_profiles(self):
        self.assertIn("bsw_scorpio_17", registry.call("profiles"))


if __name__ == "__main__":
    unittest.main()
