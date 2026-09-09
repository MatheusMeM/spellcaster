import unittest

from spellcaster.core import registry


class TestRegistry(unittest.TestCase):
    def test_call_e_schema(self):
        @registry.command("soma_teste", mcp=False)
        def soma(a: int, b: float = 1.5, on: bool = False):
            """Soma."""
            return a + b if on else a
        self.assertEqual(registry.call("soma_teste", a="2", b="0.5", on="true"), 2.5)
        self.assertEqual(registry.call("soma_teste", a="2"), 2)
        s = [c for c in registry.schema() if c["name"] == "soma_teste"][0]
        self.assertEqual(s["doc"], "Soma."); self.assertFalse(s["mcp"])
        self.assertEqual(s["params"][1], {"name": "b", "type": "float", "default": 1.5})


if __name__ == "__main__":
    unittest.main()
