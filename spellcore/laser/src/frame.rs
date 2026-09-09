//! Ponto/frame ILDA, otimizacao de scan e safety.
//!
//! Porte 1:1 de `spellcaster/protocols/ilda/frame.py`. Os numeros tem que bater com o
//! Python ponto a ponto: as fixtures de `tests/fixtures/` sao geradas por ele.
//! Toda conversao float -> inteiro trunca em direcao a zero (o `int()` do Python) e so
//! depois satura, na mesma ordem.

use serde::{Deserialize, Serialize};

/// Limite de coordenada ILDA. -32768 nao existe: o Python satura em -LIM.
pub const LIM: i32 = 32767;

pub const DWELL: usize = 2;
pub const BLANK_GAP: usize = 4;
pub const MAX_STEP: i32 = 1200;
pub const ANGLE: f64 = 25.0;

#[inline]
pub fn clamp_xy(v: f64) -> i16 {
    (v as i32).clamp(-LIM, LIM) as i16
}

#[inline]
pub fn clamp_c(v: f64) -> u8 {
    (v as i32).clamp(0, 255) as u8
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub blank: bool,
}

impl Point {
    /// Mesmo construtor do Python: trunca e satura x/y em +-32767.
    #[inline]
    pub fn new(x: f64, y: f64, r: u8, g: u8, b: u8, blank: bool) -> Point {
        Point { x: clamp_xy(x), y: clamp_xy(y), r, g, b, blank }
    }

