# The engine's only clock: fixed tick with drift compensation (perf_counter) and play/pause/stop/locate transport.
import threading, time


class Clock:
    def __init__(self, fps=30):
        self.fps = fps
        self._lock = threading.Lock()
        self._state = "stop"          # stop | play | pause
        self._pos = 0.0               # position while stopped/paused
        self._t0 = 0.0                # perf_counter matching t=0 while playing

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
        """Calls fn(t) on every tick until stop() or t >= duration. While paused, fn keeps being called with t frozen."""
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
                nxt = time.perf_counter()   # ponytail: it fell behind, does not accumulate the lag ; swap for fixed phase if external sync is needed (LTC/MTC)
