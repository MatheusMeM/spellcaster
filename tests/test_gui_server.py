import base64, http.client, json, os, socket, struct, unittest

from spellcaster.core import registry
from spellcaster.gui import server as gs


def ws_frame(op, data, mask=b"\x11\x22\x33\x44"):
    """Client frame: FIN=1, masked (the server requires the browser side to mask)."""
    n = len(data)
    h = bytes((0x80 | op, 0x80 | n)) if n < 126 else bytes((0x80 | op, 0x80 | 126)) + struct.pack(">H", n)
    return h + mask + bytes(b ^ mask[i & 3] for i, b in enumerate(data))


def recv_frame(sock):
    def rd(n):
        b = b""
        while len(b) < n:
            c = sock.recv(n - len(b))
            if not c:
                raise EOFError
            b += c
        return b
    h = rd(2)
    op, n = h[0] & 0x0F, h[1] & 0x7F
    if n == 126:
        n = struct.unpack(">H", rd(2))[0]
    elif n == 127:
        n = struct.unpack(">Q", rd(8))[0]
    return op, (rd(n) if n else b"")


@registry.command("gui_echo_test", mcp=False)
def _echo(msg: str = "hi", n: int = 1):
    """Command only for the GUI test."""
    return {"msg": msg * n}


class TestGuiServer(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.srv = gs.GuiServer(host="127.0.0.1", port=0).start()
        cls.port = cls.srv.port

    @classmethod
    def tearDownClass(cls):
        cls.srv.stop()

    def http_get(self, path):
        c = http.client.HTTPConnection("127.0.0.1", self.port, timeout=5)
        c.request("GET", path)
        r = c.getresponse()
        body = r.read()
        c.close()
        return r.status, body

    def open_ws(self):
        s = socket.create_connection(("127.0.0.1", self.port), timeout=5)
        key = base64.b64encode(b"0123456789abcdef").decode()
        s.sendall(("GET /ws HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\n"
                   "Connection: Upgrade\r\nSec-WebSocket-Key: %s\r\n"
                   "Sec-WebSocket-Version: 13\r\n\r\n" % key).encode())
        head = b""
        while b"\r\n\r\n" not in head:
            head += s.recv(1)
        self.assertIn(b"101", head.split(b"\r\n")[0])
        self.assertIn(gs.accept_key(key).encode(), head)
        return s

    # ---- static HTTP ----
    def test_get_index_and_skin(self):
        st, body = self.http_get("/index.html")
        self.assertEqual(st, 200)
        self.assertIn(b"SPELLCASTER", body)
        self.assertIn(b'data-panel="timeline"', body)
        st, body = self.http_get("/skins/feiticaria/skin.css")
        self.assertEqual(st, 200)
        self.assertIn(b".chrome", body)                  # skin.css decorates the chrome

    def test_every_skin_has_json_and_css(self):
        web = os.path.join(os.path.dirname(gs.__file__), "web", "skins")
        for name in ("feiticaria", "corporate", "headspace", "bluesky", "quicksilver", "xp"):
            with open(os.path.join(web, name, "skin.json"), encoding="utf-8") as f:
                d = json.load(f)
            self.assertEqual(d["name"], name)
            self.assertIn(d["chrome"]["transport"], ("round", "square", "pill"))
            self.assertIn(d["chrome"]["visualizer"], ("bars", "scope", "none"))
            for var in ("--bg", "--panel", "--chrome", "--accent", "--text", "--muted", "--font", "--radius"):
                self.assertIn(var, d["vars"])
            self.assertEqual(self.http_get("/skins/%s/skin.css" % name)[0], 200)

    # ---- WebSocket ----
    def test_ws_handshake_and_command(self):
        s = self.open_ws()
        try:
            s.sendall(ws_frame(gs.TEXT, json.dumps({"id": 7, "cmd": "gui_echo_test",
                                                    "args": {"msg": "ab", "n": "2"}}).encode()))
            op, data = recv_frame(s)
            self.assertEqual(op, gs.TEXT)
            self.assertEqual(json.loads(data), {"id": 7, "result": {"msg": "abab"}})

            s.sendall(ws_frame(gs.TEXT, json.dumps({"id": 8, "cmd": "schema"}).encode()))
            op, data = recv_frame(s)
            names = [c["name"] for c in json.loads(data)["result"]]
            self.assertIn("gui_echo_test", names)

            s.sendall(ws_frame(gs.TEXT, json.dumps({"id": 9, "cmd": "does_not_exist"}).encode()))
            op, data = recv_frame(s)
            self.assertIn("error", json.loads(data))
        finally:
            s.close()

    def test_ws_ping_pong(self):
        s = self.open_ws()
        try:
            s.sendall(ws_frame(gs.PING, b"pi"))
            op, data = recv_frame(s)
            self.assertEqual((op, data), (gs.PONG, b"pi"))
        finally:
            s.close()

    def test_binary_broadcast(self):
        s = self.open_ws()
        try:
            s.sendall(ws_frame(gs.TEXT, json.dumps({"id": 1, "cmd": "schema"}).encode()))
            recv_frame(s)                                  # makes sure the client is registered
            dmx = bytes(range(256)) * 2
            self.srv.push("dmx", dmx, universe=7)
            op, data = recv_frame(s)
            self.assertEqual(op, gs.BINARY)
            self.assertEqual(len(data), 3 + 512)
            tid, uni = struct.unpack(">BH", data[:3])
            self.assertEqual((tid, uni), (gs.TOPICS["dmx"], 7))
            self.assertEqual(data[3:], dmx)

            self.srv.push("log", {"level": "info", "txt": "ok"})
            op, data = recv_frame(s)
            self.assertEqual(op, gs.TEXT)
            self.assertEqual(json.loads(data), {"topic": "log", "payload": {"level": "info", "txt": "ok"}})
        finally:
            s.close()


if __name__ == "__main__":
    unittest.main()
