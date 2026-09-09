//! Helios DAC (Bitlasers) — USB, NAO implementado.
//!
//! O Helios nao fala rede: e um dispositivo USB bulk (VID 0x1209, PID 0xE500, interface 0,
//! endpoint OUT 0x02, IN 0x81). Sem `hidapi`/`rusb` no workspace nao ha como abrir o
//! dispositivo, e o PRD proibe dependencia nova sem hardware para medir.
//
// ponytail: Helios exige hidapi ; adicionar quando houver hardware
//
//! ## Layout do frame (documentado agora para o porte ser mecanico depois)
//!
//! Coordenadas do Helios sao 12 bits SEM sinal, 0..4095, com a origem no canto: o ponto
//! ILDA (+-32767) entra como `(p + 32768) >> 4`. Cor e 8 bits por canal.
//!
//! Transferencia bulk no endpoint 0x02, `7 * n + 5` bytes:
//!
//! ```text
//! por ponto (7 bytes):
//!   b0 = x >> 4                      // 8 bits altos de X
//!   b1 = ((x & 0x0F) << 4) | (y >> 8) // 4 bits baixos de X + 4 altos de Y
//!   b2 = y & 0xFF
//!   b3 = r    b4 = g    b5 = b    b6 = i   // i = intensidade global, normalmente max(r,g,b)
//! cauda (5 bytes):
//!   pps      u16 little-endian   (pontos por segundo do frame)
//!   n        u16 little-endian   (numero de pontos)
//!   flags    u8   bit0 START_IMMEDIATELY, bit1 SINGLE_MODE, bit2 DONT_BLOCK
//! ```
//!
//! Comandos de controle vao pelo mesmo endpoint em pacotes de 2 bytes: `[0x01, 0]` stop,
//! `[0x02, 0]` shutter off, `[0x03, 0]` status request (resposta `[0x83, ready]` no 0x81),
//! `[0x04, 0]` firmware version, `[0x07, shutter]` shutter set.
//!
//! Ordem de envio: status request -> esperar `ready` -> frame. O `Feed` ja serializa isso
//! porque so um envio acontece por vez na thread do DAC.

use std::io;

use crate::dac::Dac;
use crate::frame::Point;

/// Converte um ponto ILDA para o par de 12 bits do Helios. Testado; e a unica parte da
/// conversao que nao depende do USB.
pub fn to_12bit(p: &Point) -> (u16, u16) {
    let f = |v: i16| (((v as i32) + 32768) >> 4).clamp(0, 4095) as u16;
    (f(p.x), f(p.y))
}

/// Serializa um frame no formato bulk descrito acima. Sem I/O: existe para o dia do
/// hardware e para o teste do layout.
pub fn encode_frame(points: &[Point], pps: u32, flags: u8, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(points.len() * 7 + 5);
    for p in points {
        let (x, y) = to_12bit(p);
        let (r, g, b) = if p.blank { (0, 0, 0) } else { (p.r, p.g, p.b) };
        out.push((x >> 4) as u8);
        out.push((((x & 0x0F) << 4) | (y >> 8)) as u8);
        out.push((y & 0xFF) as u8);
        out.extend_from_slice(&[r, g, b, r.max(g).max(b)]);
    }
    out.extend_from_slice(&(pps.min(65535) as u16).to_le_bytes());
    out.extend_from_slice(&(points.len().min(65535) as u16).to_le_bytes());
    out.push(flags);
}

/// Placeholder do DAC USB. `open` sempre falha: sem hardware nao ha o que testar.
pub struct Helios {
    pub index: u32,
}

impl Helios {
    pub fn open(index: u32) -> io::Result<Helios> {
        let _ = index;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Helios exige USB (hidapi/rusb): nao compilado nesta build",
        ))
    }
}

impl Dac for Helios {
    fn name(&self) -> String {
        format!("helios:{}", self.index)
    }

    fn begin(&mut self, _pps: u32) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Helios nao implementado"))
    }

    fn send(&mut self, _points: &[Point]) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Helios nao implementado"))
    }

    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_de_12_bits_e_do_frame() {
        assert_eq!(to_12bit(&Point::new(-32767.0, 0.0, 0, 0, 0, false)), (0, 2048));
        assert_eq!(to_12bit(&Point::new(32767.0, 32767.0, 0, 0, 0, false)), (4095, 4095));
        let mut out = Vec::new();
        encode_frame(&[Point::new(0.0, 0.0, 10, 20, 30, false)], 30_000, 1, &mut out);
        assert_eq!(out.len(), 12);
        assert_eq!(&out[..7], &[0x80, 0x08, 0x00, 10, 20, 30, 30][..]);
        assert_eq!(&out[7..], &[0x30, 0x75, 1, 0, 1][..]); // 30000, n=1, flags
        // ponto apagado sai preto
        encode_frame(&[Point::new(0.0, 0.0, 255, 255, 255, true)], 1000, 0, &mut out);
        assert_eq!(&out[3..7], &[0, 0, 0, 0][..]);
    }

    #[test]
    fn open_recusa_sem_usb() {
        assert!(Helios::open(0).is_err());
    }
}
