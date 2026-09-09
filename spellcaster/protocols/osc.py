"""OSC 1.0 sobre UDP: encode/decode de mensagens e bundles, OscOut, OscIn com pattern matching."""
import re, socket, struct, threading

IMMEDIATE = 1  # timetag "agora"


def _pad(b):
    return b + b"\x00" * (-len(b) % 4)


def _str(s):
    return _pad(s.encode("utf-8") + b"\x00")


class Blob(bytes):
    """bytes marcados como blob OSC (tag b)."""


def timetag(t=None):
    """time.time() -> NTP 64 bits (segundos desde 1900 << 32 | fração)."""
    if t is None:
        return IMMEDIATE
    sec, frac = divmod(t + 2208988800, 1)
    return (int(sec) << 32) | int(frac * (1 << 32))


def _arg(v):
    """valor Python -> (tag, bytes)."""
    if v is True: return "T", b""
    if v is False: return "F", b""
    if v is None: return "N", b""
    if v is Ellipsis: return "I", b""  # impulso
    if isinstance(v, (bytes, bytearray)): return "b", _pad(struct.pack(">i", len(v)) + bytes(v))
    if isinstance(v, int):
        return ("i", struct.pack(">i", v)) if -2**31 <= v < 2**31 else ("h", struct.pack(">q", v))
    if isinstance(v, float): return "f", struct.pack(">f", v)
    if isinstance(v, str): return "s", _str(v)
    if isinstance(v, tuple) and len(v) == 2 and v[0] in ("d", "h", "t"):  # tipo explícito: ("d", 1.5), ("h", 7), ("t", tag)
        return v[0], struct.pack({"d": ">d", "h": ">q", "t": ">Q"}[v[0]], v[1])
    raise TypeError(f"tipo OSC nao suportado: {v!r}")


def message(address, *args):
    tags, body = ",", b""
    for a in args:
        t, b = _arg(a)
        tags += t
        body += b
    return _str(address) + _str(tags) + body


def bundle(elements, tt=IMMEDIATE):
    """elements: lista de bytes já codificados (message/bundle)."""
    out = b"#bundle\x00" + struct.pack(">Q", tt)
    for e in elements:
        out += struct.pack(">i", len(e)) + e
    return out


def _rstr(data, i):
    end = data.index(b"\x00", i)
    return data[i:end].decode("utf-8"), end + 1 + (-(end + 1) % 4)


def parse(data):
    """bytes -> (address, [args]) ou ("#bundle", timetag, [elementos parseados])."""
    if data[:8] == b"#bundle\x00":
        tt = struct.unpack_from(">Q", data, 8)[0]
        i, elems = 16, []
        while i < len(data):
            n = struct.unpack_from(">i", data, i)[0]
            elems.append(parse(data[i + 4:i + 4 + n]))
            i += 4 + n
        return "#bundle", tt, elems
    addr, i = _rstr(data, 0)
    tags, i = _rstr(data, i) if data[i:i + 1] == b"," else (",", i)
    args = []
    for t in tags[1:]:
        if t == "i": args.append(struct.unpack_from(">i", data, i)[0]); i += 4
        elif t == "f": args.append(struct.unpack_from(">f", data, i)[0]); i += 4
        elif t == "d": args.append(struct.unpack_from(">d", data, i)[0]); i += 8
        elif t == "h": args.append(struct.unpack_from(">q", data, i)[0]); i += 8
        elif t == "t": args.append(struct.unpack_from(">Q", data, i)[0]); i += 8
        elif t == "s": s, i = _rstr(data, i); args.append(s)
        elif t == "b":
            n = struct.unpack_from(">i", data, i)[0]
            args.append(Blob(data[i + 4:i + 4 + n])); i += 4 + n + (-n % 4)
        elif t == "T": args.append(True)
        elif t == "F": args.append(False)
        elif t == "N": args.append(None)
        elif t == "I": args.append(Ellipsis)
        else: raise ValueError(f"tag OSC desconhecida: {t}")
    return addr, args


def pattern_re(pattern):
    """Padrão de endereço OSC -> regex compilada. Suporta * ? [a-z] [!a-z] {a,b}."""
    out, i = "", 0
    while i < len(pattern):
        c = pattern[i]
        if c == "*": out += "[^/]*"
        elif c == "?": out += "[^/]"
        elif c == "[":
            j = pattern.index("]", i)
            body = pattern[i + 1:j]
            if body.startswith("!"): body = "^" + body[1:]
            out += "[" + body.replace("\\", "\\\\") + "]"
            i = j
        elif c == "{":
            j = pattern.index("}", i)
            out += "(" + "|".join(re.escape(x) for x in pattern[i + 1:j].split(",")) + ")"
            i = j
        else: out += re.escape(c)
        i += 1
    return re.compile("^" + out + "$")


class OscOut:
    def __init__(self, host, port):
        self.addr = (host, port)
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)

    def send(self, address, *args):
        self.sock.sendto(message(address, *args), self.addr)

    def bundle(self, msgs, tt=IMMEDIATE):
        """msgs: lista de tuplas (address, *args) ou bytes já codificados."""
        self.sock.sendto(bundle([m if isinstance(m, bytes) else message(*m) for m in msgs], tt), self.addr)

    def close(self):
        self.sock.close()


class OscIn(threading.Thread):
    """Escuta UDP; on(pattern, fn) chama fn(address, *args) para cada mensagem casada."""

    def __init__(self, port, host="0.0.0.0"):
        super().__init__(daemon=True)
        self.handlers = []
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind((host, port))
        self.sock.settimeout(0.5)
        self._stop = threading.Event()
        self.start()

    def on(self, pattern, fn):
        self.handlers.append((pattern_re(pattern), fn))

    def _dispatch(self, p):
        if p[0] == "#bundle":
            for e in p[2]:  # ponytail: timetag futuro ignorado, executa já ; agendar pelo Clock em F2
                self._dispatch(e)
            return
        addr, args = p
        for rx, fn in self.handlers:
            if rx.match(addr):
                fn(addr, *args)

    def run(self):
        while not self._stop.is_set():
            try:
                data, _ = self.sock.recvfrom(65536)
            except socket.timeout:
                continue
            except OSError:
                break
            try:
                self._dispatch(parse(data))
            except (ValueError, struct.error, IndexError, UnicodeDecodeError):
                pass  # pacote malformado: descarta

    def close(self):
        self._stop.set()
        self.sock.close()