    /// Aceso = nao apagado e com alguma cor. Ponto preto nao conta para o bbox.
    #[inline]
    pub fn lit(&self) -> bool {
        !self.blank && (self.r | self.g | self.b) != 0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub points: Vec<Point>,
    /// Nome de frame do .ild (8 chars).
    pub name: String,
}

impl Frame {
    pub fn new(points: Vec<Point>, name: &str) -> Frame {
        Frame { points, name: name.to_string() }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    // clippy (len_without_is_empty) exige o par de `len`, que o `ild::write` usa.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

}

/// bbox dos pontos acesos.
// ponytail: so lit_only=True ; o lit_only=False do Python nao tem chamador, entra quando tiver
pub fn bbox(points: &[Point]) -> Option<(i16, i16, i16, i16)> {
    let mut it = points.iter().filter(|p| p.lit());
    let first = it.next()?;
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
    for p in it {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Some((x0, y0, x1, y1))
}

#[inline]
fn lerp(p: Point, q: Point, u: f64, blank: bool) -> Point {
    Point::new(
        p.x as f64 + (q.x as f64 - p.x as f64) * u,
        p.y as f64 + (q.y as f64 - p.y as f64) * u,
        q.r,
        q.g,
        q.b,
        blank,
    )
}

/// Dwell nos vertices, pontos apagados nas transicoes aceso<->apagado, interpolacao de
/// passos maiores que `max_step` (unidades ILDA). `angle` em graus.
pub fn optimize(frame: &Frame, dwell: usize, blank_gap: usize, max_step: i32, angle: f64) -> Frame {
    let mut out = Vec::with_capacity(frame.points.len() * 2);
    optimize_into(&frame.points, &mut out, dwell, blank_gap, max_step, angle);
    Frame { points: out, name: frame.name.clone() }
}

/// Versao sem alocacao para o caminho quente: reusa `out` (so cresce ate o pico do show).
pub fn optimize_into(
    src: &[Point],
    out: &mut Vec<Point>,
    dwell: usize,
    blank_gap: usize,
    max_step: i32,
    angle: f64,
) {
    out.clear();
    if src.is_empty() {
        return;
    }
    let cos_lim = angle.to_radians().cos();
    out.push(src[0]);
    for i in 1..src.len() {
        let p = src[i - 1];
        let q = src[i];
        // salto: repete p apagado antes de sair, q apagado antes de acender
        if p.lit() && !q.lit() {
            let b = Point { blank: true, ..p };
            for _ in 0..blank_gap {
                out.push(b);
            }
        }
        let step = (q.x as i32 - p.x as i32).abs().max((q.y as i32 - p.y as i32).abs());
        if step > max_step {
            let n = (step as f64 / max_step as f64).ceil() as i32;
            for k in 1..n {
                out.push(lerp(p, q, k as f64 / n as f64, !q.lit()));
            }
        }
        if !p.lit() && q.lit() {
            let b = Point { blank: true, ..q };
            for _ in 0..blank_gap {
                out.push(b);
            }
        }
        out.push(q);
        // vertice: mudanca de direcao -> dwell
        if q.lit() && i + 1 < src.len() && src[i + 1].lit() {
            let (ax, ay) = (q.x as f64 - p.x as f64, q.y as f64 - p.y as f64);
            let (bx, by) = (src[i + 1].x as f64 - q.x as f64, src[i + 1].y as f64 - q.y as f64);
            let (na, nb) = (ax.hypot(ay), bx.hypot(by));
            if na == 0.0 || nb == 0.0 || (ax * bx + ay * by) / (na * nb) < cos_lim {
                for _ in 0..dwell {
                    out.push(q);
                }
            }
        }
    }
}

/// Safety do engine. Obrigatoria: o `Feed` a aplica antes de TODO envio, nunca opcional.
///
/// Figura menor que `min_size` escurece proporcionalmente (ponto parado da size 0 e ganho 0);
/// `max_intensity` limita o brilho; ponto fora de `zone` apaga.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Safety {
    #[serde(default = "def_min_size")]
    pub min_size: i32,
    #[serde(default = "def_max_intensity")]
    pub max_intensity: u8,
    /// (x0, y0, x1, y1) em UNIDADES ILDA (+-32767), como no Python.
    // ponytail: zone em unidades ILDA, nao normalizada ; o exemplo [-1,-0.2,1,1] do PRD e
    // normalizado e quem carrega o track multiplica por 32767 antes de chegar aqui.
    #[serde(default)]
    pub zone: Option<(f64, f64, f64, f64)>,
}

fn def_min_size() -> i32 {
    2000
}

fn def_max_intensity() -> u8 {
    255
}

impl Default for Safety {
    fn default() -> Safety {
        Safety { min_size: def_min_size(), max_intensity: def_max_intensity(), zone: None }
    }
}

impl Safety {
    /// Aplica no lugar, sem alocar. Usa o bbox ORIGINAL, medido antes de mexer nas cores.
    pub fn apply(&self, points: &mut [Point]) {
        let gain = match bbox(points) {
            Some((x0, y0, x1, y1)) => {
                let size = (x1 as i32 - x0 as i32).max(y1 as i32 - y0 as i32);
                if size < self.min_size {
                    size as f64 / self.min_size as f64
                } else {
                    1.0
                }
            }
            None => 1.0,
        };
        let mx = self.max_intensity as f64;
        for p in points.iter_mut() {
            p.r = clamp_c((p.r as f64 * gain).min(mx));
            p.g = clamp_c((p.g as f64 * gain).min(mx));
            p.b = clamp_c((p.b as f64 * gain).min(mx));
            if let Some(z) = self.zone {
                let (x, y) = (p.x as f64, p.y as f64);
                if !(z.0 <= x && x <= z.2 && z.1 <= y && y <= z.3) {
                    p.blank = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(cx: i32, cy: i32, w: i32, h: i32, col: (u8, u8, u8), k: i32) -> Vec<Point> {
        let v = [
            (cx - w, cy - h),
            (cx + w, cy - h),
            (cx + w, cy + h),
            (cx - w, cy + h),
            (cx - w, cy - h),
        ];
        let mut out = Vec::new();
        for s in v.windows(2) {
            let ((x0, y0), (x1, y1)) = (s[0], s[1]);
            for j in 0..=k {
                out.push(Point::new(
                    x0 as f64 + (x1 - x0) as f64 * j as f64 / k as f64,
                    y0 as f64 + (y1 - y0) as f64 * j as f64 / k as f64,
                    col.0,
                    col.1,
                    col.2,
                    false,
                ));
            }
        }
        out
    }

    fn square(size: i32) -> Frame {
        Frame::new(rect(0, 0, size, size, (255, 0, 0), 4), "quad")
    }

    #[test]
    fn ponto_satura_e_trunca() {
        let p = Point::new(1e9, -1e9, 1, 2, 3, false);
        assert_eq!((p.x, p.y), (32767, -32767));
        assert_eq!(Point::new(-0.9, 0.9, 0, 0, 0, false).x, 0);
        assert!(!Point::new(0.0, 0.0, 0, 0, 0, false).lit());
        assert!(!Point::new(0.0, 0.0, 255, 0, 0, true).lit());
    }

    // mesmo caso do tests/test_ilda.py::TestOptimize
    #[test]
    fn optimize_apaga_dwella_e_interpola() {
        let f = Frame::new(
            vec![
                Point::new(0.0, 0.0, 255, 0, 0, false),
                Point::new(3000.0, 0.0, 255, 0, 0, false),
                Point::new(3000.0, 3000.0, 255, 0, 0, false),
                Point::new(-20000.0, -20000.0, 0, 0, 0, true),
                Point::new(-20000.0, -20000.0, 255, 0, 0, false),
                Point::new(-19000.0, -20000.0, 255, 0, 0, false),
            ],
            "",
        );
        let o = optimize(&f, 2, 4, 1200, ANGLE);
        let p = &o.points;
        assert_eq!(p.iter().filter(|q| q.blank && (q.x, q.y) == (3000, 3000)).count(), 4);
        assert!(p.iter().filter(|q| q.blank && (q.x, q.y) == (-20000, -20000)).count() >= 5);
        assert!(p.windows(2).all(|w| (w[1].x as i32 - w[0].x as i32)
            .abs()
            .max((w[1].y as i32 - w[0].y as i32).abs())
            <= 1200));
        assert_eq!(p.iter().filter(|q| q.lit() && (q.x, q.y) == (3000, 0)).count(), 3);
        assert!(optimize(&Frame::default(), 2, 4, 1200, ANGLE).points.is_empty());
    }

    #[test]
    fn safety_escurece_figura_pequena() {
        // `Safety::apply` e' o caminho de producao (o feed monta a struct e chama).
        let sf = |f: &Frame, min_size, max_intensity, zone| {
            let mut out = f.clone();
            Safety { min_size, max_intensity, zone }.apply(&mut out.points);
            out
        };
        let small = sf(&square(500), 2000, 255, None);
        assert!(small.points.iter().all(|p| p.r == 127)); // 1000/2000 * 255, truncado
        let big = sf(&square(5000), 2000, 255, None);
        assert!(big.points.iter().all(|p| p.r == 255));
        let dot = sf(
            &Frame::new(vec![Point::new(0.0, 0.0, 255, 255, 255, false); 5], ""),
            2000,
            255,
            None,
        );
        assert!(dot.points.iter().all(|p| (p.r, p.g, p.b) == (0, 0, 0)));
    }

    #[test]
    fn safety_limita_intensidade_e_zona() {
        let mut f = square(5000);
        Safety {
            min_size: 2000,
            max_intensity: 100,
            zone: Some((-1000.0, -1000.0, 1000.0, 1000.0)),
        }
        .apply(&mut f.points);
        assert!(f.points.iter().all(|p| p.r == 100));
        assert!(f.points.iter().filter(|p| p.x.abs() == 5000).all(|p| p.blank));
    }
}
