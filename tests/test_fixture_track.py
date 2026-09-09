import unittest
from spellcaster.core.universe import Universes
from spellcaster.timeline.model import Timeline
import spellcaster.fixtures  # noqa: F401

SHOW = {"version": 1, "fps": 30, "duration": 4,
        "patch": [{"name": "bsw_1", "profile": "bsw_scorpio_17", "universe": 1, "address": 300}],
        "tracks": [{"type": "fixture", "fixture": "bsw_1", "dim": [[0, 0], [2, 255]], "color": [[0, "amarelo"]]}]}


class FixtureTrack(unittest.TestCase):
    def test_keyframes_by_channel_name(self):
        uni = Universes()
        tl = Timeline(SHOW)
        tl.apply(uni, 1.0)
        d = uni.get_or_create(1).data
        self.assertEqual(d[308], 127)        # dim (ch10 do BSW) na metade da rampa
        self.assertEqual(d[303], 48)         # color = amarelo
        tl.apply(uni, 2.0)
        self.assertEqual(uni.get_or_create(1).data[308], 255)


if __name__ == "__main__":
    unittest.main()
