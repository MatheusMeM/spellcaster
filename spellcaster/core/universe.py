# Buffers DMX: um Universe por número, 512 canais, endereço 1-based.


class Universe:
    def __init__(self, number):
        self.number = number
        self.data = bytearray(512)

    def set(self, addr, values):
        """Escreve values a partir de addr (1-based); clamp 0-255; ignora o que passar de 512."""
        i = addr - 1
        vals = bytes(max(0, min(255, int(v))) for v in values)[: max(0, 512 - i)]
        self.data[i:i + len(vals)] = vals


class Universes(dict):
    def get_or_create(self, number):
        u = self.get(number)
        if u is None:
            u = self[number] = Universe(number)
        return u
