# MCP: subprocesso stdio real e mcp_install num arquivo de %TEMP%.
import json
import os
import subprocess
import sys
import tempfile
import unittest

from spellcaster.mcp import install, server

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class TestStdio(unittest.TestCase):
    """Sobe `python -m spellcaster.mcp.server` de verdade e fala JSON-RPC por stdin/stdout."""

    @classmethod
    def setUpClass(cls):
        env = dict(os.environ, PYTHONPATH=ROOT, PYTHONIOENCODING="utf-8", PYTHONUTF8="1")
        cls.p = subprocess.Popen([sys.executable, "-m", "spellcaster.mcp.server"],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                 cwd=ROOT, env=env, text=True, encoding="utf-8", bufsize=1)

    @classmethod
    def tearDownClass(cls):
        try:
            cls.p.stdin.close()
            cls.p.wait(10)
        except (OSError, subprocess.TimeoutExpired):
            cls.p.kill()

    def rpc(self, mid, method, params=None):
        msg = {"jsonrpc": "2.0", "id": mid, "method": method}
        if params is not None:
            msg["params"] = params
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        while True:                                   # pula notificacoes do watcher do player
            line = self.p.stdout.readline()
            self.assertTrue(line, "servidor MCP fechou stdout")
            m = json.loads(line)
            if m.get("id") == mid:
                return m

    def notify(self, method, params=None):
        msg = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()

    def test_sessao_completa(self):
        r = self.rpc(1, "initialize", {"protocolVersion": "2025-06-18",
                                       "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}})
        res = r["result"]
        self.assertEqual(res["protocolVersion"], "2025-06-18")
        self.assertEqual(res["serverInfo"]["name"], "spellcaster")
        for cap in ("tools", "resources", "prompts"):
            self.assertIn(cap, res["capabilities"])
        self.notify("notifications/initialized")

        self.assertEqual(self.rpc(2, "ping")["result"], {})

        tools = self.rpc(3, "tools/list")["result"]["tools"]
        nomes = [t["name"] for t in tools]
        for n in ("net", "play_show", "stop", "monitor", "show_summary", "run_command"):
            self.assertIn(n, nomes)
        for t in tools:                               # todo inputSchema e um object JSON valido
            self.assertEqual(t["inputSchema"]["type"], "object")
            self.assertIsInstance(t["inputSchema"]["properties"], dict)
        locate = next(t for t in tools if t["name"] == "locate")
        self.assertEqual(locate["inputSchema"]["properties"]["t"]["type"], "number")
        self.assertEqual(locate["inputSchema"]["required"], ["t"])
        play = next(t for t in tools if t["name"] == "play_show")
        self.assertEqual(play["inputSchema"]["properties"]["loop"]["type"], "boolean")
        self.assertEqual(play["inputSchema"]["required"], ["file"])

        r = self.rpc(4, "tools/call", {"name": "net", "arguments": {"timeout": 1}})["result"]
        self.assertFalse(r["isError"])
        self.assertEqual(r["content"][0]["type"], "text")
        self.assertIn("interface", r["content"][0]["text"].lower())

        uris = [x["uri"] for x in self.rpc(5, "resources/list")["result"]["resources"]]
        self.assertEqual(sorted(uris), ["spell://log", "spell://net", "spell://patch", "spell://show"])
        c = self.rpc(6, "resources/read", {"uri": "spell://net"})["result"]["contents"][0]
        self.assertEqual(c["uri"], "spell://net")
        d = json.loads(c["text"])                     # o `net` unico devolve o dict com o texto em `report`
        self.assertIn("interfaces", d)
        self.assertIn("Interfaces", d["report"])

        prompts = self.rpc(7, "prompts/list")["result"]["prompts"]
        self.assertEqual(sorted(p["name"] for p in prompts), ["calibrar_grupo", "montar_show_do_video"])
        g = self.rpc(8, "prompts/get", {"name": "montar_show_do_video",
                                        "arguments": {"video": "medgrupo.mp4"}})["result"]
        self.assertEqual(g["messages"][0]["role"], "user")
        self.assertIn("medgrupo.mp4", g["messages"][0]["content"]["text"])

        self.assertEqual(self.rpc(9, "nao_existe")["error"]["code"], -32601)
        r = self.rpc(10, "tools/call", {"name": "nao_existe", "arguments": {}})["result"]
        self.assertTrue(r["isError"])


class TestInstall(unittest.TestCase):
    def test_grava_entrada_e_faz_backup(self):
        d = tempfile.mkdtemp(prefix="spell_mcp_")
        p = os.path.join(d, ".mcp.json")
        with open(p, "w", encoding="utf-8") as f:
            json.dump({"mcpServers": {"outro": {"command": "x"}}}, f)

        self.assertEqual(install.mcp_install("code", path=p, yes=True), p)
        with open(p, encoding="utf-8") as f:
            cfg = json.load(f)
        e = cfg["mcpServers"]["spellcaster"]
        self.assertEqual(e["args"], ["-m", "spellcaster.cli", "mcp"])
        self.assertTrue(e["command"].lower().endswith(("python.exe", "python", "python3")))
        self.assertEqual(e["env"]["PYTHONPATH"], install.ROOT)
        self.assertIn("outro", cfg["mcpServers"])     # nao apaga o que ja estava la
        self.assertTrue(os.path.exists(p + ".bak"))

        self.assertEqual(install.mcp_install("code", path=p, yes=True), p)   # idempotente: sem diff

    def test_target_invalido(self):
        self.assertRaises(ValueError, install.config_path, "nada")

    def test_config_path_desktop(self):
        self.assertTrue(install.config_path("desktop").endswith("claude_desktop_config.json"))
        self.assertTrue(install.config_path("code").endswith(".mcp.json"))


if __name__ == "__main__":
    unittest.main()
