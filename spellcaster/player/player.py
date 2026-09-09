# Player standalone: transporte (play/pause/stop/locate/loop) em thread propria sobre o Clock,
# timeline -> sACN / Art-Net / OSC, laser em thread separada a 20-30 kpps com safety() sempre aplicado.
# Transporte remoto por OSC: /spellcaster/play | pause | stop | locate f.
import json
import math
import os
import threading
import time

from .. import fixtures  # noqa: F401  (registra o resolvedor do track fixture)
from ..paths import SHOWS
from .. import show as showfile
from ..core.engine import Engine
from ..core.registry import command
from ..protocols import osc as osclib
from ..protocols.ilda.frame import Frame, Point, safety
from ..timeline.cues import CueList
from ..timeline.markers import find as find_markers
from ..timeline.model import Timeline

_EMPTY = {}


def _empty(t):
    return _EMPTY                    # Engine.tick(_empty, t) = so envia os universos ja escritos


class Player:
    """Player(show_dict, loop=False, outputs=None). outputs substitui as saidas DMX do .spell."""

    def __init__(self, sh, loop=False, outputs=None):
        self.show = sh
        self.base = sh.get("_dir", "")
        self.tl = Timeline(sh, self.base)
        self.cues = CueList(sh.get("cues", ()))
        self.fps = int(sh.get("fps", 30))
        self.duration = sh.get("duration")
        self.loop = loop
        self.laser_cfg, self.dac, self.osc_out = {}, None, None
        sacn_outs, art_outs = self._open(outputs)
        self.eng = Engine(sacn_outs, self.fps)
        self.clock = self.eng.clock
        self.art = Engine(art_outs, self.fps) if art_outs else None
        self.clip = self._load_clip()
        self._prev = 0.0
        self._run = False
        self._done = threading.Event()
        self._th = self._lth = self._osc_in = None

    # ---- saidas ----
    def _open(self, override):
        sacn_outs, art_outs = [], []
        for c in self.show.get("outputs", ()):
            k = c.get("type")
            if k == "sacn":
                from ..protocols.sacn import SacnOut
                sacn_outs.append(SacnOut(universes=c.get("universes", (1,)), priority=c.get("priority", 100),
                                         source_name=c.get("source_name", "Spellcaster"),
                                         interfaces=c.get("interfaces")))
            elif k == "artnet":
                from ..protocols.artnet import ArtNetOut
                art_outs.append(ArtNetOut(targets=c.get("targets"), broadcast=c.get("broadcast", True)))
            elif k == "osc":
                self.osc_out = osclib.OscOut(c.get("host", "127.0.0.1"), int(c["port"]))
            elif k == "laser":
                self.laser_cfg = c
                if c.get("dac"):
                    from ..protocols.ilda.etherdream import EtherDream
                    self.dac = EtherDream(c["dac"], int(c.get("port", 7765)))
        if override is not None:
            sacn_outs = list(override)
        return sacn_outs, art_outs

    def _load_clip(self):
        """Frames do track laser: clipe .ild ou gerador nomeado de ilda.generators."""
        if not self.tl.laser:
            return None
        sp = self.tl.laser[0].spec
        if sp.get("clip"):
            from ..protocols.ilda import ild
            return ild.read(os.path.join(self.base, sp["clip"]))
        return None                                  # gerador: frame calculado em laser_frame(t)

    # ---- transporte ----
    def start(self):
        self._run = True
        self._done.clear()
        self._th = threading.Thread(target=self._loop, daemon=True)
        self._th.start()
        if self.dac:
            self._lth = threading.Thread(target=self._laser_loop, daemon=True)
            self._lth.start()
        port = (self.show.get("transport") or {}).get("osc_port")
        if port:
            i = self._osc_in = osclib.OscIn(int(port))
            i.on("/spellcaster/play", lambda a, *g: self.play())
            i.on("/spellcaster/pause", lambda a, *g: self.pause())
            i.on("/spellcaster/stop", lambda a, *g: self.stop())
            i.on("/spellcaster/locate", lambda a, *g: self.locate(float(g[0]) if g else 0.0))
        return self

    def play(self):
        self._done.clear()
        self.clock.play()

    def pause(self):
        self.clock.pause()

    def stop(self):
        self.clock.stop()
        self.cues.reset()
        self._prev = 0.0
        self._done.set()

    def locate(self, t):
        self.clock.locate(float(t))
        self.cues.reset()
        self._prev = float(t)

    def wait(self, timeout=None):
        """Bloqueia ate stop() (ou fim do show). Espera curta em fatias para o Ctrl+C chegar."""
        end = None if timeout is None else time.perf_counter() + timeout
        while not self._done.wait(0.2):
            if end is not None and time.perf_counter() >= end:
                return False
        return True

    def close(self):
        self._run = False
        self.clock.stop()
        self._done.set()
        for th in (self._th, self._lth):
            if th:
                th.join(2)
        if self._osc_in:
            self._osc_in.close()
        if self.osc_out:
            self.osc_out.close()
        for out in self.eng.outputs + (self.art.outputs if self.art else []):
            out.close()

    # ---- loop de transporte ----
    def _loop(self):
        while self._run:
            if self.clock.state == "stop":
                time.sleep(0.005)
                continue
            self.clock.run(self._tick, self.duration)     # volta em stop() ou ao chegar na duracao
            if self._run and self.clock.state != "stop":
                if self.loop:
                    self.locate(0.0)
                else:
                    self.stop()

    def _tick(self, t):
        tl = self.tl
        tl.apply(self.eng.universes, t)
        if self.art:
            tl.apply(self.art.universes, t, tl.artnet)
        self._side(t)
        self._prev = t
        self.eng.tick(_empty, t)
        if self.art:
            self.art.tick(_empty, t)

    def _side(self, t):
        """Tracks de efeito colateral: OSC (envia quando o valor muda), media por OSC e cues."""
        for tr in self.tl.osc:
            v = tr.keys.value(t, tr.spec.get("args"))
            if v != tr.last:
                tr.last = v
                if self.osc_out is not None and v is not None:
                    if tr.type == "media":
                        self.osc_out.send(tr.spec["address"].rstrip("/") + "/" + str(v))
                    else:
                        self.osc_out.send(tr.spec["address"], *(v if isinstance(v, list) else [v]))
        for tr in self.tl.cue:
            for k in tr.keys.crossed(self._prev, t):
                self.cues.go(t, None if k.value in ("GO", "go", None) else k.value)
        if self.cues.cues:
            for (u, addr), vals in self.cues.update(t).items():
                self.eng.universes.get_or_create(u).set(addr, vals)

    # ---- laser ----
    def laser_frame(self, t):
        """Frame do laser em t: clipe (indice = t * fps) ou gerador, transformacoes por keyframe e safety()."""
        tr = self.tl.laser[0]
        fps = float(tr.spec.get("fps", self.fps))
        if self.clip:
            fr = self.clip[int(t * fps) % len(self.clip)]
        else:
            from ..protocols.ilda import generators
            fr = getattr(generators, tr.spec.get("gen", "medgrupo"))(t)
        x = tr.param("x", t, 0.0)
        y = tr.param("y", t, 0.0)
        s = tr.param("scale", t, 1.0)
        a = math.radians(tr.param("rot", t, 0.0))
        col = tr.param("color", t, None)
        ca, sa = math.cos(a), math.sin(a)
        pts = []
        for p in fr.points:
            px, py = p.x * s, p.y * s
            r, g, b = (p.r, p.g, p.b) if col is None else (col[0] * p.r / 255, col[1] * p.g / 255, col[2] * p.b / 255)
            pts.append(Point(px * ca - py * sa + x, px * sa + py * ca + y, r, g, b, p.blank))
        return safety(Frame(pts, fr.name), **self.laser_cfg.get("safety", {}))

    def _laser_loop(self):
        d = self.dac
        pps = int(self.laser_cfg.get("pps", 25000))
        chunk = max(1, pps // 50)                     # ~20 ms de pontos por comando
        try:
            d.connect()
            d.prepare()
        except OSError:
            return
        begun = False
        while self._run:
            if self.clock.state == "stop":
                time.sleep(0.02)
                continue
            pts = self.laser_frame(self.clock.time).points
            for i in range(0, len(pts), chunk):
                blk = pts[i:i + chunk]
                # ponytail: espera o buffer esvaziar por polling, igual ao EtherDream.play
                # ; trocar por low_water quando houver DAC real na bancada.
                while self._run and d.status["buffer_fullness"] + len(blk) > d.capacity:
                    time.sleep(len(blk) / pps / 2)
                    d.ping()
                if not self._run:
                    break
                try:
                    if d.send(blk)["ack"] != "a":
                        break
                    if not begun or d.status["playback_state"] == 0:
                        d.begin(pps)
                        begun = True
                except OSError:
                    return
        try:
            d.close()
        except OSError:
            pass


CURRENT = None                       # player em execucao neste processo (stop/pause/locate agem nele)


@command
def play_show(file: str, loop: bool = False):
    """Toca um show .spell (timeline, cues, laser) ate o fim ou Ctrl+C."""
    global CURRENT
    file = file if os.path.exists(file) else str(SHOWS / file)   # nome solto = shows/ ao lado do exe
    p = CURRENT = Player(showfile.load(file), loop=loop)
    outs = [c.get("type") for c in p.show.get("outputs", ())]
    print(f"{p.show.get('name', file)}: {len(p.tl.tracks)} tracks, {p.fps} fps, {p.duration}s, saidas {outs}", flush=True)
    p.start()
    p.play()
    try:
        p.wait()
    except KeyboardInterrupt:
        pass
    finally:
        p.close()
        CURRENT = None
    return p


@command
def stop():
    """Para o player em execucao neste processo."""
    if CURRENT:
        CURRENT.stop()


@command
def pause():
    """Pausa o player em execucao neste processo."""
    if CURRENT:
        CURRENT.pause()


@command
def locate(t: float):
    """Salta o player para o instante t (segundos)."""
    if CURRENT:
        CURRENT.locate(t)


@command
def markers(video: str, threshold: float = 0.3):
    """Lista os cortes de cena de um video (ffmpeg) ou os onsets de um .wav (JSON)."""
    ts = find_markers(video, threshold)
    print(json.dumps([round(x, 3) for x in ts]))
    return ts
