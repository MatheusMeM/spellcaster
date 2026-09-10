import struct, time, unittest
from spellcaster.protocols import osc


class EncodeTest(unittest.TestCase):
    def test_all_types(self):
        tt = osc.timetag(1_700_000_000.5)
        multibyte = "caf\u00e9"                      # UTF-8 multi-byte, written as an escape: this file stays ASCII
        args = [1, -2, 3.5, multibyte, osc.Blob(b"\x01\x02\x03"), True, False, None, Ellipsis,
                ("d", 2.25), ("h", 2**40), ("t", tt), 2**40]
        addr, got = osc.parse(osc.message("/x/y", *args))
        self.assertEqual(addr, "/x/y")
        self.assertEqual(got, [1, -2, 3.5, multibyte, b"\x01\x02\x03", True, False, None, Ellipsis,
                               2.25, 2**40, tt, 2**40])
        self.assertIsInstance(got[4], osc.Blob)

    def test_alignment(self):
        for a in ("/a", "/ab", "/abc", "/abcd"):
            for s in ("", "x", "xyz", "wxyz"):
                m = osc.message(a, s, osc.Blob(b"12345"))
                self.assertEqual(len(m) % 4, 0, (a, s))
                self.assertEqual(osc.parse(m), (a, [s, b"12345"]))
        self.assertEqual(osc.message("/abc", 1), b"/abc\x00\x00\x00\x00,i\x00\x00\x00\x00\x00\x01")

    def test_no_typetag(self):
        self.assertEqual(osc.parse(b"/go\x00"), ("/go", []))

    def test_bundle(self):
        b = osc.bundle([osc.message("/a", 1), osc.bundle([osc.message("/b", "z")])])
        self.assertEqual(b[:8], b"#bundle\x00")
        self.assertEqual(struct.unpack_from(">Q", b, 8)[0], 1)
        self.assertEqual(osc.parse(b), ("#bundle", 1, [("/a", [1]), ("#bundle", 1, [("/b", ["z"])])]))

    def test_timetag(self):
        self.assertEqual(osc.timetag(-2208988800), 0)  # 1900-01-01
        self.assertEqual(osc.timetag(0) >> 32, 2208988800)
        self.assertEqual(osc.timetag(), 1)


class PatternTest(unittest.TestCase):
    def test_match(self):
        cases = [("/a/*", "/a/xyz", True), ("/a/*", "/a/x/y", False), ("/a/?", "/a/x", True), ("/a/?", "/a/xy", False),
                 ("/fx[0-9]", "/fx3", True), ("/fx[0-9]", "/fxa", False), ("/fx[!0-9]", "/fxa", True),
                 ("/{play,stop}", "/play", True), ("/{play,stop}", "/pause", False), ("/a.b", "/axb", False),
                 ("/*/dim", "/mv1/dim", True)]
        for pat, addr, ok in cases:
            self.assertEqual(bool(osc.pattern_re(pat).match(addr)), ok, (pat, addr))


class LoopbackTest(unittest.TestCase):
    def test_out_in(self):
        rx = osc.OscIn(0, host="127.0.0.1")
        port = rx.sock.getsockname()[1]
        got = []
        rx.on("/spell/*", lambda a, *args: got.append((a, args)))
        rx.on("/other", lambda a, *args: got.append("wrong"))
        tx = osc.OscOut("127.0.0.1", port)
        tx.send("/spell/play", 1, "go", 0.5)
        tx.bundle([("/spell/a", 7), osc.message("/spell/b")])
        for _ in range(40):
            if len(got) >= 3: break
            time.sleep(0.05)
        tx.close(); rx.close()
        self.assertEqual(got, [("/spell/play", (1, "go", 0.5)), ("/spell/a", (7,)), ("/spell/b", ())])


if __name__ == "__main__":
    unittest.main()
