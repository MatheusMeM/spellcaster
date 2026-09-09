# Relógio único do engine: tick fixo com compensação de deriva (perf_counter) e transporte play/pause/stop/locate.
import threading, time


class Clock:
    def __init__(self, fps=30):
        self.fps = fps
        self._lock = threading.Lock()
        self._state = "stop"          # stop | play | pause
        self._pos = 0.0               # posição quando parado/pausado
        self._t0 = 0.0                # perf_counter correspondente a t=0 quando tocando

    @property
    def time(self):
        with self._lock:
            return time.perf_counter() - self._t0 if self._state == "play" else self._pos

    @property
    def state(self):
        return self._state

    def play(self):
        with self._lock:
            if self._state != "play":
                self._t0 = time.perf_counter() - self._pos
                self._state = "play"

    def pause(self):
        with self._lock:
            if self._state == "play":
                self._pos = time.perf_counter() - self._t0
            self._state = "pause"

    def stop(self):
        with self._lock:
            self._state, self._pos = "stop", 0.0

    def locate(self, t):
        with self._lock:
            self._pos = t
            self._t0 = time.perf_counter() - t

    def run(self, fn, duration=None):
        """Chama fn(t) a cada tick até stop() ou t >= duration. Em pause, fn segue sendo chamada com t congelado."""
        if self._state == "stop":
            self.play()
        period = 1.0 / self.fps
        nxt = time.perf_counter()
        while self._state != "stop":
            t = self.time
            if duration is not None and t >= duration:
                break
            fn(t)
            nxt += period
            d = nxt - time.perf_counter()
            if d > 0:
                time.sleep(d)
            else:
                nxt = time.perf_counter()   # ponytail: atrasou, não acumula o atraso ; trocar por fase fixa se precisar de sync externo (LTC/MTC)
