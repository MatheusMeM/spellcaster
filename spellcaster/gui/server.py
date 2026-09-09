# Servidor da GUI: http.server serve web/ e o endpoint /ws fala WebSocket (RFC 6455 feito a mao).
#
# Protocolo /ws
#   cliente -> servidor : texto  {"id": <n>, "cmd": "<nome>", "args": {...}}
#   servidor -> cliente : texto  {"id": <n>, "result": ...} | {"id": <n>, "error": "Tipo: msg"}
#   cmd "schema" e intrinseco (nao passa pelo registry) e devolve registry.schema().
#   Todo o resto vai para registry.call(cmd, **args): a GUI nao implementa logica de produto.
#
# Broadcast (servidor -> todos)
#   push(topic, payload)                 texto  {"topic": ..., "payload": ...}
#   push(topic, bytes, universe=N)       binario topic_id:u8 + universe:u16 + payload
#                                        (DMX = 512 bytes; ids dos topicos em TOPICS)
import base64, hashlib, json, struct, threading, webbrowser
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

from ..core import registry
from ..core.registry import command

WEB = Path(__file__).parent / "web"
GUID = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
TOPICS = {"dmx": 1}                                   # ids dos topicos binarios (espelhado em app.js)
TEXT, BINARY, CLOSE, PING, PONG = 1, 2, 8, 9, 10


def accept_key(key):
    """Sec-WebSocket-Accept = base64(sha1(chave + GUID))."""
    return base64.b64encode(hashlib.sha1(key.encode() + GUID).digest()).decode()


def frame(op, data):
    """Frame do servidor: FIN=1, sem mascara (so o cliente mascara)."""
    n = len(data)
    if n < 126:
        h = bytes((0x80 | op, n))
    elif n < 65536:
        h = bytes((0x80 | op, 126)) + struct.pack(">H", n)
    else:
        h = bytes((0x80 | op, 127)) + struct.pack(">Q", n)
    return h + data


def read_frame(rf):
    """Le um frame do cliente -> (opcode, payload); (None, None) em EOF. Desmascara se vier mascarado."""
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
    # ponytail: frames de continuacao (FIN=0) nao sao remontados ; remontar se algum cliente fragmentar


class GuiServer:
    def __init__(self, host="0.0.0.0", port=8000):
        self.clients, self.lock = set(), threading.Lock()
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
                self.send_header("Cache-Control", "no-store")   # editar skin/JS e dar F5 basta
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
        with self.lock:      # ponytail: um lock global de envio ; lock por cliente se um cliente lento travar os outros
            sock.sendall(data)

    def push(self, topic, payload, universe=0):
        """Broadcast para todos os clientes. bytes -> frame binario com cabecalho de topico."""
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
            return h.send_error(400, "Sec-WebSocket-Key ausente")
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

    def _handle(self, data):   # ponytail: comando roda na thread do cliente ; thread por mensagem se um comando longo travar a conexao
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
    """Sobe a GUI web em http://0.0.0.0:<port> (WebSocket em /ws). --browser abre o navegador."""
    srv = GuiServer(port=port).start()
    print(f"GUI em http://127.0.0.1:{srv.port}  (Ctrl+C encerra)", flush=True)
    if browser:
        webbrowser.open(f"http://127.0.0.1:{srv.port}")
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        srv.stop()
    return srv


# Entradas: `python -m spellcaster.gui.window` (janela, ou navegador sem pywebview).
# `spell serve` passa a existir quando cli.py importar o pacote gui (uma linha: `from . import gui`);
# ponytail: nao mexi em cli.py nesta rodada ; adicionar o import junto do resto do F4.
