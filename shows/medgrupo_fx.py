# Show MED GRUPO RJ (plenaria, 09/2026) em fixtures nomeadas: nenhum indice de canal no codigo do show.
# Mesma dramaturgia de shows/medgrupo.py e os MESMOS 512 bytes do universo 1 (tests/test_fixtures.py compara
# frame a frame, 30 fps, do 0 ao fim). Uso: spell play shows/medgrupo_fx.py [--loop]
import bisect, math

from spellcaster.fixtures.group import Group
from spellcaster.fixtures.patch import Patch

# ---- patch (lido do Capture, projeto medgrupo_rj.c2p, 08/09/2026) ----
P = Patch()


def _add(prefix, profile, addrs):
    return [P.add(f"{prefix}{i + 1}", profile, 1, a).name for i, a in enumerate(addrs)]


PAR_VARA = _add("par_vara_", "par_rgb_3", (1, 4, 7, 10))          # Brisas BX-940: 5 ch fisicos, patch de 3 em 3 => 3 ch uteis
LASER = P.add("laser", "laser_butrym_10", 1, 13).name
FRESNEL = _add("fresnel_", "dimmer_1", (29, 34, 39, 44, 49, 54))
PAR_PALCO = _add("par_palco_", "par_rgb_3", (187, 190, 193, 196, 199, 202))
BAR = _add("bar_", "bar_wled_6", range(205, 205 + 6 * 8, 6))
WASH = _add("wash_", "wash_bx402_4", range(260, 260 + 4 * 10, 4))
MV_FUNDO = _add("mv_fundo_", "bsw_scorpio_17", (300, 320, 340, 360))   # vara do fundo, junto do telao
MV_FRENTE = _add("mv_frente_", "bsw_scorpio_17", (380, 404, 424, 444))  # chao da boca de cena, apontando para cima
VIDEO = P.add("video", "media_player_2", 1, 400).name

# ---- geometria dos movings, CALIBRADA no Capture (08/09/2026, camera Frontal) ----
# pan 128 / tilt 128 = reto para baixo nos 8. O tilt inclina o raio LATERALMENTE (fundo: tilt maior = direita;
# frente: tilt maior = esquerda) e o pan gira essa inclinacao: pan 171 (+90 graus) = plateia, pan 85 = tela.
# Os BSW estavam patcheados de 16 em 16 e o ch17 de cada um recebia o PAN do vizinho: por isso pareciam malucos.
# Hoje o Patch recusa esse espacamento na hora (17 ch em 300/316 = PatchError).
BRANCO, VERMELHO, VERDE, AMARELO, AZUL = "branco", "vermelho", "verde", "amarelo", "azul"
G_PONTOS, G_ANEL, G_RAIOS = "pontos", "anel", "raios"
ABERTO, STROBE = "aberto", "strobe"
ZMIN, ZMAX = 80, 255                                     # 13 ... 36 graus: nunca abaixo de 13, preferir aberto
DEF_BSW = dict(color=BRANCO, gobo=0, grot=0, gobo2=0, shutter=ABERTO, dim=0, focus=128, zoom=ZMAX, prism=0, frost=0)
FUNDO = Group(P, MV_FUNDO, "fundo", pan=128, tilt=128, amp=38, lado=+1, zmin=-1.2,
              zoom_min=ZMIN, zoom_max=ZMAX, defaults=DEF_BSW)
FRENTE = Group(P, MV_FRENTE, "frente", pan=128, tilt=128, amp=38, lado=-1, zmin=-0.15,
               zoom_min=ZMIN, zoom_max=ZMAX, defaults=DEF_BSW)     # dz<0 aqui mira o telao: evitar


def mv8(**kw):
    FUNDO.aim(**kw); FRENTE.aim(**kw)


def mv_solo(name):
    """Epilogo: uma unidade sozinha, reto para baixo, gobo girando. O resto do grupo so anda a histerese."""
    (FUNDO if name in MV_FUNDO else FRENTE).aim(dim=255, color=AMARELO, gobo=G_RAIOS, grot=140, zoom=200, only=name)


