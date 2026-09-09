# Uso: bsw_hold.py G pan tilt [zoom]  — segura um grupo de BSW (G=1: 300..360, G=2: 380..444), outro apagado, por 20 s
import sys, time
src = open(__file__.replace("bsw_hold.py", "show_medgrupo.py"), encoding="utf-8").read().split("dmx = bytearray(512)")[0]
ns = {}; exec(src, ns); dmx = bytearray(512)
g, p, t = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]); z = int(sys.argv[4]) if len(sys.argv) > 4 else 160
def bsw(on): return [p, 0, t, 0, 0, 0, 0, 0, 255 if on else 0, 255, 128, 0, z, 0, 0, 0, 0]
o = {a: bsw(g == 1) for a in [300, 320, 340, 360]}; o.update({a: bsw(g == 2) for a in [380, 404, 424, 444]})
ns["apply"](dmx, o); t0 = time.time()
while time.time() - t0 < 600: ns["send"](dmx); time.sleep(1 / 30)
