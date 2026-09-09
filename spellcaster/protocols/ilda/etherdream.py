"""Ether Dream DAC: beacon UDP 7654 (1 Hz), stream de pontos TCP 7765. Tudo little-endian.
Fluxo: connect -> 'p' prepare -> 'd' data -> 'b' begin -> 'd' data enquanto o buffer estiver abaixo da capacidade."""
import socket
import struct
import threading
import time

BEACON_PORT, TCP_PORT = 7654, 7765
STATUS = struct.Struct("<BBBBHHHHII")  # 20 bytes
STATUS_KEYS = ("protocol", "light_engine_state", "playback_state", "source", "light_engine_flags",
               "playback_flags", "source_flags", "buffer_fullness", "point_rate", "point_count")
BEACON = struct.Struct("<6sHHHI")  # + STATUS = 36 bytes
POINT = struct.Struct("<HhhHHHHHH")  # control x y r g b i u1 u2 = 18 bytes
BEGIN = struct.Struct("<cHI")  # 'b' low_water point_rate
RESP_LEN = 2 + STATUS.size  # 22
PLAYBACK = {0: "idle", 1: "prepared", 2: "playing"}


def parse_status(b):
    return dict(zip(STATUS_KEYS, STATUS.unpack_from(b)))


def parse_response(b):
    """22 bytes: ack ('a' ok, 'F' cheio, 'I' invalido, '!' stop), comando ecoado, status."""
    if len(b) < RESP_LEN:
        raise ValueError(f"resposta curta: {len(b)} bytes")
    return {"ack": chr(b[0]), "command": chr(b[1]), "status": parse_status(b[2:])}


def parse_beacon(b):
    mac, hw, sw, cap, rate = BEACON.unpack_from(b)
    return {"mac": mac.hex(":"), "hw_rev": hw, "sw_rev": sw, "buffer_capacity": cap,
            "max_point_rate": rate, "status": parse_status(b[BEACON.size:])}


def encode_data(points):
    """Comando 'd': pontos Point (x, y, r, g, b, blank) ou tuplas (x, y, r, g, b). Cores 0-255 -> 0-65535."""
    out = [b"d", struct.pack("<H", len(points))]
    for p in points:
        if hasattr(p, "blank"):
            x, y, r, g, b = p.x, p.y, p.r, p.g, p.b
            if p.blank:
                r = g = b = 0
        else:
            x, y, r, g, b = p[:5]
        out.append(POINT.pack(0, x, y, r * 257, g * 257, b * 257, max(r, g, b) * 257, 0, 0))
    return b"".join(out)


