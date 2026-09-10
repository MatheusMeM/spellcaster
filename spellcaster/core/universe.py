# DMX buffers: one Universe per number, 512 channels, 1-based address.


class Universe:
    def __init__(self, number):
        self.number = number
        self.data = bytearray(512)

    def set(self, addr, values):
        """Writes values starting at addr (1-based); clamps to 0-255; drops anything past 512."""
        i = addr - 1
        vals = bytes(max(0, min(255, int(v))) for v in values)[: max(0, 512 - i)]
        self.data[i:i + len(vals)] = vals


class Universes(dict):
    def get_or_create(self, number):
        u = self.get(number)
        if u is None:
            u = self[number] = Universe(number)
        return u
