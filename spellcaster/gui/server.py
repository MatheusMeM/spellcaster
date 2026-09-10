# GUI server: http.server serves web/ and the /ws endpoint speaks WebSocket (RFC 6455 done by hand).
#
# /ws protocol
#   client -> server : text    {"id": <n>, "cmd": "<name>", "args": {...}}
#   server -> client : text    {"id": <n>, "result": ...} | {"id": <n>, "error": "Type: msg"}
#   cmd "schema" is intrinsic (it does not go through the registry) and returns registry.schema().
#   Everything else goes to registry.call(cmd, **args): the GUI implements no product logic.
#
# Broadcast (server -> everyone)
#   push(topic, payload)                 text    {"topic": ..., "payload": ...}
#   push(topic, bytes, universe=N)       binary  topic_id:u8 + universe:u16 + payload
#                                        (DMX = 512 bytes; topic ids in TOPICS)
import base64, hashlib, json, struct, threading, webbrowser
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler

from ..core import registry
from ..core.registry import command
from ..paths import WEB

GUID = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
TOPICS = {"dmx": 1}                                   # binary topic ids (mirrored in app.js)
TEXT, BINARY, CLOSE, PING, PONG = 1, 2, 8, 9, 10
SERVER = None                                         # last GuiServer created (api.py publishes on it)


def accept_key(key):
    """Sec-WebSocket-Accept = base64(sha1(key + GUID))."""
    return base64.b64encode(hashlib.sha1(key.encode() + GUID).digest()).decode()


def frame(op, data):
    """Server frame: FIN=1, unmasked (only the client masks)."""
    n = len(data)
    if n < 126:
        h = bytes((0x80 | op, n))
    elif n < 65536:
        h = bytes((0x80 | op, 126)) + struct.pack(">H", n)
    else:
        h = bytes((0x80 | op, 127)) + struct.pack(">Q", n)
    return h + data


def read_frame(rf):
    """Reads one client frame -> (opcode, payload); (None, None) on EOF. Unmasks it if it arrives masked."""
    h = rf.read(2)
    if len(h) < 2:
        return None, None
    op, n = h[0] & 0x0F, h[1] & 0x7F
    if n == 126:
        n = struct.unpack(">H", rf.read(2))[0]
    elif n == 127:
        n = struct.unpack(">Q", rf.read(8))[0]
    mask = rf.read(4) if h[1] & 0x80 else b""
    data = rf.read(n) if n else b""
    if mask:
        data = bytes(b ^ mask[i & 3] for i, b in enumerate(data))
    return op, data
    # ponytail: continuation frames (FIN=0) are not reassembled ; reassemble them if some client fragments


class GuiServer:
    def __init__(self, host="0.0.0.0", port=8000):
        global SERVER
        self.clients, self.lock = set(), threading.Lock()
        SERVER = self
        srv = self
        mime = {**SimpleHTTPRequestHandler.extensions_map, ".js": "text/javascript", ".css": "text/css",
                ".json": "application/json", ".svg": "image/svg+xml", ".html": "text/html"}

        class Handler(SimpleHTTPRequestHandler):
            extensions_map = mime
            protocol_version = "HTTP/1.1"

            def __init__(self, *a, **k):
                super().__init__(*a, directory=str(WEB), **k)

            def log_message(self, *a):
                pass

            def end_headers(self):
                self.send_header("Cache-Control", "no-store")   # editing a skin/JS and hitting F5 is enough
                super().end_headers()

            def do_GET(self):
                if self.path.split("?")[0] == "/ws" and "websocket" in self.headers.get("Upgrade", "").lower():
                    return srv._ws(self)
                super().do_GET()

        self.httpd = ThreadingHTTPServer((host, port), Handler)
        self.port = self.httpd.server_address[1]

    def start(self):
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
        return self

    def stop(self):
        with self.lock:
            for c in list(self.clients):
                try:
                    c.close()
                except OSError:
                    pass
            self.clients.clear()
        self.httpd.shutdown()
        self.httpd.server_close()

    def _send(self, sock, data):
        with self.lock:      # ponytail: one global send lock ; per-client lock if a slow client stalls the others
            sock.sendall(data)

    def push(self, topic, payload, universe=0):
        """Broadcast to every client. bytes -> binary frame with a topic header."""
        if isinstance(payload, (bytes, bytearray)):
            data = frame(BINARY, struct.pack(">BH", TOPICS[topic], universe) + bytes(payload))
        else:
            data = frame(TEXT, json.dumps({"topic": topic, "payload": payload}, default=str).encode())
        for c in list(self.clients):
            try:
                self._send(c, data)
            except OSError:
                self.clients.discard(c)

    def _ws(self, h):
        key = h.headers.get("Sec-WebSocket-Key")
        if not key:
            return h.send_error(400, "Sec-WebSocket-Key missing")
        h.close_connection = True
        h.send_response(101, "Switching Protocols")
        h.send_header("Upgrade", "websocket")
        h.send_header("Connection", "Upgrade")
        h.send_header("Sec-WebSocket-Accept", accept_key(key))
        h.end_headers()
        h.wfile.flush()
        sock = h.connection
        self.clients.add(sock)
        try:
            while True:
                op, data = read_frame(h.rfile)
                if op is None or op == CLOSE:
                    self._send(sock, frame(CLOSE, data[:2] if data else b""))
                    break
                if op == PING:
                    self._send(sock, frame(PONG, data))
                elif op == TEXT:
                    self._send(sock, frame(TEXT, self._handle(data).encode()))
        except OSError:
            pass
        finally:
            self.clients.discard(sock)

    def _handle(self, data):   # ponytail: the command runs on the client thread ; a thread per message if a long command stalls the connection
        mid = None
        try:
            m = json.loads(data)
            mid = m.get("id")
            if m["cmd"] == "schema":
                return json.dumps({"id": mid, "result": registry.schema()}, default=str)
            return json.dumps({"id": mid, "result": registry.call(m["cmd"], **(m.get("args") or {}))}, default=str)
        except Exception as e:
            return json.dumps({"id": mid, "error": f"{type(e).__name__}: {e}"})


@command
def serve(port: int = 8000, browser: bool = False):
    """Brings the web GUI up at http://0.0.0.0:<port> (WebSocket on /ws). --browser opens the browser."""
    srv = GuiServer(port=port).start()
    print(f"GUI at http://127.0.0.1:{srv.port}  (Ctrl+C quits)", flush=True)
    if browser:
        webbrowser.open(f"http://127.0.0.1:{srv.port}")
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        srv.stop()
    return srv


from . import api  # noqa: E402,F401  (registers show_*/track_*/key_*/transport/patch_check/profiles)

# Entry points: `python -m spellcaster.gui.window` (window, or browser without pywebview).
# `spell serve` starts existing once cli.py imports the gui package (one line: `from . import gui`);
# ponytail: cli.py was left alone in this round ; add the import along with the rest of F4.
