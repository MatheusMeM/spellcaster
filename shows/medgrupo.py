# MED GRUPO RJ show (plenary, 09/2026): constants, profiles, look, DUR. This is what `spell play` loads and
# the source of the pyfx track in shows/medgrupo.spell.
# Usage: spell play shows/medgrupo.py [--loop]
import bisect, math

# ---- patch (read from Capture, project medgrupo_rj.c2p, 08/09/2026) ----
VIDEO = 400                                  # Media player 1: ch1 control (8-15 play, 16-23 replay, 24-31 stop), ch2 clip
PAR_VARA = [1, 4, 7, 10]                     # Brisas LED BX-940 5 ch (R,G,B,W,A) - Matheus's patch, stride 3 (overlaps W/A)
PAR_PALCO = [187, 190, 193, 196, 199, 202]
LASER = 13                                   # Flash Butrym Laser 10W RGB, 10 ch
FRESNEL = [29, 34, 39, 44, 49, 54]           # Altman Fresnel 165, 1 ch
MV_FUNDO = [300, 320, 340, 360]              # FOS Scorpio BSW 17 ch, 4 on the back bar (next to the screen) - repatched 08/09/2026
MV_FRENTE = [380, 404, 424, 444]             # 4 on the front bar (over the proscenium)
BAR = [205, 211, 217, 223, 229, 235, 241, 247]   # Ribalta WLED bar 6 ch
WASH = [260 + 4 * i for i in range(10)]          # Ribalta BX-402 Wash 4 ch (audience) - patched on 08/09/2026

# Flash Butrym 10W RGB laser, 10 ch (patch 13-22): 1 on/off, 2 strobe, 3 size, 4 pos h, 5 pos v, 6 pattern file,
#        7 colour, 8 scan speed, 9 pattern/effect lib, 10 auto/sound
def laser(size, posh=128, posv=128, rot=0, fig=0, color=107, on=True):
    if not on: return [0] * 10
    return [255, 255, int(size), int(posh), int(posv), int(fig), int(color), 0, 0, 0]   # rot: no channel in this profile
def rgb(r, g, b): return [int(r), int(g), int(b)]
def bar(v, r=1, g=1, b=1): return [int(v * r), int(v * g), int(v * b), int(v * min(r, g, b)), 0, 0]   # R,G,B,W,UV,A

def beat(t, f=1.0, k=4): return math.exp(-k * ((t * f) % 1.0))   # decay per beat
def ease(a, b, u): u = max(0.0, min(1.0, u)); u = u * u * (3 - 2 * u); return a + (b - a) * u
def sin01(t, f, ph=0): return 0.5 + 0.5 * math.sin(2 * math.pi * f * t + ph)

# ---- BSW 17 ch: 1 pan, 2 pan f, 3 tilt, 4 tilt f, 5 colour, 6 gobo rot, 7 rot gobo, 8 fixed gobo, 9 shutter (0 closed/255 open),
#      10 dimmer, 11 focus, 12 -, 13 zoom (0=2.5 deg ... 255=36 deg; 13 deg ~ 80), 14 prism, 15 frost, 16 P/T speed, 17 function ----
# Why the moving lights looked "crazy": the BSWs have 17 channels and were patched every 16 - ch17 (special function)
# of each one received the PAN of its neighbour. Repatching at 300/320/340/360 and 380/404/424/444 fixes it. 16-bit pan/tilt = no steps.
# Geometry CALIBRATED in Capture (08/09/2026, Front camera): pan 128 / tilt 128 = straight down on all 8.
# Tilt leans the beam SIDEWAYS (back bar: higher tilt = right; front bar: higher tilt = left) and pan rotates that
# lean: pan 171 (+90 deg) = audience, pan 85 = screen on both groups. Direction (dx, dz) => angle phi => pan; magnitude => tilt.
GEO = {"fundo": dict(pan=128, tilt=128, amp=38, lado=+1, zmin=-1.2),      # back bar, pointing down
       "frente": dict(pan=128, tilt=128, amp=38, lado=-1, zmin=-0.15)}    # proscenium FLOOR, pointing up: dz<0 = aims at the screen, avoid
