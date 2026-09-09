# Uso: bsw_multi.py pan tilt ch v1 v2 ... v8  — os 8 BSW (300..360 depois 380..444) com o canal ch (1-17) em valores diferentes
import sys, time
src = open(__file__.replace("bsw_multi.py", "show_medgrupo.py"), encoding="utf-8").read().split("dmx = bytearray(512)")[0]
ns = {}; exec(src, ns); dmx = bytearray(512)
p, t, ch = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]); vals = [int(x) for x in sys.argv[4:12]]
o = {}
for a, v in zip([300, 320, 340, 360, 380, 404, 424, 444], vals):
    d = [p, 0, t, 0, 0, 0, 0, 0, 255, 255, 128, 0, 200, 0, 0, 0, 0]; d[ch - 1] = v; o[a] = d
ns["apply"](dmx, o); t0 = time.time()
while time.time() - t0 < 600: ns["send"](dmx); time.sleep(1 / 30)
