//! `pixelmap` — R3 do Spellcaster: amostra um frame RGB/RGBA e escreve os bytes DMX de cada
//! pixel fisico, um buffer de 512 canais por universo, pronto para `protocols::Output::send`.
//!
//! Crate autocontido: nao depende de `engine` nem de `protocols`. A fonte do frame e um
//! `&[u8]` — quando a R2 (midia) existir, o texture pool entrega o mesmo slice.
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
//! No `.spell` o bloco vive fora de `tracks` e sobrevive ao round-trip do `engine::Show`
//! (campo `extra`, `#[serde(flatten)]`). O chamador desserializa com serde_json:
//!
//! ```json
//! {"pixelmaps": [{"name": "wall", "source": "media/1",
//!                 "fixtures": [{"universe": 10, "channel": 1, "order": "grb",
//!                               "u": 0.0, "v": 0.0}]}]}
//! ```

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Ordem dos canais de cor de um pixel fisico.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    #[default]
    Rgb,
    Grb,
    Rgbw,
}

impl Order {
    /// Quantos canais DMX o pixel ocupa.
    pub const fn channels(self) -> usize {
        match self {
            Order::Rgbw => 4,
            _ => 3,
        }
    }

    /// Escreve a cor em `out` (ate 4 bytes), truncando no que couber.
    ///
    /// RGBW extrai o branco por `w = min(r, g, b)` e desconta dos tres — e a conversao que
    /// nao estoura o fluxo total da luminaria.
    // ponytail: extracao de branco fixa ; virar campo do Fixture quando aparecer luminaria
    // que espere W independente (branco quente separado, CCT).
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

/// Um pixel fisico: onde ele le no frame (`u`, `v` normalizados) e onde escreve em DMX.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Fixture {
    /// Universo, 1-based (sACN).
    pub universe: u16,
    /// Primeiro canal DMX, 1-based.
    pub channel: u16,
    #[serde(default)]
    pub order: Order,
    /// Coordenada normalizada no frame; fora de 0..1 e clampada na borda.
    pub u: f32,
    pub v: f32,
}

/// Um mapa do `.spell`: uma fonte de video e os pixels fisicos que a leem.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PixelMap {
    #[serde(default)]
    pub name: String,
    /// Fonte do frame, `"media/<id>"`. Resolvida pela R2; o crate nao a interpreta.
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub fixtures: Vec<Fixture>,
}

/// Como o pixel le o frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Sampling {
    /// Pixel mais proximo. Default: um LED e um pixel, nao ha o que interpolar.
    #[default]
    Nearest,
    /// Bilinear entre os 4 vizinhos. Para mapa mais denso que a fonte.
    Bilinear,
}

/// Frame de entrada emprestado: `stride` = 3 (RGB) ou 4 (RGBA), linhas contiguas.
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

    /// Fronteira de confianca: o frame vem do decoder e e validado UMA vez aqui; o caminho
    /// quente le sem checar de novo.
    pub fn new(w: u32, h: u32, stride: usize, px: &'a [u8]) -> Result<Frame<'a>, String> {
        if w == 0 || h == 0 {
            return Err(format!("frame {}x{}: dimensao zero", w, h));
        }
        if stride != 3 && stride != 4 {
            return Err(format!("stride {}: so RGB (3) ou RGBA (4)", stride));
        }
        let need = w as usize * h as usize * stride;
        if px.len() < need {
            return Err(format!(
                "frame {}x{} stride {}: {} bytes, precisa de {}",
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

    /// Amostra em coordenada normalizada. `u`/`v` fora de 0..1 sao clampados na borda.
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
                // centro do texel em u = (x + 0.5) / w
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

/// Mapa de um painel retangular `cols` x `rows` lido inteiro do frame, pixels em ordem de
/// leitura, empacotados a partir de `first_universe`. Um pixel nunca cruza universo: cabem
/// `512 / order.channels()` por universo (170 em RGB; os 2 canais que sobram ficam em zero).
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

/// Mapa compilado: fixtures agrupadas por universo e um buffer de 512 canais por universo,
/// alocados uma vez. `render` nao aloca.
///
/// O Mapper e dono dos universos que lista: canal que nenhuma fixture cobre fica em zero.
/// Universo que tambem recebe tracks `dmx` e somado pelo chamador, nao aqui.
pub struct Mapper {
    pub sampling: Sampling,
    unis: Vec<Uni>,
}

impl Mapper {
    /// Compila o mapa. Fixture com universo 0 ou primeiro canal fora de 1..=512 e descartada
    /// aqui; a que passa dos 512 no ultimo canal e truncada no `render`.
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

    /// Amostra o frame e reescreve os buffers. Paralelo por universo.
    // ponytail: amostragem em CPU com rayon ; trocar pelo compute shader wgpu do PRD quando
    // `map_bench` nao fechar os 2 ms (mais pixels, bilinear em 4K, varias fontes por frame).
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

    /// `(universo, 512 bytes)` de cada universo, em ordem crescente — exatamente o par que
    /// `protocols::Output::send` recebe.
    pub fn frames(&self) -> impl Iterator<Item = (u16, &[u8; 512])> {
        self.unis.iter().map(|u| (u.number, &u.data))
    }

    /// Buffer de um universo, se o mapa o cobre.
    pub fn universe(&self, number: u16) -> Option<&[u8; 512]> {
        self.unis
            .binary_search_by_key(&number, |u| u.number)
            .ok()
            .map(|i| &self.unis[i].data)
    }
}
