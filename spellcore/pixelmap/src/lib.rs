//! `pixelmap` - Spellcaster R3: samples an RGB/RGBA frame and writes the DMX bytes of each
//! physical pixel, one 512-channel buffer per universe, ready for
//! `protocols::Output::send`.
//!
//! Self-contained crate: it depends neither on `engine` nor on `protocols`. The frame source
//! is a `&[u8]` - when R2 (media) exists, the texture pool hands over the same slice.
//!
//! ```
//! use pixelmap::{Fixture, Frame, Mapper, Order, PixelMap};
//! let px = [10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120]; // 2x2 RGB
//! let mut m = Mapper::new(&PixelMap {
//!     name: "wall".into(),
//!     source: "media/1".into(),
//!     fixtures: vec![Fixture { universe: 1, channel: 1, order: Order::Rgb, u: 0.25, v: 0.25 }],
//! });
//! m.render(&Frame::rgb(2, 2, &px).unwrap());
//! for (universe, data) in m.frames() {
//!     assert_eq!((universe, &data[..3]), (1, &[10u8, 20, 30][..]));
//! }
//! ```
//!
//! In the `.spell` the block lives outside `tracks` and survives the `engine::Show`
//! round-trip (field `extra`, `#[serde(flatten)]`). The caller deserializes with serde_json:
//!
//! ```json
//! {"pixelmaps": [{"name": "wall", "source": "media/1",
//!                 "fixtures": [{"universe": 10, "channel": 1, "order": "grb",
//!                               "u": 0.0, "v": 0.0}]}]}
//! ```

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Color channel order of a physical pixel.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    #[default]
    Rgb,
    Grb,
    Rgbw,
}

impl Order {
    /// How many DMX channels the pixel takes.
    pub const fn channels(self) -> usize {
        match self {
            Order::Rgbw => 4,
            _ => 3,
        }
    }

    /// Writes the color into `out` (up to 4 bytes), truncating to what fits.
    ///
    /// RGBW extracts the white as `w = min(r, g, b)` and subtracts it from the three - it is
    /// the conversion that does not blow past the total output of the fixture.
    // ponytail: fixed white extraction ; make it a Fixture field when a fixture shows up that
    // expects an independent W (separate warm white, CCT).
    #[inline]
    fn write(self, r: u8, g: u8, b: u8, out: &mut [u8]) {
        let px = match self {
            Order::Rgb => [r, g, b, 0],
            Order::Grb => [g, r, b, 0],
            Order::Rgbw => {
                let w = r.min(g).min(b);
                [r - w, g - w, b - w, w]
            }
        };
        let n = out.len().min(self.channels());
        out[..n].copy_from_slice(&px[..n]);
    }
}

/// One physical pixel: where it reads in the frame (normalized `u`, `v`) and where it writes
/// in DMX.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Fixture {
    /// Universe, 1-based (sACN).
    pub universe: u16,
    /// First DMX channel, 1-based.
    pub channel: u16,
    #[serde(default)]
    pub order: Order,
    /// Normalized coordinate in the frame; outside 0..1 it is clamped to the edge.
    pub u: f32,
    pub v: f32,
}

/// One map from the `.spell`: one video source and the physical pixels that read it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PixelMap {
    #[serde(default)]
    pub name: String,
    /// Frame source, `"media/<id>"`. Resolved by R2; the crate does not interpret it.
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub fixtures: Vec<Fixture>,
}

/// How the pixel reads the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Sampling {
    /// Nearest pixel. Default: one LED is one pixel, there is nothing to interpolate.
    #[default]
    Nearest,
    /// Bilinear between the 4 neighbors. For a map denser than the source.
    Bilinear,
}

