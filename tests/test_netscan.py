import struct
import unittest

from spellcaster.protocols import artnet, netscan, sacn

IPCONFIG_PTBR = """
Configuração de IP do Windows


Adaptador Ethernet Ethernet:

   Estado da mídia. . . . . . . . . . . . . .  : mídia desconectada
   Sufixo DNS específico de conexão. . . . . . :

Adaptador de Rede sem Fio Wi-Fi:

   Sufixo DNS específico de conexão. . . . . . : lan
   Endereço IPv6 de link local . . . . . . . . : fe80::cfcb:2c7e:3705:89a0%15
   Endereço IPv4. . . . . . . .  . . . . . . . : 192.168.0.132
   Máscara de Sub-rede . . . . . . . . . . . . : 255.255.255.0
   Gateway Padrão. . . . . . . . . . . . . . . : fe80::763a:efff:fe76:d266%15
                                                 192.168.0.1

Adaptador Ethernet Ethernet 2:

   Sufixo DNS específico de conexão. . . . . . :
   Endere�o IPv4. . . . . . . . . . . . . . . . : 2.0.0.10
   M�scara de Sub-rede . . . . . . . . . . . . : 255.0.0.0
   Gateway Padr�o. . . . . . . . . . . . . . . :
"""

IPCONFIG_EN = """
Windows IP Configuration


Ethernet adapter Ethernet 2:

   Connection-specific DNS Suffix  . :
   IPv4 Address. . . . . . . . . . . : 10.0.0.5(Preferred)
   Subnet Mask . . . . . . . . . . . : 255.0.0.0
   Default Gateway . . . . . . . . . :

Wireless LAN adapter Wi-Fi:

   Connection-specific DNS Suffix  . : home
   Link-local IPv6 Address . . . . . : fe80::1%12
   IPv4 Address. . . . . . . . . . . : 192.168.1.20
   Subnet Mask . . . . . . . . . . . : 255.255.255.0
   Default Gateway . . . . . . . . . : 192.168.1.1
"""


def artpollreply(ip="2.0.0.50", short=b"Node1", long_=b"Nodo de teste", net=0, sub=1, swout=(2, 3), swin=(5,)):
    p = bytearray(239)
    p[0:8] = b"Art-Net\0"
    struct.pack_into("<H", p, 8, 0x2100)
    p[10:14] = bytes(int(x) for x in ip.split("."))
    struct.pack_into("<H", p, 14, 6454)
    p[18], p[19] = net, sub
    p[26:26 + len(short)] = short
    p[44:44 + len(long_)] = long_
    n = max(len(swout), len(swin))
    struct.pack_into(">H", p, 172, n)
    for i in range(n):
        p[174 + i] = (0x80 if i < len(swout) else 0) | (0x40 if i < len(swin) else 0)
    p[186:186 + len(swin)] = bytes(swin)
    p[190:190 + len(swout)] = bytes(swout)
    p[201:207] = bytes.fromhex("aabbccddeeff")
    return bytes(p)


def sacn_discovery(name=b"Fonte X", universes=(1, 2, 10)):
    p = bytearray(120)
    struct.pack_into(">HH", p, 0, 0x10, 0)
    p[4:16] = b"ASC-E1.17\0\0\0"
    struct.pack_into(">I", p, 18, 8)
    p[22:38] = bytes(range(16))
    struct.pack_into(">I", p, 40, 2)
    p[44:44 + len(name)] = name
    struct.pack_into(">H", p, 112, 0x7000 | (8 + 2 * len(universes)))
    struct.pack_into(">I", p, 114, 1)
    return bytes(p) + struct.pack(f">{len(universes)}H", *universes)