_LAST = {}
SLEW_PAN, SLEW_TILT = 2.0, 0.03                  # per frame (30 Hz): max 60 deg/s of pan, ~1 s from screen to audience
PAN_DEG = 255 / 540                              # DMX units per degree of pan
ZMIN, ZMAX = 80, 255                             # 13 deg ... 36 deg: never below 13 deg, prefer it open
BRANCO, VERMELHO, VERDE, AMARELO, AZUL = 0, 16, 32, 48, 64    # BSW colour wheel (calibrated 08/09)
G_PONTOS, G_ANEL, G_RAIOS = 104, 88, 120         # rotating gobo wheel (calibrated 08/09): dots, rings, spiral
ABERTO = 255                                     # shutter open; 32-63 = strobe
STROBE = 48
def mv(group, dim, dz=0.0, dx=0.0, spread=0.0, color=0, gobo=0, grot=0, gobo2=0, prism=0, frost=0, zoom=ZMAX, strobe=ABERTO,
       wave=0.0, t=0.0, wph=0.8, wf=0.6, focus=128):
    """The group as one unit. dz: -1 screen ... +1 audience; dx: -1 left ... +1 right (seen from the audience); spread: symmetric fan
    (left opens to the left, right to the right); spread<0 = crossed beams. wave: SLOW wave (wf Hz) of dz running across the units.
    16-bit pan/tilt; the pan turn is chosen frame by frame by continuity (hysteresis), with no jumps."""
    g = GEO["fundo" if group[0] in MV_FUNDO else "frente"]; o = {}
    n = len(group)
    for i, a in enumerate(group):
        side = (i - (n - 1) / 2) / max(1, (n - 1) / 2)             # -1 ... +1 from left to right
        x = dx + spread * side; z = max(g["zmin"], dz + wave * math.sin(2 * math.pi * wf * t + i * wph))
        m = min(1.2, math.hypot(x, z))
        phi = math.degrees(math.atan2(z, g["lado"] * x)) if m > 0.02 else 0.0
        lp, lm = _LAST.get(a, (phi, m))                              # continuity: the pan turn closest to the previous frame...
        phi = min((phi + k for k in (-360, 0, 360) if abs(phi + k) <= 265), key=lambda c: abs(c - lp))
        phi = lp + max(-SLEW_PAN, min(SLEW_PAN, phi - lp)); m = lm + max(-SLEW_TILT, min(SLEW_TILT, m - lm))   # ...and a limited ramp
        _LAST[a] = (phi, m)
        pn = g["pan"] + phi * PAN_DEG; tl = g["tilt"] + g["amp"] * m
        p16, t16 = [int(max(0.0, min(255.0, v)) * 257) for v in (pn, tl)]
        o[a] = [p16 >> 8, p16 & 255, t16 >> 8, t16 & 255, int(color), int(gobo), int(grot), int(gobo2), int(strobe), int(dim),
                int(focus), 0, int(max(ZMIN, min(ZMAX, zoom))), int(prism), int(frost), 0, 0]
    return o
def mv8(*a, **k): o = mv(MV_FUNDO, *a, **k); o.update(mv(MV_FRENTE, *a, **k)); return o
# MED GRUPO communication colours (social media / video): yellow and lime green over a dark background, white for support.
C_AMARELO, C_LIMA, C_NAVY, C_BRANCO = (255, 215, 0), (150, 255, 0), (0, 25, 110), (255, 255, 255)
def wash(v, c): return [int(v * c[0] / 255), int(v * c[1] / 255), int(v * c[2] / 255), int(v * min(c) / 255)]   # BX-402 R,G,B,W
def par(v, c): return rgb(v * c[0] / 255, v * c[1] / 255, v * c[2] / 255)
def barc(v, c): return bar(v, c[0] / 255, c[1] / 255, c[2] / 255)
def lerp3(c1, c2, u): return tuple(c1[k] + (c2[k] - c1[k]) * max(0, min(1, u)) for k in range(3))

VIDEO_END = 46.8
CUES = [12.0, 14.8, 17.72, 19.72, 21.36, 23.36, 26.84, 29.5]   # video cuts (ffmpeg scene) in the cities stretch
PICO = 33.12                                             # cut to the logo
def mv_solo(a):                                   # epilogue: one unit alone, straight down, gobo turning
    grp = MV_FUNDO if a in MV_FUNDO else MV_FRENTE
    return mv(grp, 255, color=AMARELO, gobo=G_RAIOS, grot=140, zoom=200)[a]
