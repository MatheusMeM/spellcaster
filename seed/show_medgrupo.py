# Show sincronizado ao vídeo de abertura (46,8 s): dispara o Reprodutor de Mídia do Capture (telão) e roda a
# timeline de luz/laser no mesmo relógio. sACN universo 1 (multicast + localhost), só stdlib.
# Uso: python show_medgrupo.py            roda uma vez
#      python show_medgrupo.py loop       repete
#      python show_medgrupo.py laser 5=40 8=200 ...   modo calibração: laser parado com esses canais (1-16)
import bisect, math, socket, struct, sys, time, uuid

# ---- patch (lido do Capture, projeto medgrupo_rj.c2p, 08/09/2026) ----
VIDEO = 400                                  # Reprodutor de mídia 1: ch1 controle (8-15 play, 16-23 replay, 24-31 stop), ch2 clipe
PAR_VARA = [1, 4, 7, 10]                     # Brisas LED BX-940 5 ch (R,G,B,W,A) — patch do Matheus, stride 3 (sobrepõe W/A)
PAR_PALCO = [187, 190, 193, 196, 199, 202]
LASER = 13                                   # Flash Butrym Laser 10W RGB, 10 ch
FRESNEL = [29, 34, 39, 44, 49, 54]           # Altman Fresnel 165, 1 ch
MV_FUNDO = [300, 320, 340, 360]              # FOS Scorpio BSW 17 ch, 4 na vara do fundo (junto do telão) — repatch 08/09/2026
MV_FRENTE = [380, 404, 424, 444]             # 4 na vara da frente (sobre a boca de cena)
BAR = [205, 211, 217, 223, 229, 235, 241, 247]   # Ribalta WLED bar 6 ch
WASH = [260 + 4 * i for i in range(10)]          # Ribalta BX-402 Wash 4 ch (plateia) — patch feito em 08/09/2026

# Laser Flash Butrym 10W RGB, 10 ch (patch 13-22): 1 on/off, 2 strobe, 3 tamanho, 4 pos h, 5 pos v, 6 pattern file,
#        7 cor, 8 vel scan, 9 pattern/effect lib, 10 auto/som
def laser(size, posh=128, posv=128, rot=0, fig=0, color=107, on=True):
    if not on: return [0] * 10
    return [255, 255, int(size), int(posh), int(posv), int(fig), int(color), 0, 0, 0]   # rot: sem canal neste perfil
def rgb(r, g, b): return [int(r), int(g), int(b)]
def bar(v, r=1, g=1, b=1): return [int(v * r), int(v * g), int(v * b), int(v * min(r, g, b)), 0, 0]   # R,G,B,W,UV,A

def beat(t, f=1.0, k=4): return math.exp(-k * ((t * f) % 1.0))   # decaimento por batida
def ease(a, b, u): u = max(0.0, min(1.0, u)); u = u * u * (3 - 2 * u); return a + (b - a) * u
def sin01(t, f, ph=0): return 0.5 + 0.5 * math.sin(2 * math.pi * f * t + ph)

# ---- BSW 17 ch: 1 pan, 2 pan f, 3 tilt, 4 tilt f, 5 cor, 6 gobo rot, 7 rot gobo, 8 gobo fixo, 9 shutter (0 fechado/255 aberto),
#      10 dimmer, 11 foco, 12 -, 13 zoom (0=2,5° … 255=36°; 13° ≈ 80), 14 prisma, 15 frost, 16 vel P/T, 17 função ----
# Por que os movings pareciam "malucos": os BSW têm 17 canais e estavam patcheados de 16 em 16 — o ch17 (função especial)
# de cada um recebia o PAN do vizinho. Repatch em 300/320/340/360 e 380/404/424/444 resolve. Pan/tilt em 16 bit = sem degraus.
# Geometria CALIBRADA no Capture (08/09/2026, câmera Frontal): pan 128 / tilt 128 = reto para baixo nos 8.
# O tilt inclina o raio LATERALMENTE (fundo: tilt maior = direita; frente: tilt maior = esquerda) e o pan gira essa
# inclinação: pan 171 (+90°) = plateia, pan 85 = tela nos dois grupos. Direção (dx, dz) => ângulo phi => pan; módulo => tilt.
GEO = {"fundo": dict(pan=128, tilt=128, amp=38, lado=+1, zmin=-1.2),      # vara do fundo, apontando para baixo
       "frente": dict(pan=128, tilt=128, amp=38, lado=-1, zmin=-0.15)}    # CHÃO da boca de cena, apontando para cima: dz<0 = mira o telão, evitar