def blackout(name):
    P.set(name, **dict.fromkeys(P.fixtures[name].chans, 0))


# ---- valores por nome de canal (dict, para o epilogo poder escalar tudo por um ganho) ----
def d_par(v, c): return {"r": int(v * c[0] / 255), "g": int(v * c[1] / 255), "b": int(v * c[2] / 255)}
def d_wash(v, c): return {**d_par(v, c), "w": int(v * min(c) / 255)}
def d_bar(v, r=1, g=1, b=1):
    return {"r": int(v * r), "g": int(v * g), "b": int(v * b), "w": int(v * min(r, g, b)), "uv": 0, "a": 0}
def d_barc(v, c): return d_bar(v, c[0] / 255, c[1] / 255, c[2] / 255)
def d_laser(size, posh=128, posv=128, fig=0, color=107, on=True):
    if not on: return dict.fromkeys(("on", "strobe", "size", "posh", "posv", "fig", "color", "scan", "lib", "auto"), 0)
    return {"on": 255, "strobe": 255, "size": int(size), "posh": int(posh), "posv": int(posv), "fig": int(fig),
            "color": int(color), "scan": 0, "lib": 0, "auto": 0}   # este perfil nao tem canal de rotacao


def beat(t, f=1.0, k=4): return math.exp(-k * ((t * f) % 1.0))   # decaimento por batida
def ease(a, b, u): u = max(0.0, min(1.0, u)); u = u * u * (3 - 2 * u); return a + (b - a) * u
def sin01(t, f, ph=0): return 0.5 + 0.5 * math.sin(2 * math.pi * f * t + ph)
def lerp3(c1, c2, u): return tuple(c1[k] + (c2[k] - c1[k]) * max(0, min(1, u)) for k in range(3))


# Cores da comunicacao MED GRUPO (redes sociais / video): amarelo e verde-lima sobre fundo escuro, branco de apoio.
C_AMARELO, C_LIMA, C_NAVY, C_BRANCO = (255, 215, 0), (150, 255, 0), (0, 25, 110), (255, 255, 255)

VIDEO_END = 46.8
CUES = [12.0, 14.8, 17.72, 19.72, 21.36, 23.36, 26.84, 29.5]   # cortes do video (ffmpeg scene) no trecho das cidades
PICO = 33.12                                                   # corte para o logo
ORDEM = [("Par LED vara", n, lambda: d_par(255, C_AMARELO)) for n in PAR_VARA] + \
        [("Par LED palco", n, lambda: d_par(255, C_LIMA)) for n in PAR_PALCO] + \
        [("Fresnel", n, lambda: {"dim": 255}) for n in FRESNEL] + \
        [("Moving BSW", n, (lambda n=n: mv_solo(n))) for n in MV_FUNDO + MV_FRENTE] + \
        [("Ribalta WLED", n, lambda: d_barc(255, C_AMARELO)) for n in BAR] + \
        [("Ribalta BX-402 plateia", n, lambda: d_wash(255, C_LIMA)) for n in WASH] + \
        [("Laser", LASER, lambda: d_laser(200, color=107))]
T_CADA = 0.7; T_DID = 1.0 + len(ORDEM) * T_CADA           # epilogo: 1 s de breu + 0,7 s por aparelho
DUR = VIDEO_END + T_DID + 6                               # + todos juntos 4 s + fade 2 s


