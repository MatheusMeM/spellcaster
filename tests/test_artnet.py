import socket, struct, time, unittest
from spellcaster.protocols import artnet


class ArtDmxTest(unittest.TestCase):
    def test_roundtrip(self):
        data = bytes(range(256)) * 2
        pkt = artnet.artdmx(17, data, 42)
        self.assertEqual(pkt[:8], b"Art-Net\x00")
        self.assertEqual(pkt[8:10], b"\x00\x50")        # OpCode 0x5000 little-endian
        self.assertEqual(pkt[10:12], b"\x00\x0e")       # ProtVer 14
        self.assertEqual((pkt[14], pkt[15]), (16, 0))   # universe 17 -> port-address 16 (subnet 1, uni 0)
        p = artnet.parse(pkt)
        self.assertEqual((p["op"], p["universe"], p["sequence"], p["data"]), ("ArtDmx", 17, 42, data))

    def test_odd_length_padded(self):
        p = artnet.parse(artnet.artdmx(1, b"\x01\x02\x03", 1))
        self.assertEqual(p["data"], b"\x01\x02\x03\x00")

    def test_port_address_range(self):
        self.assertEqual(artnet.port_address(32768), 0x7FFF)
        with self.assertRaises(ValueError):
            artnet.port_address(0)

    def test_loopback(self):
        rx = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        rx.bind(("127.0.0.1", 0))
        rx.settimeout(2)
        out = artnet.ArtNetOut(targets=["127.0.0.1"], port=rx.getsockname()[1])
        out.send(3, b"\x10\x20\x30\x40")
        out.send(3, b"\x10\x20\x30\x40")
        p1 = artnet.parse(rx.recv(1024)); p2 = artnet.parse(rx.recv(1024))
        out.close(); rx.close()
        self.assertEqual((p1["universe"], p1["data"]), (3, b"\x10\x20\x30\x40"))
        self.assertEqual((p1["sequence"], p2["sequence"]), (1, 2))

    def test_artnetin(self):
        rx = artnet.ArtNetIn([5], host="127.0.0.1", port=0)
        port = rx.sock.getsockname()[1]
        out = artnet.ArtNetOut(targets=["127.0.0.1"], port=port)
        out.send(5, b"\xaa\xbb"); out.send(6, b"\xcc\xdd")
        for _ in range(40):
            if 5 in rx.frames: break
            time.sleep(0.05)
        out.close(); rx.close()
        self.assertEqual(rx.frames, {5: b"\xaa\xbb"})

    def test_pollreply(self):
        pkt = bytearray(239)
        pkt[:8] = b"Art-Net\x00"
        pkt[8:10] = struct.pack("<H", 0x2100)
        pkt[10:14] = bytes([2, 0, 0, 7]); pkt[14:16] = struct.pack("<H", 6454)
        pkt[18], pkt[19] = 1, 2                      # net 1, subnet 2
        pkt[26:31] = b"Node1"; pkt[44:53] = b"Node Long"
        pkt[172:174] = struct.pack(">H", 2)
        pkt[174:176] = bytes([0x80, 0xC0])           # port 0 output only, port 1 output and input
        pkt[186:190] = bytes([9, 8, 0, 0])           # SwIn
        pkt[190:194] = bytes([3, 4, 0, 0])           # SwOut
        pkt[201:207] = bytes.fromhex("0a1b2c3d4e5f")
        p = artnet.parse(bytes(pkt))
        self.assertEqual(p["op"], "ArtPollReply")
        self.assertEqual((p["ip"], p["short_name"], p["long_name"], p["mac"]),
                         ("2.0.0.7", "Node1", "Node Long", "0a:1b:2c:3d:4e:5f"))
        self.assertEqual(p["port"], 6454)
        self.assertEqual(p["ports"], [{"dir": "out", "universe": 0x123}, {"dir": "out", "universe": 0x124},
                                      {"dir": "in", "universe": 0x128}])

    def test_poll_parse(self):
        self.assertEqual(artnet.parse(artnet.artpoll())["op"], "ArtPoll")
        self.assertEqual(artnet.parse(artnet.artsync())["op"], "ArtSync")
        self.assertIsNone(artnet.parse(b"nope"))


if __name__ == "__main__":
    unittest.main()