_LAST = {}
SLEW_PAN, SLEW_TILT = 2.0, 0.03                  # por frame (30 Hz): máx. 60°/s de pan, ~1 s de tela a plateia
PAN_DEG = 255 / 540                              # unidades DMX por grau de pan
ZMIN, ZMAX = 80, 255                             # 13° … 36°: nunca abaixo de 13°, preferir aberto
BRANCO, VERMELHO, VERDE, AMARELO, AZUL = 0, 16, 32, 48, 64    # roda de cor BSW (calibrada 08/09)
G_PONTOS, G_ANEL, G_RAIOS = 104, 88, 120         # roda de gobo rotativa (calibrada 08/09): pontos, anéis, espiral
ABERTO = 255                                     # shutter aberto; 32-63 = strobe
STROBE = 48
def mv(group, dim, dz=0.0, dx=0.0, spread=0.0, color=0, gobo=0, grot=0, gobo2=0, prism=0, frost=0, zoom=ZMAX, strobe=ABERTO,
       wave=0.0, t=0.0, wph=0.8, wf=0.6, focus=128):
    """Grupo como unidade. dz: -1 tela … +1 plateia; dx: -1 esq … +1 dir (visto da plateia); spread: leque simétrico
    (esq. abre p/ esq., dir. p/ dir.); spread<0 = raios cruzados. wave: onda LENTA (wf Hz) de dz percorrendo as unidades.
    Pan/tilt em 16 bit; a volta de pan é escolhida frame a frame pela continuidade (histerese), sem saltos."""
    g = GEO["fundo" if group[0] in MV_FUNDO else "frente"]; o = {}
    n = len(group)
    for i, a in enumerate(group):
        side = (i - (n - 1) / 2) / max(1, (n - 1) / 2)             # -1 … +1 da esquerda para a direita
        x = dx + spread * side; z = max(g["zmin"], dz + wave * math.sin(2 * math.pi * wf * t + i * wph))
        m = min(1.2, math.hypot(x, z))
        phi = math.degrees(math.atan2(z, g["lado"] * x)) if m > 0.02 else 0.0
        lp, lm = _LAST.get(a, (phi, m))                              # continuidade: volta de pan mais perto do frame anterior…
        phi = min((phi + k for k in (-360, 0, 360) if abs(phi + k) <= 265), key=lambda c: abs(c - lp))
        phi = lp + max(-SLEW_PAN, min(SLEW_PAN, phi - lp)); m = lm + max(-SLEW_TILT, min(SLEW_TILT, m - lm))   # …e rampa limitada
        _LAST[a] = (phi, m)
        pn = g["pan"] + phi * PAN_DEG; tl = g["tilt"] + g["amp"] * m
        p16, t16 = [int(max(0.0, min(255.0, v)) * 257) for v in (pn, tl)]
        o[a] = [p16 >> 8, p16 & 255, t16 >> 8, t16 & 255, int(color), int(gobo), int(grot), int(gobo2), int(strobe), int(dim),
                int(focus), 0, int(max(ZMIN, min(ZMAX, zoom))), int(prism), int(frost), 0, 0]
    return o
def mv8(*a, **k): o = mv(MV_FUNDO, *a, **k); o.update(mv(MV_FRENTE, *a, **k)); return o
# Cores da comunicação MED GRUPO (redes sociais / vídeo): amarelo e verde-lima sobre fundo escuro, branco de apoio.
C_AMARELO, C_LIMA, C_NAVY, C_BRANCO = (255, 215, 0), (150, 255, 0), (0, 25, 110), (255, 255, 255)
def wash(v, c): return [int(v * c[0] / 255), int(v * c[1] / 255), int(v * c[2] / 255), int(v * min(c) / 255)]   # BX-402 R,G,B,W
def par(v, c): return rgb(v * c[0] / 255, v * c[1] / 255, v * c[2] / 255)
def barc(v, c): return bar(v, c[0] / 255, c[1] / 255, c[2] / 255)
def lerp3(c1, c2, u): return tuple(c1[k] + (c2[k] - c1[k]) * max(0, min(1, u)) for k in range(3))

