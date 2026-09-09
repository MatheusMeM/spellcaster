"""Art-Net 4 (Art-Net 4 spec, Artistic Licence): ArtDmx out/in, ArtPoll/ArtPollReply, ArtSync.

Universos no Spellcaster são 1-based (como sACN). Conversão para Art-Net:
    port_address = universe - 1   (15 bits: net[7] | subnet[4] | universe[4])
Ex.: universe 1 -> port-address 0 (net 0, subnet 0, uni 0); universe 17 -> subnet 1, uni 0.
"""
import socket, struct, threading

PORT = 6454
HEADER = b"Art-Net\x00"
PROT_VER = 14
OP_POLL, OP_POLL_REPLY, OP_DMX, OP_SYNC = 0x2000, 0x2100, 0x5000, 0x5200
BROADCASTS = ("2.255.255.255", "10.255.255.255", "255.255.255.255")


def port_address(universe):
    """universe 1-based -> port-address 15 bits."""
    pa = universe - 1
    if not 0 <= pa <= 0x7FFF:
        raise ValueError(f"universo fora de faixa: {universe}")
    return pa


def artdmx(universe, data, sequence, physical=0):
    """Pacote ArtDmx. data: 2..512 bytes, comprimento par (Art-Net exige)."""
    data = bytes(data)
    if len(data) % 2:
        data += b"\x00"
    data = data[:512] or b"\x00\x00"
    pa = port_address(universe)
    # OpCode little-endian, ProtVer big-endian, SubUni/Net, Length big-endian
    return HEADER + struct.pack("<H", OP_DMX) + struct.pack(">HBBBBH", PROT_VER, sequence & 0xFF, physical,
                                                              pa & 0xFF, pa >> 8, len(data)) + data


def artpoll(flags=0x06, priority=0x10):
    return HEADER + struct.pack("<H", OP_POLL) + struct.pack(">HBB", PROT_VER, flags, priority)


def artsync():
    return HEADER + struct.pack("<H", OP_SYNC) + struct.pack(">HBB", PROT_VER, 0, 0)


def parse(packet):
    """Decodifica ArtDmx / ArtPoll / ArtPollReply / ArtSync -> dict. None se não for Art-Net."""
    if len(packet) < 10 or packet[:8] != HEADER:
        return None
    op = struct.unpack_from("<H", packet, 8)[0]
    if op == OP_DMX and len(packet) >= 18:
        seq, phys, sub, net, length = struct.unpack_from(">BBBBH", packet, 12)
        pa = (net << 8) | sub
        return {"op": "ArtDmx", "universe": pa + 1, "port_address": pa, "sequence": seq,
                "physical": phys, "data": packet[18:18 + length]}
    if op == OP_POLL:
        return {"op": "ArtPoll", "flags": packet[12] if len(packet) > 12 else 0}
    if op == OP_SYNC:
        return {"op": "ArtSync"}
    if op == OP_POLL_REPLY and len(packet) >= 194:
        net, sub = packet[18], packet[19]
        types, swin, swout = packet[174:178], packet[186:190], packet[190:194]
        ports = []                                       # port-address por porta, com a direcao do no
        for i in range(min(struct.unpack_from(">H", packet, 172)[0], 4)):
            if types[i] & 0x80:
                ports.append({"dir": "out", "universe": net << 8 | sub << 4 | swout[i] & 0xF})
            if types[i] & 0x40:
                ports.append({"dir": "in", "universe": net << 8 | sub << 4 | swin[i] & 0xF})
        return {"op": "ArtPollReply", "ip": ".".join(map(str, packet[10:14])),
                "port": struct.unpack_from("<H", packet, 14)[0],
                "short_name": packet[26:44].split(b"\x00")[0].decode("latin-1"),
                "long_name": packet[44:108].split(b"\x00")[0].decode("latin-1"),
                "mac": packet[201:207].hex(":") if len(packet) >= 207 else None, "ports": ports}
    return {"op": f"0x{op:04x}"}


class ArtNetOut:
    """send(universe, data) por broadcast (2.x, 10.x, limited) ou unicast para `targets`."""

    def __init__(self, targets=None, broadcast=True, port=PORT):
        self.targets = [(t, port) for t in (targets or [])]
        if broadcast and not targets:
            self.targets = [(b, port) for b in BROADCASTS]
        self.seq = {}
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)

    def _tx(self, pkt):
        for t in self.targets:
            try:
                self.sock.sendto(pkt, t)
            except OSError:
                pass  # rede sem rota para 2.x/10.x: ignora

    def send(self, universe, data):
        s = self.seq.get(universe, 0) % 255 + 1  # 1..255, 0 = sem sequência
        self.seq[universe] = s
        self._tx(artdmx(universe, data, s))

    def close(self):
        self.sock.close()


class ArtNetIn(threading.Thread):
    """Escuta 6454 e guarda o último ArtDmx por universo em `frames`."""

    def __init__(self, universes, host="0.0.0.0", port=PORT):
        super().__init__(daemon=True)
        self.universes = set(universes)
        self.frames = {}
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind((host, port))
        self.sock.settimeout(0.5)
        self._stop = threading.Event()
        self.start()

    def run(self):
        while not self._stop.is_set():
            try:
                pkt, _ = self.sock.recvfrom(1024)
            except socket.timeout:
                continue
            except OSError:
                break
            p = parse(pkt)
            if p and p["op"] == "ArtDmx" and p["universe"] in self.universes:
                self.frames[p["universe"]] = p["data"]

    def close(self):
        self._stop.set()
        self.sock.close()
