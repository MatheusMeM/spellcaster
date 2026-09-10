//! Helios DAC (Bitlasers) - USB, NOT implemented.
//!
//! The Helios does not speak network: it is a USB bulk device (VID 0x1209, PID 0xE500,
//! interface 0, endpoint OUT 0x02, IN 0x81). Without `hidapi`/`rusb` in the workspace there
//! is no way to open the device, and the PRD forbids a new dependency with no hardware to
//! measure against.
//
// ponytail: Helios requires hidapi ; add it when there is hardware
//
//! ## Frame layout (documented now so the port is mechanical later)
//!
//! Helios coordinates are 12 UNSIGNED bits, 0..4095, with the origin at the corner: the ILDA
//! point (+-32767) goes in as `(p + 32768) >> 4`. Color is 8 bits per channel.
//!
//! Bulk transfer on endpoint 0x02, `7 * n + 5` bytes:
//!
//! ```text
//! per point (7 bytes):
//!   b0 = x >> 4                      // high 8 bits of X
//!   b1 = ((x & 0x0F) << 4) | (y >> 8) // low 4 bits of X + high 4 of Y
//!   b2 = y & 0xFF
//!   b3 = r    b4 = g    b5 = b    b6 = i   // i = global intensity, usually max(r,g,b)
//! tail (5 bytes):
//!   pps      u16 little-endian   (points per second of the frame)
//!   n        u16 little-endian   (number of points)
//!   flags    u8   bit0 START_IMMEDIATELY, bit1 SINGLE_MODE, bit2 DONT_BLOCK
//! ```
//!
//! Control commands go through the same endpoint in 2-byte packets: `[0x01, 0]` stop,
//! `[0x02, 0]` shutter off, `[0x03, 0]` status request (reply `[0x83, ready]` on 0x81),
//! `[0x04, 0]` firmware version, `[0x07, shutter]` shutter set.
//!
//! Send order: status request -> wait for `ready` -> frame. The `Feed` already serializes
//! this because only one send happens at a time on the DAC thread.

use crate::frame::Point;

/// Converts an ILDA point to the Helios 12-bit pair. Tested; it is the only part of the
/// conversion that does not depend on USB.
pub fn to_12bit(p: &Point) -> (u16, u16) {
    let f = |v: i16| (((v as i32) + 32768) >> 4).clamp(0, 4095) as u16;
    (f(p.x), f(p.y))
}

/// Serializes a frame in the bulk format described above. No I/O: it exists for the day the
/// hardware arrives and for the layout test.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_of_12_bits_and_of_the_frame() {
        assert_eq!(
            to_12bit(&Point::new(-32767.0, 0.0, 0, 0, 0, false)),
            (0, 2048)
        );
        assert_eq!(
            to_12bit(&Point::new(32767.0, 32767.0, 0, 0, 0, false)),
            (4095, 4095)
        );
        let mut out = Vec::new();
        encode_frame(
            &[Point::new(0.0, 0.0, 10, 20, 30, false)],
            30_000,
            1,
            &mut out,
        );
        assert_eq!(out.len(), 12);
        assert_eq!(&out[..7], &[0x80, 0x08, 0x00, 10, 20, 30, 30][..]);
        assert_eq!(&out[7..], &[0x30, 0x75, 1, 0, 1][..]); // 30000, n=1, flags
                                                           // a blanked point goes out black
        encode_frame(
            &[Point::new(0.0, 0.0, 255, 255, 255, true)],
            1000,
            0,
            &mut out,
        );
        assert_eq!(&out[3..7], &[0, 0, 0, 0][..]);
    }
}