VIDEO_END = 46.8
CUES = [12.0, 14.8, 17.72, 19.72, 21.36, 23.36, 26.84, 29.5]   # cortes do vídeo (ffmpeg scene) no trecho das cidades
PICO = 33.12                                             # corte para o logo
def mv_solo(a):                                   # epílogo: unidade sozinha, reto para baixo, gobo girando
    grp = MV_FUNDO if a in MV_FUNDO else MV_FRENTE
    return mv(grp, 255, color=AMARELO, gobo=G_RAIOS, grot=140, zoom=200)[a]
ORDEM = [("Par LED vara", a, lambda: par(255, C_AMARELO)) for a in PAR_VARA] + \
        [("Par LED palco", a, lambda: par(255, C_LIMA)) for a in PAR_PALCO] + \
        [("Fresnel", a, lambda: [255]) for a in FRESNEL] + \
        [("Moving BSW", a, (lambda a=a: mv_solo(a))) for a in MV_FUNDO + MV_FRENTE] + \
        [("Ribalta WLED", a, lambda: barc(255, C_AMARELO)) for a in BAR] + \
        [("Ribalta BX-402 plateia", a, lambda: wash(255, C_LIMA)) for a in WASH] + \
        [("Laser", LASER, lambda: laser(200, color=107))]
T_CADA = 0.7; T_DID = 1.0 + len(ORDEM) * T_CADA          # epílogo: 1 s de breu + 0,7 s por aparelho
DUR = VIDEO_END + T_DID + 6                               # + todos juntos 4 s + fade 2 s

