# Rodada 6 do design: o laser como modulo do orquestrador. Valida design/laser/module.json (manifesto gerado por
# Bind.manifest()) e design/laser/graph.json (trecho graph do .spell gerado por Bind.graph()):
# todo wire liga portas de tipo compativel e todo endereco de saida existe no manifesto.
import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1] / "design" / "laser"

# tipo da porta de saida por tipo de comando/parametro do manifesto (INTEGRACAO.md, "Tipos de porta")
PORT = {"trigger": "trigger", "toggle": "toggle", "value": "number", "float": "number", "int": "number", "bool": "toggle"}
# o que cada tipo de fonte pode ligar sem conversao
OK = {"trigger": {"trigger", "toggle", "number"}, "number": {"number"}, "toggle": {"toggle"}}
FILTER = {"filter.lag": ("number", "number"), "logic.toggle": ("trigger", "toggle")}


def src_type(port):
    if port.startswith("midi/cc:"):
        return "number"
    if port.startswith("midi/note:") or port.startswith("key/"):
        return "trigger"
    raise AssertionError("fonte desconhecida: " + port)


class TestLaserGraph(unittest.TestCase):
    def setUp(self):
        self.m = json.loads((ROOT / "module.json").read_text(encoding="utf-8"))
        self.g = json.loads((ROOT / "graph.json").read_text(encoding="utf-8"))
        self.ports = {}
        for n, p in self.m["parameters"].items():
            self.ports[n] = PORT[p["type"]]
        for n, c in self.m["commands"].items():
            self.ports[n] = PORT[c["type"]]

    def test_manifesto(self):
        m = self.m
        self.assertEqual(m["name"], "laser")
        for k in ("parameters", "values", "commands", "dependency"):
            self.assertIn(k, m)
        for n, c in m["commands"].items():
            self.assertIn(c["context"], ("action", "mapping", "both"), n)
            self.assertIn(c["type"], ("trigger", "toggle", "value"), n)
        for n, v in m["values"].items():
            self.assertTrue(v["readOnly"], n)
        for d in m["dependency"]:
            self.assertIn(d["source"], m["commands"])
            self.assertIn(d["target"], m["commands"])
        # os enderecos que a rodada 6 promete
        for n in ("kpps", "shutter", "arm", "net/sacn", "clip"):
            self.assertIn(n, m["commands"])
        for n in ("limit/r", "dmx/addr"):
            self.assertIn(n, m["parameters"])

    def test_wires(self):
        nodes = {n["uid"]: n for n in self.g["nodes"]}
        self.assertIn("laser/1", nodes)
        self.assertEqual(self.g["states"], [])
        self.assertEqual(set(nodes), set(self.g["view"]))
        self.assertTrue(self.g["wires"])
        for w in self.g["wires"]:
            t = src_type(w["from"])
            self.assertTrue(w["to"].startswith("laser/1/"), w)
            name = w["to"][len("laser/1/"):]
            self.assertIn(name, self.ports, "endereco fora do manifesto: " + w["to"])
            if "filter" in w:
                f = nodes[w["filter"]]
                fin, fout = FILTER[f["type"]]
                self.assertEqual(t, fin, w)
                t = fout
            dst = self.ports[name]
            self.assertIn(dst, OK[t], "porta incompativel: %s (%s) -> %s (%s)" % (w["from"], t, w["to"], dst))
            if t == "trigger" and dst == "number":
                need = set(self.m["commands"][name]["parameters"])
                self.assertTrue(need & set(w.get("args", {})), "trigger em valor precisa de argumento: " + w["to"])
        # todo cc continuo passa por Lag
        for w in self.g["wires"]:
            if w["from"].startswith("midi/cc:") and self.ports[w["to"][8:]] == "number":
                self.assertEqual(nodes[w["filter"]]["type"], "filter.lag", w)

    def test_params_do_no(self):
        laser = [n for n in self.g["nodes"] if n["uid"] == "laser/1"][0]
        self.assertEqual(set(laser["params"]), set(self.m["parameters"]))


if __name__ == "__main__":
    unittest.main()
