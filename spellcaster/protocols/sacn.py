# sACN (ANSI E1.31): multicast output per interface + localhost unicast, input, discovery. Stdlib only.
# Common output interface (every protocol): send(universe:int, data:bytes) and close().
import socket, struct, threading, uuid

PORT = 5568
ROOT = struct.pack(">HH12s", 0x10, 0, b"ASC-E1.17\0\0\0")
DISCOVERY_IP = "239.255.250.214"


def mcast(universe):
    return f"239.255.{universe >> 8}.{universe & 255}"


def packet(universe, data, cid, seq, source_name="Spellcaster", priority=100):
    """E1.31 data packet (start code 0). Pure function: the same bytes as the MED GRUPO generator."""
    data = bytes(data)
    dmp = struct.pack(">HBBHHH", 0x7000 | (11 + len(data)), 0x02, 0xA1, 0, 1, 1 + len(data)) + b"\0" + data
    fr = struct.pack(">H", 0x7000 | (77 + len(dmp))) + struct.pack(">I", 2) \
        + source_name.encode("utf-8")[:63].ljust(64, b"\0") \
        + struct.pack(">BHBBH", priority, 0, seq, 0, universe) + dmp
    return ROOT + struct.pack(">H", 0x7000 | (22 + len(fr))) + struct.pack(">I", 4) + cid + fr


def parse(pk):
    """Returns a dict for a data packet (vector 4) or a discovery packet (vector 8); None if it is not E1.31."""
    if len(pk) < 48 or pk[:16] != ROOT:
        return None
    vec = struct.unpack(">I", pk[18:22])[0]
    cid = pk[22:38]
    name = pk[44:108].split(b"\0")[0].decode("utf-8", "replace")
    if vec == 4 and len(pk) >= 126:
        prio, _sync, seq, _opt, universe = struct.unpack(">BHBBH", pk[108:115])
        return {"kind": "data", "cid": cid, "name": name, "priority": prio, "seq": seq,
                "universe": universe, "data": pk[126:]}
    if vec == 8 and len(pk) >= 120 and struct.unpack(">I", pk[40:44])[0] == 2:
        # n from the declared length of the Universe Discovery PDU (8 + 2n), capped by what actually arrived
        n = max(0, min(((struct.unpack_from(">H", pk, 112)[0] & 0x0FFF) - 8) // 2, (len(pk) - 120) // 2))
        return {"kind": "discovery", "cid": cid, "name": name,
                "universes": list(struct.unpack(f">{n}H", pk[120:120 + 2 * n]))}
    return None


def interfaces():
    """Local IPv4 addresses (by hostname) + loopback."""
    return sorted({ai[4][0] for ai in socket.getaddrinfo(socket.gethostname(), None, socket.AF_INET)} | {"127.0.0.1"})


class SacnOut:
    def __init__(self, universes=(1,), priority=100, source_name="Spellcaster", interfaces=None):
        self.universes = tuple(universes)
        self.priority, self.source_name = priority, source_name
        self.cid = uuid.uuid4().bytes
        self.seq = {u: 0 for u in self.universes}
        self.ifaces = list(interfaces) if interfaces else globals()["interfaces"]()
        self.socks = []
        for ip in self.ifaces:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 1)
            s.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton(ip))
            self.socks.append(s)

    def send(self, universe, data):
        self.seq[universe] = seq = (self.seq.get(universe, 0) + 1) & 255
        pk = packet(universe, data, self.cid, seq, self.source_name, self.priority)
        for s in self.socks:
            try:
                s.sendto(pk, (mcast(universe), PORT))
            except OSError:
                pass
        self.socks[0].sendto(pk, ("127.0.0.1", PORT))

    def close(self):
        for s in self.socks:
            s.close()
        self.socks = []


def _listener(groups):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("", PORT))
    for g in groups:
        s.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, socket.inet_aton(g) + socket.inet_aton("0.0.0.0"))
    s.settimeout(0.2)
    return s


class SacnIn:
    """Listens to the universes and keeps the last frame of each one. get(universe) -> bytes | None."""

    def __init__(self, universes=(1,)):
        self.last = {}
        self.sources = {}                       # universe -> name of the last source seen
        self._sock = _listener([mcast(u) for u in universes])
        self._run = True
        self._th = threading.Thread(target=self._loop, daemon=True)
        self._th.start()

    def _loop(self):
        while self._run:
            try:
                pk, _ = self._sock.recvfrom(2048)
            except socket.timeout:
                continue
            except OSError:
                break
            p = parse(pk)
            if p and p["kind"] == "data":
                self.last[p["universe"]] = p["data"]
                self.sources[p["universe"]] = p["name"]

    def get(self, universe):
        return self.last.get(universe)

    def close(self):
        self._run = False
        self._sock.close()