ORDEM = [("LED par, boom", a, lambda: par(255, C_AMARELO)) for a in PAR_VARA] + \
        [("LED par, stage", a, lambda: par(255, C_LIMA)) for a in PAR_PALCO] + \
        [("Fresnel", a, lambda: [255]) for a in FRESNEL] + \
        [("Moving BSW", a, (lambda a=a: mv_solo(a))) for a in MV_FUNDO + MV_FRENTE] + \
        [("WLED footlight", a, lambda: barc(255, C_AMARELO)) for a in BAR] + \
        [("BX-402 footlight, audience", a, lambda: wash(255, C_LIMA)) for a in WASH] + \
        [("Laser", LASER, lambda: laser(200, color=107))]
T_CADA = 0.7; T_DID = 1.0 + len(ORDEM) * T_CADA          # epilogue: 1 s of blackout + 0.7 s per fixture
DUR = VIDEO_END + T_DID + 6                               # + all of them together 4 s + 2 s fade

def look(t):
    """5E dramaturgy over the video (46.8 s) + didactic epilogue:
    EXCITEMENT 0-10.5 countdown: reveal on the beat, the moving lights bring gobos down slowly, brand yellow/green.
    ENTRY 10.5-12 flash: the threshold - yellow flash (like the video) with a short strobe, then white on the '0'; the 8 moving lights with prism.
    ENGAGEMENT 12-33.1 cities: every pose change of the moving lights lands on a video cut (CUES); back bar and front bar
      (floor, pointing up) alternate fans, crossings and waves; at 23.36 (the list stops at Cachoeiro) = contemplative pause (valley),
      26.84 pick-up, 29.5 'final destination' rises to the logo. A blackout breath at 9.6-10 s before the threshold.
    PEAK 33.12-40 logo: the 8 as one unit, zoom closed to 13 deg only at this moment, then it opens; prism, yellow on the beat.
    EXIT 40-46.8: everything closes to the centre (frost) and goes out; the fresnels warm, last.
    EPILOGUE 46.8+: blackout, each fixture lights up alone (patch order), then all together, fade."""
    o = {}
    if t < 10.5:                                                   # EXCITEMENT (countdown 10...1 up to 10.5 s)
        b = beat(t); k = int(t)
        for a in FRESNEL: o[a] = [int(120 * b) if k < 2 else 30]                                    # 0-2 s: the fresnels pulse
        for i, a in enumerate(BAR): o[a] = barc(255 * b if k >= 2 and k % 2 == i % 2 else 0, C_AMARELO)   # 2 s+: WLED bar chase
        for i, a in enumerate(WASH): o[a] = wash(180 * b if k >= 3 and (i // 5) == k % 2 else 0, C_LIMA)  # 3 s+: the walls alternate
        for a in PAR_VARA + PAR_PALCO: o[a] = par((40 + 160 * b) if k >= 4 else 0, C_NAVY)              # 4 s+: navy pars breathing
        o.update(mv(MV_FUNDO, ease(0, 220, (t - 5) / 1) * (0.6 + 0.4 * b), dz=-0.3, spread=ease(0, 0.6, (t - 5) / 4),
                    color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230))                                  # 5 s+: back bar opens a gobo fan
        o.update(mv(MV_FRENTE, ease(0, 220, (t - 7) / 1) * (0.6 + 0.4 * b), dz=+0.3, spread=ease(0, 0.6, (t - 7) / 3),
                    color=VERDE, gobo=G_ANEL, grot=120, zoom=230))                                     # 7 s+: front bar opens a fan
        o[LASER] = laser(100 + 80 * b if k >= 1 else 0, on=k >= 1)
        if t >= 10.1:                                                  # breath: 0.4 s of blackout before the video's yellow flash
            for a in list(o):
                if a not in MV_FUNDO + MV_FRENTE: o[a] = [0] * len(o[a])
            o.update(mv(MV_FUNDO, 0, dz=-0.3, spread=0.6, color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230))
            o.update(mv(MV_FRENTE, 0, dz=+0.3, spread=0.6, color=VERDE, gobo=G_ANEL, grot=120, zoom=230))
    elif t < 12.0:                                                 # ENTRY - the video's yellow flash (10.5-11.3) and the '0' (11.5-12)
        u = (t - 10.5) / 1.5; st = STROBE if u < 0.5 else ABERTO; c = C_AMARELO if u < 0.55 else C_BRANCO
        for a in PAR_VARA + PAR_PALCO: o[a] = par(255, c)
        for a in FRESNEL: o[a] = [255]
        for a in BAR: o[a] = barc(255, c)
        for a in WASH: o[a] = wash(255, c)
        o.update(mv8(255, dz=ease(-0.3, -0.9, u), spread=ease(0.6, 0.2, u), color=AMARELO if u < 0.55 else BRANCO, prism=255, zoom=255, strobe=st))
        o[LASER] = laser(ease(80, 255, u), color=255)
    elif t < PICO:                                                 # ENGAGEMENT - blocks anchored to the video cuts (CUES)
        seg = bisect.bisect_right(CUES, t) - 1; t0 = CUES[seg]; t1 = CUES[seg + 1] if seg + 1 < len(CUES) else PICO
        us = (t - t0) / (t1 - t0); u = (t - 12.0) / (PICO - 12.0)
        cp = [C_AMARELO, C_LIMA, C_NAVY][seg % 3]; cs = [AMARELO, VERDE, BRANCO][seg % 3]; gb = [G_PONTOS, G_RAIOS, G_ANEL][seg % 3]
        for i, a in enumerate(PAR_VARA + PAR_PALCO): o[a] = par(255 * (0.3 + 0.7 * sin01(t, 0.25, i)), cp)
        for i, a in enumerate(FRESNEL): o[a] = [int(80 * sin01(t, 0.25, i)) if seg % 2 == 0 else 0]
        for i, a in enumerate(BAR): o[a] = barc(255 if int(t * 2) % 8 == i else 40, cp)
        for i, a in enumerate(WASH): o[a] = wash(255 if int(t * 2) % 8 == i else 70, [C_LIMA, C_AMARELO, C_NAVY][seg % 3])
        if seg == 0:                # 12.72 the list comes in: back bar in a gobo fan breathing on the screen; front bar in tight vertical beams
            o.update(mv(MV_FUNDO, 230, dz=-0.5, spread=0.5 + 0.3 * math.sin(2 * math.pi * 0.3 * t), color=cs, gobo=gb, grot=135, zoom=240))
            o.update(mv(MV_FRENTE, 200, dz=+0.1, spread=0.2, color=cs, gobo=gb, grot=120, zoom=200))
        elif seg == 1:              # 14.8 Rio: front bar sweeps up towards the audience with prism; back bar washes (frost) the screen edges
            o.update(mv(MV_FRENTE, 240, dz=ease(0.1, 0.9, us), spread=0.4, color=cs, gobo=gb, prism=255, zoom=220))
            o.update(mv(MV_FUNDO, 180, dz=-0.4, spread=0.6, color=cs, frost=255, zoom=255))
        elif seg == 2:              # 17.72: the 8 in a slow wave running from left to right
            o.update(mv8(240, dz=0.3, spread=0.5, color=cs, gobo=gb, grot=130, wave=0.4, t=t, wph=0.8, wf=0.4, zoom=230))
        elif seg == 3:              # 19.72: back bar in crossed beams with prism; front bar in an open fan
            o.update(mv(MV_FUNDO, 240, dz=-0.3, spread=-0.7, color=cs, gobo=gb, prism=255, zoom=200))
            o.update(mv(MV_FRENTE, 200, dz=+0.4, spread=0.8, color=cs, gobo=gb, grot=120, zoom=255))
        elif seg == 4:              # 21.36: the 8 turn in unison from the screen to the audience, mirrored fan
            o.update(mv8(250, dz=ease(-0.6, 0.8, us), spread=0.5, color=cs, gobo=gb, grot=140, zoom=230))
        elif seg == 5:              # 23.36 the list stops at CACHOEIRO: contemplative PAUSE (valley) - front bar goes out, back bar slow frost
            for i, a in enumerate(PAR_VARA + PAR_PALCO): o[a] = par(90, C_NAVY)
            for a in FRESNEL: o[a] = [0]
            for a in BAR: o[a] = barc(30, C_AMARELO)
            for a in WASH: o[a] = wash(50, C_NAVY)
            o.update(mv(MV_FUNDO, ease(240, 140, us * 2), dz=-0.2, spread=0.6, color=AMARELO, frost=255, zoom=255))
            o.update(mv(MV_FRENTE, ease(200, 0, us * 2), dz=+0.3, spread=0.5, color=cs, frost=255, zoom=255))
        elif seg == 6:              # 26.84: pick-up - the 8 rise together, gobo turning, fan opening
            o.update(mv8(ease(0, 250, us * 3), dz=ease(-0.2, 0.6, us), spread=ease(0.2, 0.6, us), color=AMARELO, gobo=G_RAIOS, grot=140, zoom=230))
        else:                       # 29.5 "the final destination is approval": the 8 cross and open with prism, rising to the logo
            o.update(mv8(255, dz=0.4, spread=-0.7 + 1.3 * sin01(us, 1, -math.pi / 2), color=AMARELO, gobo=G_RAIOS, grot=140, prism=255, zoom=240))
        o[LASER] = laser(150, posh=170, posv=128 + 70 * math.sin(u * 2 * math.pi * 3))
    elif t < 40:                                                   # PEAK - the logo enters at 33.12: the 8 as one unit
        u = (t - PICO) / (40 - PICO); b = beat(t, 2)
        for a in PAR_VARA + PAR_PALCO: o[a] = par(255, lerp3(C_LIMA, C_AMARELO, b))
        for a in FRESNEL: o[a] = [int(150 + 100 * b)]
        for i, a in enumerate(BAR): o[a] = barc(255 * b, C_AMARELO)
        for i, a in enumerate(WASH): o[a] = wash(255 * (b if i % 2 == int(t * 2) % 2 else 0.3), C_LIMA)
        o.update(mv8(255, dz=ease(-0.8, 0.8, u), spread=0.3 + 0.3 * math.sin(2 * math.pi * 0.5 * t),
                     color=AMARELO if b > 0.5 else BRANCO, prism=255, gobo=G_RAIOS, grot=140,
                     zoom=ease(ZMIN, ZMAX, (u - 0.2) / 0.6), strobe=STROBE if u < 0.15 else ABERTO))   # the only moment at 13 deg: opens up to 36 deg
        o[LASER] = laser(ease(255, 140, u), color=255)
    elif t < VIDEO_END:                                            # EXIT
        u = (t - 40) / 6.8
        for a in PAR_VARA + PAR_PALCO: o[a] = par(ease(150, 0, u * 1.5), C_AMARELO)
        for a in BAR: o[a] = barc(0, C_AMARELO)
        for a in WASH: o[a] = wash(ease(120, 0, u * 1.2), C_LIMA)
        o.update(mv8(ease(220, 0, u * 1.3), dz=ease(0.5, 0, u), spread=ease(0.5, 0, u), color=AMARELO, frost=255, zoom=255))
        for a in FRESNEL: o[a] = [int(ease(200, 0, max(0, (u - 0.5) * 2)))]
        o[LASER] = laser(ease(140, 0, u * 1.3), on=u < 0.7)
    else:                                                          # didactic EPILOGUE
        te = t - VIDEO_END
        for a in PAR_VARA + PAR_PALCO: o[a] = [0, 0, 0]
        for a in FRESNEL: o[a] = [0]
        for a in BAR: o[a] = bar(0)
        for a in WASH: o[a] = [0, 0, 0, 0]
        o.update(mv8(0))
        o[LASER] = laser(0, on=False)
        if te < 1.0: return o                                      # blackout
        n = int((te - 1.0) / T_CADA)
        if n < len(ORDEM):                                         # one fixture at a time
            _, a, f = ORDEM[n]; o[a] = f()
        else:                                                      # all together 4 s, then a 2 s fade
            u = (te - T_DID) / 4; g = 1.0 if u < 1 else max(0.0, 1 - (u - 1) * 2)
            for _, a, f in ORDEM:
                v = f(); o[a] = [int(x * g) for x in v] if len(v) != 17 else v
            o.update(mv8(255 * g, dz=0.3, spread=0.5, color=AMARELO, prism=255, gobo=G_RAIOS, grot=140, zoom=230))
            if g == 0: o[LASER] = laser(0, on=False)
    return o

# ---- compatibility with the seed's run_once: video pre-roll (1 s stop + 1 s replay) and the player's control channel ----
PRE = 2.0
DUR_LUZ = DUR
DUR = DUR_LUZ + PRE
_look_luz = look
def look(t):
    o = _look_luz(max(0.0, t - PRE)) if t >= PRE else _look_luz(DUR_LUZ)
    o[VIDEO] = [28 if t < 1 else (20 if t < PRE else (10 if t < PRE + VIDEO_END else 28)), 0]   # ch1 control, ch2 clip
    return o