/// Borrowed input frame: `stride` = 3 (RGB) or 4 (RGBA), contiguous rows.
#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    pub w: u32,
    pub h: u32,
    pub stride: usize,
    px: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn rgb(w: u32, h: u32, px: &'a [u8]) -> Result<Frame<'a>, String> {
        Frame::new(w, h, 3, px)
    }

    pub fn rgba(w: u32, h: u32, px: &'a [u8]) -> Result<Frame<'a>, String> {
        Frame::new(w, h, 4, px)
    }

    /// Trust boundary: the frame comes from the decoder and is validated ONCE here; the hot
    /// path reads without checking again.
    pub fn new(w: u32, h: u32, stride: usize, px: &'a [u8]) -> Result<Frame<'a>, String> {
        if w == 0 || h == 0 {
            return Err(format!("frame {}x{}: zero dimension", w, h));
        }
        if stride != 3 && stride != 4 {
            return Err(format!("stride {}: only RGB (3) or RGBA (4)", stride));
        }
        let need = w as usize * h as usize * stride;
        if px.len() < need {
            return Err(format!(
                "frame {}x{} stride {}: {} bytes, needs {}",
                w,
                h,
                stride,
                px.len(),
                need
            ));
        }
        Ok(Frame { w, h, stride, px })
    }

    #[inline]
    fn texel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let i = (y as usize * self.w as usize + x as usize) * self.stride;
        (self.px[i], self.px[i + 1], self.px[i + 2])
    }

    /// Samples at a normalized coordinate. `u`/`v` outside 0..1 are clamped to the edge.
    #[inline]
    pub fn sample(&self, u: f32, v: f32, s: Sampling) -> (u8, u8, u8) {
        let (u, v) = (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
        let (xmax, ymax) = (self.w - 1, self.h - 1);
        match s {
            Sampling::Nearest => {
                let x = ((u * self.w as f32) as u32).min(xmax);
                let y = ((v * self.h as f32) as u32).min(ymax);
                self.texel(x, y)
            }
            Sampling::Bilinear => {
                // texel center at u = (x + 0.5) / w
                let fx = (u * self.w as f32 - 0.5).max(0.0);
                let fy = (v * self.h as f32 - 0.5).max(0.0);
                let x0 = (fx as u32).min(xmax);
                let y0 = (fy as u32).min(ymax);
                let (x1, y1) = ((x0 + 1).min(xmax), (y0 + 1).min(ymax));
                let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
                let (a, b, c, d) = (
                    self.texel(x0, y0),
                    self.texel(x1, y0),
                    self.texel(x0, y1),
                    self.texel(x1, y1),
                );
                let mix = |p: u8, q: u8, r: u8, t: u8| -> u8 {
                    let top = p as f32 + (q as f32 - p as f32) * tx;
                    let bot = r as f32 + (t as f32 - r as f32) * tx;
                    (top + (bot - top) * ty + 0.5) as u8
                };
                (
                    mix(a.0, b.0, c.0, d.0),
                    mix(a.1, b.1, c.1, d.1),
                    mix(a.2, b.2, c.2, d.2),
                )
            }
        }
    }
}

/// Map of a rectangular `cols` x `rows` panel read whole from the frame, pixels in reading
/// order, packed from `first_universe` on. A pixel never crosses a universe:
/// `512 / order.channels()` fit per universe (170 in RGB; the 2 leftover channels stay zero).
pub fn grid(cols: u32, rows: u32, first_universe: u16, order: Order) -> PixelMap {
    let per = 512 / order.channels();
    let mut fixtures = Vec::with_capacity(cols as usize * rows as usize);
    for y in 0..rows {
        for x in 0..cols {
            let i = fixtures.len();
            fixtures.push(Fixture {
                universe: first_universe + (i / per) as u16,
                channel: ((i % per) * order.channels()) as u16 + 1,
                order,
                u: (x as f32 + 0.5) / cols as f32,
                v: (y as f32 + 0.5) / rows as f32,
            });
        }
    }
    PixelMap {
        name: format!("grid {}x{}", cols, rows),
        source: String::new(),
        fixtures,
    }
}

struct Uni {
    number: u16,
    fixtures: Vec<Fixture>,
    data: [u8; 512],
}

/// Compiled map: fixtures grouped by universe and one 512-channel buffer per universe,
/// allocated once. `render` does not allocate.
///
/// The Mapper owns the universes it lists: a channel no fixture covers stays zero.
/// A universe that also receives `dmx` tracks is merged by the caller, not here.
pub struct Mapper {
    pub sampling: Sampling,
    unis: Vec<Uni>,
}

impl Mapper {
    /// Compiles the map. A fixture with universe 0 or a first channel outside 1..=512 is
    /// dropped here; one that goes past 512 on its last channel is truncated in `render`.
    pub fn new(map: &PixelMap) -> Mapper {
        let mut unis: Vec<Uni> = Vec::new();
        for f in &map.fixtures {
            if f.universe == 0 || f.channel == 0 || f.channel > 512 {
                continue;
            }
            match unis.binary_search_by_key(&f.universe, |u| u.number) {
                Ok(i) => unis[i].fixtures.push(*f),
                Err(i) => unis.insert(
                    i,
                    Uni {
                        number: f.universe,
                        fixtures: vec![*f],
                        data: [0u8; 512],
                    },
                ),
            }
        }
        Mapper {
            sampling: Sampling::default(),
            unis,
        }
    }

    pub fn universes(&self) -> usize {
        self.unis.len()
    }

    pub fn pixels(&self) -> usize {
        self.unis.iter().map(|u| u.fixtures.len()).sum()
    }

    /// Samples the frame and rewrites the buffers. Parallel per universe.
    // ponytail: CPU sampling with rayon ; switch to the PRD's wgpu compute shader when
    // `map_bench` no longer fits the 2 ms (more pixels, bilinear at 4K, several sources per
    // frame).
    pub fn render(&mut self, frame: &Frame) {
        let s = self.sampling;
        self.unis.par_iter_mut().for_each(|u| {
            for f in &u.fixtures {
                let (r, g, b) = frame.sample(f.u, f.v, s);
                f.order
                    .write(r, g, b, &mut u.data[f.channel as usize - 1..]);
            }
        });
    }

    /// `(universe, 512 bytes)` of each universe, in ascending order - exactly the pair that
    /// `protocols::Output::send` receives.
    pub fn frames(&self) -> impl Iterator<Item = (u16, &[u8; 512])> {
        self.unis.iter().map(|u| (u.number, &u.data))
    }

    /// Buffer of one universe, if the map covers it.
    pub fn universe(&self, number: u16) -> Option<&[u8; 512]> {
        self.unis
            .binary_search_by_key(&number, |u| u.number)
            .ok()
            .map(|i| &self.unis[i].data)
    }
}