def _look_luz(t):
    """Dramaturgia 5E sobre o video (46,8 s) + epilogo didatico:
    EXCITEMENT 0-10,5 contagem: revelacao por batida, os movings descem gobos devagar, amarelo/verde da marca.
    ENTRY 10,5-12 flash: limiar - flash amarelo (como o video) com strobe curto, depois branco no '0'; prisma nos 8.
    ENGAGEMENT 12-33,1 cidades: cada troca de pose cai num corte do video (CUES); fundo (vara) e frente (chao,
      para cima) alternam leques, cruzamentos e ondas; 23,36 (Cachoeiro) = pausa contemplativa, 26,84 retomada.
    PICO 33,12-40 logo: os 8 como unidade, zoom fechado em 13 graus so neste instante, depois abre.
    EXIT 40-46,8: tudo fecha ao centro (frost) e apaga; fresnel quente por ultimo.
    EPILOGO 46,8+: breu, cada aparelho acende sozinho (ordem do patch), depois todos juntos, fade."""
    if t < 10.5:                                                   # EXCITEMENT (contagem 10...1 ate 10,5 s)
        b = beat(t); k = int(t)
        for n in FRESNEL: P.set(n, dim=int(120 * b) if k < 2 else 30)                                # 0-2 s: fresnel pulsa
        for i, n in enumerate(BAR): P.set(n, **d_barc(255 * b if k >= 2 and k % 2 == i % 2 else 0, C_AMARELO))
        for i, n in enumerate(WASH): P.set(n, **d_wash(180 * b if k >= 3 and (i // 5) == k % 2 else 0, C_LIMA))
        for n in PAR_VARA + PAR_PALCO: P.set(n, **d_par((40 + 160 * b) if k >= 4 else 0, C_NAVY))
        FUNDO.aim(dim=int(ease(0, 220, (t - 5) / 1) * (0.6 + 0.4 * b)), dz=-0.3, spread=ease(0, 0.6, (t - 5) / 4),
                  color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230)                            # 5 s+: fundo abre leque
        FRENTE.aim(dim=int(ease(0, 220, (t - 7) / 1) * (0.6 + 0.4 * b)), dz=+0.3, spread=ease(0, 0.6, (t - 7) / 3),
                   color=VERDE, gobo=G_ANEL, grot=120, zoom=230)                               # 7 s+: frente abre leque
        P.set(LASER, **d_laser(100 + 80 * b if k >= 1 else 0, on=k >= 1))
        if t >= 10.1:                                              # respiro: 0,4 s de breu antes do flash amarelo
            for n in FRESNEL + BAR + WASH + PAR_VARA + PAR_PALCO + [LASER]: blackout(n)
            FUNDO.aim(dim=0, dz=-0.3, spread=0.6, color=AMARELO, gobo=G_PONTOS, grot=135, zoom=230)
            FRENTE.aim(dim=0, dz=+0.3, spread=0.6, color=VERDE, gobo=G_ANEL, grot=120, zoom=230)
    elif t < 12.0:                                                 # ENTRY - flash amarelo (10,5-11,3) e '0' (11,5-12)
        u = (t - 10.5) / 1.5; st = STROBE if u < 0.5 else ABERTO; c = C_AMARELO if u < 0.55 else C_BRANCO
        for n in PAR_VARA + PAR_PALCO: P.set(n, **d_par(255, c))
        for n in FRESNEL: P.set(n, dim=255)
        for n in BAR: P.set(n, **d_barc(255, c))
        for n in WASH: P.set(n, **d_wash(255, c))
        mv8(dim=255, dz=ease(-0.3, -0.9, u), spread=ease(0.6, 0.2, u), color=AMARELO if u < 0.55 else BRANCO,
            prism=255, zoom=255, shutter=st)
        P.set(LASER, **d_laser(ease(80, 255, u), color=255))
    elif t < PICO:                                                 # ENGAGEMENT - blocos ancorados nos CUES
        seg = bisect.bisect_right(CUES, t) - 1; t0 = CUES[seg]; t1 = CUES[seg + 1] if seg + 1 < len(CUES) else PICO
        us = (t - t0) / (t1 - t0); u = (t - 12.0) / (PICO - 12.0)
        cp = [C_AMARELO, C_LIMA, C_NAVY][seg % 3]; cs = [AMARELO, VERDE, BRANCO][seg % 3]
        gb = [G_PONTOS, G_RAIOS, G_ANEL][seg % 3]
        for i, n in enumerate(PAR_VARA + PAR_PALCO): P.set(n, **d_par(255 * (0.3 + 0.7 * sin01(t, 0.25, i)), cp))
        for i, n in enumerate(FRESNEL): P.set(n, dim=int(80 * sin01(t, 0.25, i)) if seg % 2 == 0 else 0)
        for i, n in enumerate(BAR): P.set(n, **d_barc(255 if int(t * 2) % 8 == i else 40, cp))
        for i, n in enumerate(WASH):
            P.set(n, **d_wash(255 if int(t * 2) % 8 == i else 70, [C_LIMA, C_AMARELO, C_NAVY][seg % 3]))
        if seg == 0:                # 12,72 lista entra: fundo leque respirando na tela; frente raios verticais
            FUNDO.aim(dim=230, dz=-0.5, spread=0.5 + 0.3 * math.sin(2 * math.pi * 0.3 * t), color=cs, gobo=gb,
                      grot=135, zoom=240)
            FRENTE.aim(dim=200, dz=+0.1, spread=0.2, color=cs, gobo=gb, grot=120, zoom=200)
        elif seg == 1:              # 14,8 Rio: frente varre para cima com prisma; fundo wash (frost) nas bordas
            FRENTE.aim(dim=240, dz=ease(0.1, 0.9, us), spread=0.4, color=cs, gobo=gb, prism=255, zoom=220)
            FUNDO.aim(dim=180, dz=-0.4, spread=0.6, color=cs, frost=255, zoom=255)
        elif seg == 2:              # 17,72: os 8 em onda lenta percorrendo da esquerda para a direita
            mv8(dim=240, dz=0.3, spread=0.5, color=cs, gobo=gb, grot=130, wave=0.4, t=t, wph=0.8, wf=0.4, zoom=230)
        elif seg == 3:              # 19,72: fundo raios cruzados com prisma; frente leque aberto
            FUNDO.aim(dim=240, dz=-0.3, spread=-0.7, color=cs, gobo=gb, prism=255, zoom=200)
            FRENTE.aim(dim=200, dz=+0.4, spread=0.8, color=cs, gobo=gb, grot=120, zoom=255)
        elif seg == 4:              # 21,36: os 8 giram em unissono da tela para a plateia, leque espelhado
            mv8(dim=250, dz=ease(-0.6, 0.8, us), spread=0.5, color=cs, gobo=gb, grot=140, zoom=230)
        elif seg == 5:              # 23,36 CACHOEIRO: PAUSA contemplativa - frente apaga, fundo frost lento
            for n in PAR_VARA + PAR_PALCO: P.set(n, **d_par(90, C_NAVY))
            for n in FRESNEL: P.set(n, dim=0)
            for n in BAR: P.set(n, **d_barc(30, C_AMARELO))
            for n in WASH: P.set(n, **d_wash(50, C_NAVY))
            FUNDO.aim(dim=int(ease(240, 140, us * 2)), dz=-0.2, spread=0.6, color=AMARELO, frost=255, zoom=255)
            FRENTE.aim(dim=int(ease(200, 0, us * 2)), dz=+0.3, spread=0.5, color=cs, frost=255, zoom=255)
        elif seg == 6:              # 26,84: retomada - os 8 sobem juntos, gobo girando, leque abrindo
            mv8(dim=int(ease(0, 250, us * 3)), dz=ease(-0.2, 0.6, us), spread=ease(0.2, 0.6, us), color=AMARELO,
                gobo=G_RAIOS, grot=140, zoom=230)
        else:                       # 29,5 "o destino final e a aprovacao": cruzam e abrem com prisma, ate o logo
            mv8(dim=255, dz=0.4, spread=-0.7 + 1.3 * sin01(us, 1, -math.pi / 2), color=AMARELO, gobo=G_RAIOS,
                grot=140, prism=255, zoom=240)
        P.set(LASER, **d_laser(150, posh=170, posv=128 + 70 * math.sin(u * 2 * math.pi * 3)))
    elif t < 40:                                                   # PICO - logo entra em 33,12: 8 como unidade
        u = (t - PICO) / (40 - PICO); b = beat(t, 2)
        for n in PAR_VARA + PAR_PALCO: P.set(n, **d_par(255, lerp3(C_LIMA, C_AMARELO, b)))
        for n in FRESNEL: P.set(n, dim=int(150 + 100 * b))
        for n in BAR: P.set(n, **d_barc(255 * b, C_AMARELO))
        for i, n in enumerate(WASH): P.set(n, **d_wash(255 * (b if i % 2 == int(t * 2) % 2 else 0.3), C_LIMA))
        mv8(dim=255, dz=ease(-0.8, 0.8, u), spread=0.3 + 0.3 * math.sin(2 * math.pi * 0.5 * t),
            color=AMARELO if b > 0.5 else BRANCO, prism=255, gobo=G_RAIOS, grot=140,
            zoom=ease(ZMIN, ZMAX, (u - 0.2) / 0.6), shutter=STROBE if u < 0.15 else ABERTO)   # unico 13 graus
        P.set(LASER, **d_laser(ease(255, 140, u), color=255))
    elif t < VIDEO_END:                                            # EXIT
        u = (t - 40) / 6.8
        for n in PAR_VARA + PAR_PALCO: P.set(n, **d_par(ease(150, 0, u * 1.5), C_AMARELO))
        for n in BAR: P.set(n, **d_barc(0, C_AMARELO))
        for n in WASH: P.set(n, **d_wash(ease(120, 0, u * 1.2), C_LIMA))
        mv8(dim=int(ease(220, 0, u * 1.3)), dz=ease(0.5, 0, u), spread=ease(0.5, 0, u), color=AMARELO,
            frost=255, zoom=255)
        for n in FRESNEL: P.set(n, dim=int(ease(200, 0, max(0, (u - 0.5) * 2))))
        P.set(LASER, **d_laser(ease(140, 0, u * 1.3), on=u < 0.7))
    else:                                                          # EPILOGO didatico
        te = t - VIDEO_END
        for n in PAR_VARA + PAR_PALCO: P.set(n, r=0, g=0, b=0)
        for n in FRESNEL: P.set(n, dim=0)
        for n in BAR: P.set(n, **d_bar(0))
        for n in WASH: P.set(n, r=0, g=0, b=0, w=0)
        mv8(dim=0)
        P.set(LASER, **d_laser(0, on=False))
        if te < 1.0: return                                        # breu
        idx = int((te - 1.0) / T_CADA)
        if idx < len(ORDEM):                                       # um aparelho por vez
            _, name, f = ORDEM[idx]; d = f()
            if d: P.set(name, **d)                                 # moving ja escreveu sozinho (mv_solo)
        else:                                                      # todos juntos 4 s, depois fade 2 s
            u = (te - T_DID) / 4; gain = 1.0 if u < 1 else max(0.0, 1 - (u - 1) * 2)
            for _, name, f in ORDEM:
                d = f()
                if d: P.set(name, **{c: int(x * gain) for c, x in d.items()})
            mv8(dim=int(255 * gain), dz=0.3, spread=0.5, color=AMARELO, prism=255, gobo=G_RAIOS, grot=140, zoom=230)
            if gain == 0: P.set(LASER, **d_laser(0, on=False))


# ---- pre-rolo de video (1 s stop + 1 s replay) e canal de controle do player ----
PRE = 2.0
DUR_LUZ = DUR
DUR = DUR_LUZ + PRE
# ponytail: look devolve sempre o MESMO dict apontando para o bytearray do universo (zero alocacao por frame);
# Engine.apply copia os 512 bytes. Trocar por delta so se 512 int() por frame aparecerem no perfil.
FRAME = {(u.number, 1): u.data for u in P.universes.values()}


def look(t):
    _look_luz(t - PRE if t >= PRE else DUR_LUZ)
    P.set(VIDEO, ctrl="stop" if t < 1 else ("replay" if t < PRE else ("play" if t < PRE + VIDEO_END else "stop")),
          clip=0)
    return FRAME
