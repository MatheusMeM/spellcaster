# Regressão byte a byte contra o gerador do seed + loopback UDP.
import socket, struct, unittest, uuid

from spellcaster.protocols import sacn

CID = uuid.uuid4().bytes


def packet_seed(universe, data, seq):
    """Recorte de seed/show_medgrupo.py (seq passado em vez de global)."""
    dmp = struct.pack(">HBBHHH", 0x7000 | (11 + len(data)), 0x02, 0xA1, 0, 1, 1 + len(data)) + b"\0" + data
    fr = struct.pack(">H", 0x7000 | (77 + len(dmp))) + struct.pack(">I", 2) + b"Feiticaria show medgrupo".ljust(64, b"\0") \
         + struct.pack(">BHBBH", 100, 0, seq, 0, universe) + dmp
    return struct.pack(">HH12s", 0x10, 0, b"ASC-E1.17\0\0\0") + struct.pack(">H", 0x7000 | (22 + len(fr))) \
           + struct.pack(">I", 4) + CID + fr


class TestPacket(unittest.TestCase):
    def test_bytes_iguais_ao_seed(self):
        data = bytes(range(256)) * 2
        for seq, u in ((1, 1), (255, 1), (7, 300)):
            self.assertEqual(sacn.packet(u, data, CID, seq, "Feiticaria show medgrupo", 100), packet_seed(u, data, seq))

    def test_parse(self):
        data = bytes([9] * 512)
        p = sacn.parse(sacn.packet(3, data, CID, 42, "X", 50))
        self.assertEqual((p["kind"], p["universe"], p["seq"], p["priority"], p["name"], p["cid"]), ("data", 3, 42, 50, "X", CID))
        self.assertEqual(p["data"], data)


class TestLoopback(unittest.TestCase):
    def test_unicast_localhost(self):
        rx = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        rx.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            rx.bind(("127.0.0.1", sacn.PORT))
        except OSError as e:
            rx.close(); self.skipTest(f"porta 5568 ocupada: {e}")
        rx.settimeout(1.0)
        out = sacn.SacnOut(universes=(1,), interfaces=["127.0.0.1"])
        data = bytes(range(256)) * 2
        try:
            out.send(1, data)
            pk, _ = rx.recvfrom(2048)
        finally:
            out.close(); rx.close()
        self.assertEqual(pk, sacn.packet(1, data, out.cid, 1, "Spellcaster", 100))
        p = sacn.parse(pk)
        self.assertEqual((p["universe"], p["seq"], p["data"]), (1, 1, data))

    def test_seq_por_universo(self):
        out = sacn.SacnOut(universes=(1, 2), interfaces=["127.0.0.1"])
        try:
            out.send(1, b"\0"); out.send(1, b"\0"); out.send(2, b"\0")
        finally:
            out.close()
        self.assertEqual(out.seq, {1: 2, 2: 1})


if __name__ == "__main__":
    unittest.main()
