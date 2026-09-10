# Engine loop: look(t) -> universes -> outputs, on every Clock tick.
from .clock import Clock
from .universe import Universes


class Engine:
    def __init__(self, outputs, fps=30):
        self.outputs = list(outputs)
        self.universes = Universes()
        self.clock = Clock(fps)

    def apply(self, frame):
        """frame: {addr: [..]} (universe 1) or {(universe, addr): [..]}."""
        for key, vals in frame.items():
            u, a = key if isinstance(key, tuple) else (1, key)
            self.universes.get_or_create(u).set(a, vals)

    def tick(self, look, t):
        self.apply(look(t))
        for u in self.universes.values():
            data = bytes(u.data)
            for out in self.outputs:
                out.send(u.number, data)

    def run(self, look, duration=None):
        self.clock.run(lambda t: self.tick(look, t), duration)