def look(t):
    """Dramaturgia 5E sobre o vídeo (46,8 s) + epílogo didático:
    EXCITEMENT 0-10,5 contagem: revelação por batida, os movings descem gobos devagar, amarelo/verde da marca.
    ENTRY 10,5-12 flash: limiar — flash amarelo (como o vídeo) com strobe curto, depois branco no '0'; os 8 movings com prisma.
    ENGAGEMENT 12-33,1 cidades: cada troca de pose dos movings cai num corte do vídeo (CUES); fundo (vara) e frente
      (chão, para cima) alternam leques, cruzamentos e ondas; em 23,36 (lista para em Cachoeiro) = pausa contemplativa (vale),
      26,84 retomada, 29,5 'destino final' sobe até o logo. Respiro de breu em 9,6-10 s antes do limiar.
    PICO 33,12-40 logo: os 8 como unidade, zoom fechado em 13° só neste instante, depois abre; prisma, amarelo na batida.
    EXIT 40-46,8: tudo fecha ao centro (frost) e apaga; fresnel quente por último.
    EPÍLOGO 46,8+: breu, cada aparelho acende sozinho (ordem do patch), depois todos juntos, fade."""
    o = {}
    if t < 10.5:                                                   # EXCITEMENT (contagem 10…1 até 10,5 s)
        b = beat(t); k = int(t)
        for a in FRESNEL: o[a] = [int(120 * b) if k < 2 else 30]                                    # 0-2 s: fresnel pulsa
        for i, a in enumerate(BAR): o[a] = barc(255 * b if k >= 2 and k % 2 == i % 2 else 0, C_AMARELO)   # 2 s+: ribalta chase
        for i, a in enumerate(WASH): o[a] = wash(180 * b if k >= 3 and (i // 5) == k % 2 else 0, C_LIMA)  # 3 s+: paredes alternam
        for a in PAR_VARA + PAR_PALCO: o[a] = par((40 + 160 * b) if k >= 4 else 0, C_NAVY)              # 4 s+: pares navy respirando
        o.update(mv(MV_FUNDO, ease(0, 220, (t - 5) / 1) * (0.6 + 0.4 * b), dz=-0.3, spread=ease(0, 0.6, (t - 5) / 4),
                    color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230))                                  # 5 s+: fundo abre leque de gobo
        o.update(mv(MV_FRENTE, ease(0, 220, (t - 7) / 1) * (0.6 + 0.4 * b), dz=+0.3, spread=ease(0, 0.6, (t - 7) / 3),
                    color=VERDE, gobo=G_ANEL, grot=120, zoom=230))                                     # 7 s+: frente abre leque
        o[LASER] = laser(100 + 80 * b if k >= 1 else 0, on=k >= 1)
        if t >= 10.1:                                                  # respiro: 0,4 s de breu antes do flash amarelo do vídeo
            for a in list(o):
                if a not in MV_FUNDO + MV_FRENTE: o[a] = [0] * len(o[a])
            o.update(mv(MV_FUNDO, 0, dz=-0.3, spread=0.6, color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230))
            o.update(mv(MV_FRENTE, 0, dz=+0.3, spread=0.6, color=VERDE, gobo=G_ANEL, grot=120, zoom=230))
    elif t < 12.0:                                                 # ENTRY — flash amarelo do vídeo (10,5-11,3) e '0' (11,5-12)
        u = (t - 10.5) / 1.5; st = STROBE if u < 0.5 else ABERTO; c = C_AMARELO if u < 0.55 else C_BRANCO
        for a in PAR_VARA + PAR_PALCO: o[a] = par(255, c)
        for a in FRESNEL: o[a] = [255]
        for a in BAR: o[a] = barc(255, c)
        for a in WASH: o[a] = wash(255, c)
        o.update(mv8(255, dz=ease(-0.3, -0.9, u), spread=ease(0.6, 0.2, u), color=AMARELO if u < 0.55 else BRANCO, prism=255, zoom=255, strobe=st))
        o[LASER] = laser(ease(80, 255, u), color=255)
    elif t < PICO:                                                 # ENGAGEMENT — blocos ancorados nos cortes do vídeo (CUES)
        seg = bisect.bisect_right(CUES, t) - 1; t0 = CUES[seg]; t1 = CUES[seg + 1] if seg + 1 < len(CUES) else PICO
        us = (t - t0) / (t1 - t0); u = (t - 12.0) / (PICO - 12.0)
        cp = [C_AMARELO, C_LIMA, C_NAVY][seg % 3]; cs = [AMARELO, VERDE, BRANCO][seg % 3]; gb = [G_PONTOS, G_RAIOS, G_ANEL][seg % 3]
        for i, a in enumerate(PAR_VARA + PAR_PALCO): o[a] = par(255 * (0.3 + 0.7 * sin01(t, 0.25, i)), cp)
        for i, a in enumerate(FRESNEL): o[a] = [int(80 * sin01(t, 0.25, i)) if seg % 2 == 0 else 0]
        for i, a in enumerate(BAR): o[a] = barc(255 if int(t * 2) % 8 == i else 40, cp)
        for i, a in enumerate(WASH): o[a] = wash(255 if int(t * 2) % 8 == i else 70, [C_LIMA, C_AMARELO, C_NAVY][seg % 3])
        if seg == 0:                # 12,72 lista entra: fundo leque de gobo respirando na tela; frente raios verticais fechados
            o.update(mv(MV_FUNDO, 230, dz=-0.5, spread=0.5 + 0.3 * math.sin(2 * math.pi * 0.3 * t), color=cs, gobo=gb, grot=135, zoom=240))
            o.update(mv(MV_FRENTE, 200, dz=+0.1, spread=0.2, color=cs, gobo=gb, grot=120, zoom=200))
        elif seg == 1:              # 14,8 Rio: frente varre para cima rumo à plateia com prisma; fundo wash (frost) nas bordas da tela
            o.update(mv(MV_FRENTE, 240, dz=ease(0.1, 0.9, us), spread=0.4, color=cs, gobo=gb, prism=255, zoom=220))
            o.update(mv(MV_FUNDO, 180, dz=-0.4, spread=0.6, color=cs, frost=255, zoom=255))
        elif seg == 2:              # 17,72: os 8 em onda lenta percorrendo da esquerda para a direita
            o.update(mv8(240, dz=0.3, spread=0.5, color=cs, gobo=gb, grot=130, wave=0.4, t=t, wph=0.8, wf=0.4, zoom=230))
        elif seg == 3:              # 19,72: fundo raios cruzados com prisma; frente leque aberto
            o.update(mv(MV_FUNDO, 240, dz=-0.3, spread=-0.7, color=cs, gobo=gb, prism=255, zoom=200))
            o.update(mv(MV_FRENTE, 200, dz=+0.4, spread=0.8, color=cs, gobo=gb, grot=120, zoom=255))
        elif seg == 4:              # 21,36: os 8 giram em uníssono da tela para a plateia, leque espelhado
            o.update(mv8(250, dz=ease(-0.6, 0.8, us), spread=0.5, color=cs, gobo=gb, grot=140, zoom=230))
        elif seg == 5:              # 23,36 lista para em CACHOEIRO: PAUSA contemplativa (vale) — frente apaga, fundo frost lento
            for i, a in enumerate(PAR_VARA + PAR_PALCO): o[a] = par(90, C_NAVY)
            for a in FRESNEL: o[a] = [0]
            for a in BAR: o[a] = barc(30, C_AMARELO)
            for a in WASH: o[a] = wash(50, C_NAVY)
            o.update(mv(MV_FUNDO, ease(240, 140, us * 2), dz=-0.2, spread=0.6, color=AMARELO, frost=255, zoom=255))
            o.update(mv(MV_FRENTE, ease(200, 0, us * 2), dz=+0.3, spread=0.5, color=cs, frost=255, zoom=255))
        elif seg == 6:              # 26,84: retomada — os 8 sobem juntos, gobo girando, leque abrindo
            o.update(mv8(ease(0, 250, us * 3), dz=ease(-0.2, 0.6, us), spread=ease(0.2, 0.6, us), color=AMARELO, gobo=G_RAIOS, grot=140, zoom=230))
        else:                       # 29,5 "o destino final é a aprovação": os 8 cruzam e abrem com prisma, subindo até o logo
            o.update(mv8(255, dz=0.4, spread=-0.7 + 1.3 * sin01(us, 1, -math.pi / 2), color=AMARELO, gobo=G_RAIOS, grot=140, prism=255, zoom=240))
        o[LASER] = laser(150, posh=170, posv=128 + 70 * math.sin(u * 2 * math.pi * 3))
    elif t < 40:                                                   # PICO — logo entra em 33,12: 8 como unidade
        u = (t - PICO) / (40 - PICO); b = beat(t, 2)
        for a in PAR_VARA + PAR_PALCO: o[a] = par(255, lerp3(C_LIMA, C_AMARELO, b))
        for a in FRESNEL: o[a] = [int(150 + 100 * b)]
        for i, a in enumerate(BAR): o[a] = barc(255 * b, C_AMARELO)
        for i, a in enumerate(WASH): o[a] = wash(255 * (b if i % 2 == int(t * 2) % 2 else 0.3), C_LIMA)
        o.update(mv8(255, dz=ease(-0.8, 0.8, u), spread=0.3 + 0.3 * math.sin(2 * math.pi * 0.5 * t),
                     color=AMARELO if b > 0.5 else BRANCO, prism=255, gobo=G_RAIOS, grot=140,
                     zoom=ease(ZMIN, ZMAX, (u - 0.2) / 0.6), strobe=STROBE if u < 0.15 else ABERTO))   # único momento a 13°: abre até 36°
        o[LASER] = laser(ease(255, 140, u), color=255)
    elif t < VIDEO_END:                                            # EXIT
        u = (t - 40) / 6.8
        for a in PAR_VARA + PAR_PALCO: o[a] = par(ease(150, 0, u * 1.5), C_AMARELO)
        for a in BAR: o[a] = barc(0, C_AMARELO)
        for a in WASH: o[a] = wash(ease(120, 0, u * 1.2), C_LIMA)
        o.update(mv8(ease(220, 0, u * 1.3), dz=ease(0.5, 0, u), spread=ease(0.5, 0, u), color=AMARELO, frost=255, zoom=255))
        for a in FRESNEL: o[a] = [int(ease(200, 0, max(0, (u - 0.5) * 2)))]
        o[LASER] = laser(ease(140, 0, u * 1.3), on=u < 0.7)
    else:                                                          # EPÍLOGO didático
        te = t - VIDEO_END
        for a in PAR_VARA + PAR_PALCO: o[a] = [0, 0, 0]
        for a in FRESNEL: o[a] = [0]
        for a in BAR: o[a] = bar(0)
        for a in WASH: o[a] = [0, 0, 0, 0]
        o.update(mv8(0))
        o[LASER] = laser(0, on=False)
        if te < 1.0: return o                                      # breu
        n = int((te - 1.0) / T_CADA)
        if n < len(ORDEM):                                         # um aparelho por vez
            _, a, f = ORDEM[n]; o[a] = f()
        else:                                                      # todos juntos 4 s, depois fade 2 s
            u = (te - T_DID) / 4; g = 1.0 if u < 1 else max(0.0, 1 - (u - 1) * 2)
            for _, a, f in ORDEM:
                v = f(); o[a] = [int(x * g) for x in v] if len(v) != 17 else v
            o.update(mv8(255 * g, dz=0.3, spread=0.5, color=AMARELO, prism=255, gobo=G_RAIOS, grot=140, zoom=230))
            if g == 0: o[LASER] = laser(0, on=False)
    return o

# ---- sACN ----
CID = uuid.uuid4().bytes; seq = 0
def packet(universe, data):
    global seq; seq = (seq + 1) & 255
    dmp = struct.pack(">HBBHHH", 0x7000 | (11 + len(data)), 0x02, 0xA1, 0, 1, 1 + len(data)) + b"\0" + data
    fr = struct.pack(">H", 0x7000 | (77 + len(dmp))) + struct.pack(">I", 2) + b"Feiticaria show medgrupo".ljust(64, b"\0") \
         + struct.pack(">BHBBH", 100, 0, seq, 0, universe) + dmp
    return struct.pack(">HH12s", 0x10, 0, b"ASC-E1.17\0\0\0") + struct.pack(">H", 0x7000 | (22 + len(fr))) \
           + struct.pack(">I", 4) + CID + fr
ifaces = sorted({ai[4][0] for ai in socket.getaddrinfo(socket.gethostname(), None, socket.AF_INET)} | {"127.0.0.1"})
socks = []
for ip in ifaces:
    s_ = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s_.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 1)
    s_.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton(ip))
    socks.append(s_)