class EtherDream:
    def __init__(self, ip, port=TCP_PORT, capacity=1800):
        self.ip, self.port, self.capacity = ip, port, capacity
        self.sock = None
        self.status = None  # ultimo dac_status recebido
        self._run = False
        self._thread = None

    def connect(self, timeout=2):
        self.sock = socket.create_connection((self.ip, self.port), timeout)
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        return self._read()  # o DAC manda um status ao conectar

    def _read(self):
        buf = b""
        while len(buf) < RESP_LEN:
            chunk = self.sock.recv(RESP_LEN - len(buf))
            if not chunk:
                raise ConnectionError("Ether Dream fechou a conexao")
            buf += chunk
        r = parse_response(buf)
        self.status = r["status"]
        return r

    def cmd(self, data):
        self.sock.sendall(data)
        return self._read()

    def prepare(self):
        return self.cmd(b"p")

    def begin(self, pps, low_water=0):
        return self.cmd(BEGIN.pack(b"b", low_water, pps))

    def send(self, points):
        return self.cmd(encode_data(points))

    def ping(self):
        return self.cmd(b"?")

    def play(self, frames, pps=20000, chunk=None):
        """Toca frames (iteravel de Frame ou listas de pontos) em thread; para quando acabar ou em stop()."""
        chunk = chunk or max(1, pps // 50)  # ~20 ms de pontos por comando
        self._run = True
        self._thread = threading.Thread(target=self._loop, args=(frames, pps, chunk), daemon=True)
        self._thread.start()
        return self._thread

    def _loop(self, frames, pps, chunk):
        if self.status and self.status["playback_state"] != 0:
            self.cmd(b"s")
        self.prepare()
        begun = False
        for fr in frames:
            pts = getattr(fr, "points", fr)
            for i in range(0, len(pts), chunk):
                block = pts[i:i + chunk]
                # ponytail: polling do buffer_fullness a cada ack ; trocar por low_water medido em DAC real
                while self._run and self.status["buffer_fullness"] + len(block) > self.capacity:
                    time.sleep(len(block) / pps / 2)
                    self.ping()
                if not self._run:
                    return
                r = self.send(block)
                if r["ack"] != "a":
                    raise RuntimeError(f"Ether Dream respondeu {r['ack']!r} ao comando d")
                if not begun or self.status["playback_state"] == 0:  # inicio ou underrun
                    self.begin(pps)
                    begun = True
        self._run = False

    def stop(self):
        self._run = False
        if self._thread and self._thread is not threading.current_thread():
            self._thread.join(2)
        if self.sock:
            self.cmd(b"s")

    def close(self):
        if self.sock:
            try:
                self.stop()
            finally:
                self.sock.close()
                self.sock = None


class Emulator:
    """Servidor TCP minimo que responde ack e simula o buffer; para testes sem DAC.
    received = pontos recebidos (tuplas control,x,y,r,g,b,i,u1,u2); commands = bytes de comando na ordem."""

    def __init__(self, host="127.0.0.1", port=0, capacity=1800):
        self.capacity = capacity
        self.received, self.commands = [], []
        self.state = dict.fromkeys(STATUS_KEYS, 0)
        self.state["protocol"] = 1
        self._srv = socket.socket()
        self._srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._srv.bind((host, port))
        self._srv.listen(1)
        self._srv.settimeout(0.2)
        self.host, self.port = self._srv.getsockname()
        self._t = time.monotonic()
        self._run = False
        self._thread = threading.Thread(target=self._serve, daemon=True)

    def start(self):
        self._run = True
        self._thread.start()
        return self

    def stop(self):
        self._run = False
        self._thread.join(2)
        self._srv.close()

    def beacon(self, mac=b"\x01\x02\x03\x04\x05\x06"):
        return BEACON.pack(mac, 1, 2, self.capacity, 100000) + STATUS.pack(*self.state.values())

    def _resp(self, ack, cmd):
        return ack + cmd + STATUS.pack(*self.state.values())

    def _drain(self):
        """Consome pontos pelo tempo decorrido quando tocando; buffer vazio = underrun -> idle."""
        now, st = time.monotonic(), self.state
        if st["playback_state"] == 2:
            st["buffer_fullness"] = max(0, st["buffer_fullness"] - int((now - self._t) * st["point_rate"]))
            if st["buffer_fullness"] == 0:
                st["playback_state"] = 0
        self._t = now

    def _serve(self):
        while self._run:
            try:
                conn, _ = self._srv.accept()
            except socket.timeout:
                continue
            except OSError:
                return
            with conn:
                self._handle(conn)

    def _handle(self, conn):
        conn.sendall(self._resp(b"a", b"?"))
        f = conn.makefile("rb")
        st = self.state
        while self._run:
            c = f.read(1)
            if not c:
                return
            self._drain()
            ack = b"a"
            if c == b"p":
                st["playback_state"] = 1
            elif c == b"b":
                _lw, st["point_rate"] = struct.unpack("<HI", f.read(6))
                st["playback_state"] = 2
            elif c == b"q":
                st["point_rate"], = struct.unpack("<I", f.read(4))
            elif c == b"d":
                n, = struct.unpack("<H", f.read(2))
                raw = f.read(POINT.size * n)
                if st["buffer_fullness"] + n > self.capacity:
                    ack = b"F"
                else:
                    self.received += [POINT.unpack_from(raw, i * POINT.size) for i in range(n)]
                    st["buffer_fullness"] += n
                    st["point_count"] += n
            elif c == b"s":
                st["playback_state"] = 0
                st["buffer_fullness"] = 0
            elif c in (b"0", b"\0"):
                st["playback_state"] = 0
                st["light_engine_state"] = 3  # e-stop
            elif c == b"c":
                st["light_engine_state"] = 0
            elif c != b"?":
                ack = b"I"
            self.commands.append(c)
            try:
                conn.sendall(self._resp(ack, c))
            except OSError:
                return