class TestParse(unittest.TestCase):
    def test_ipconfig_ptbr(self):
        ifs = netscan.parse_ipconfig(IPCONFIG_PTBR)
        self.assertEqual([i["name"] for i in ifs], ["Wi-Fi", "Ethernet 2"])
        self.assertEqual(ifs[0]["ip"], "192.168.0.132")
        self.assertEqual(ifs[0]["mask"], "255.255.255.0")
        self.assertEqual(ifs[0]["gateway"], "192.168.0.1")  # IPv4 na linha depois do IPv6
        self.assertEqual(ifs[1], {"name": "Ethernet 2", "ip": "2.0.0.10", "mask": "255.0.0.0", "gateway": None})

    def test_ipconfig_en(self):
        ifs = netscan.parse_ipconfig(IPCONFIG_EN)
        self.assertEqual(ifs[0], {"name": "Ethernet 2", "ip": "10.0.0.5", "mask": "255.0.0.0", "gateway": None})
        self.assertEqual(ifs[1]["name"], "Wi-Fi")
        self.assertEqual(ifs[1]["gateway"], "192.168.1.1")

    def test_ip_addr_linux(self):
        txt = "1: lo: <LOOPBACK>\n    inet 127.0.0.1/8 scope host lo\n2: eth0: <UP>\n    inet 10.1.2.3/24 brd 10.1.2.255\n"
        ifs = netscan.parse_ip_addr(txt, "default via 10.1.2.1 dev eth0\n")
        self.assertEqual(ifs[1], {"name": "eth0", "ip": "10.1.2.3", "mask": "255.255.255.0", "gateway": "10.1.2.1"})

    def test_artpollreply(self):
        r = artnet.parse(artpollreply())
        self.assertEqual(r["ip"], "2.0.0.50")
        self.assertEqual(r["short_name"], "Node1")
        self.assertEqual(r["long_name"], "Nodo de teste")
        self.assertEqual(r["mac"], "aa:bb:cc:dd:ee:ff")
        self.assertEqual(r["ports"], [{"dir": "out", "universe": 0x12}, {"dir": "in", "universe": 0x15},
                                      {"dir": "out", "universe": 0x13}])
        self.assertEqual(artnet.parse(netscan.ARTPOLL)["op"], "ArtPoll")

    def test_sacn_discovery(self):
        r = sacn.parse(sacn_discovery())
        self.assertEqual((r["kind"], r["name"]), ("discovery", "Fonte X"))
        self.assertEqual(r["universes"], [1, 2, 10])
        # o n sai do comprimento do PDU, nao do datagrama: padding depois dos universos e ignorado
        self.assertEqual(sacn.parse(sacn_discovery() + bytes(8))["universes"], [1, 2, 10])
        self.assertIsNone(sacn.parse(b"x" * 200))


class TestSuggest(unittest.TestCase):
    def test_rules(self):
        ifs = [{"name": "Wi-Fi", "ip": "192.168.0.132", "mask": "255.255.255.0", "gateway": "192.168.0.1"},
               {"name": "Ethernet", "ip": "192.168.0.7", "mask": "255.255.255.0", "gateway": None},
               {"name": "Laser", "ip": "2.0.0.10", "mask": "255.0.0.0", "gateway": None}]
        s = netscan.suggest(ifs, windows=True)
        txt = "\n".join(s)
        self.assertIn("Laser 2.0.0.10/255.0.0.0: Art-Net ok", txt)
        self.assertIn('netsh interface ip set address name="Wi-Fi" static 2.0.0.132 255.0.0.0', txt)
        self.assertIn("Wi-Fi, Ethernet na mesma subrede 192.168.0.0", txt)
        self.assertNotIn("netsh", "\n".join(netscan.suggest(ifs, windows=False)))
        self.assertEqual(len(netscan.suggest([])), 1)


class TestReport(unittest.TestCase):
    def test_report_text(self):
        d = {"interfaces": [{"name": "Wi-Fi", "ip": "192.168.0.132", "mask": "255.255.255.0", "gateway": "192.168.0.1"}],
             "suggestions": ["x"], "artnet": [artnet.parse(artpollreply())],
             "sacn": {"error": "porta ocupada"}, "etherdream": []}
        txt = netscan.report(d)
        self.assertIn("2.0.0.50  'Node1'", txt)
        self.assertIn("erro: porta ocupada", txt)
        self.assertIn("(nada encontrado)", txt)


if __name__ == "__main__":
    unittest.main()