def send(dmx):
    pk = packet(1, bytes(dmx))
    for s_ in socks:
        try: s_.sendto(pk, ("239.255.0.1", 5568))
        except OSError: pass
    socks[0].sendto(pk, ("127.0.0.1", 5568))

def apply(dmx, o):
    for a, vals in o.items():
        for i, v in enumerate(vals): dmx[a - 1 + i] = max(0, min(255, int(v)))

dmx = bytearray(512)
if sys.argv[1:2] == ["hold"]:                    # calibração: python show_medgrupo.py hold 59=0,0,255 260=255,0,0,0 ...
    for kv in sys.argv[2:]:
        a, xs = kv.split("="); apply(dmx, {int(a): [int(x) for x in xs.split(",")]})
    print("hold", sys.argv[2:], flush=True)
    while True: send(dmx); time.sleep(1 / 30)
if sys.argv[1:2] == ["laser"]:                   # calibração: python show_medgrupo.py laser 5=40 8=200
    v = laser(200)
    for kv in sys.argv[2:]:
        k, x = kv.split("="); v[int(k) - 1] = int(x)
    apply(dmx, {LASER: v}); print("laser", v, "(Ctrl+C para parar)", flush=True)
    while True: send(dmx); time.sleep(1 / 30)

PRE = 2.0                                        # pré-rolo: 1 s stop + 1 s replay, luz apagada, para o player reiniciar do zero
def run_once():
    dmx[VIDEO] = 0
    t0 = time.perf_counter()
    while (t := time.perf_counter() - t0) < DUR + PRE:
        dmx[VIDEO - 1] = 28 if t < 1 else (20 if t < PRE else (10 if t < PRE + VIDEO_END else 28))
        apply(dmx, look(max(0.0, t - PRE)) if t >= PRE else look(DUR)); send(dmx); time.sleep(1 / 30)
    dmx[VIDEO - 1] = 28; apply(dmx, look(DUR))
    for _ in range(15): send(dmx); time.sleep(1 / 30)
print(f"sACN universo 1 via {ifaces}; vídeo ch{VIDEO}; {DUR}s por passada", flush=True)
while True:
    run_once()
    if "loop" not in sys.argv[1:]: break
