//! FOSFORO: RGBA bitmap -> contours -> ILDA points.
//!
//! The half of NDI->ILDA that does not depend on an SDK (`design/FUNCOES/ndi-ilda.md`,
//! algorithm "Contornos"): luma mask, Moore boundary following, Ramer-Douglas-Peucker
//! simplification, path ordering by nearest neighbor, blanking between paths and the SAME
//! `optimize`/`safety` as the rest of the crate. Pure Rust, no new dependency; the input is
//! a raw RGBA buffer, wherever it comes from.
//!
//! Deterministic: same input, same output bytes (a distance tie always keeps the smallest
//! index, and there is no iteration over a hash).
//!
//! ponytail: no NDI ; it arrives when the NDI SDK is installed (ROADMAP R2) - all that is
//! missing is someone to fill in the `rgba`.
//! ponytail: no Spout and no screen capture ; same entry point, same reason.
//! ponytail: no temporal smoothing between frames ; Damping/Lag/OneEuro are graph nodes in
//! `script` (design/FUNCOES/ndi-ilda.md section 1), not a hidden parameter here.
//! ponytail: only the "Contornos" algorithm ; skeleton (Zhang-Suen) and raster arrive when
//! the source material is a stroke or solid text.
//! ponytail: one path per object, outer contour only ; an inner hole becomes a path when
//! someone asks for a ring - today it would cost a second edge pass.
//! ponytail: luma mask only ; the alpha mask (transparent background with black RGB) arrives
//! together with NDI, which is what produces such a frame - today there is no producer.

use crate::frame::{optimize_into, Point, Safety, ANGLE, BLANK_GAP, DWELL, LIM, MAX_STEP};

/// Ceiling of paths per frame, from the design (MadMapper: "the engine gives up above
/// 2 000"). It also caps the O(n^2) cost of the nearest-neighbor ordering.
pub const MAX_PATHS: usize = 2000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opts {
    /// Mask cutoff: a pixel is in when `luma > threshold`.
    pub threshold: u8,
    /// Ramer-Douglas-Peucker tolerance, IN PIXELS of the input image.
    pub epsilon: f32,
    /// Ceiling of points BEFORE `optimize` (which still adds dwell and blanking).
    pub max_points: usize,
    /// Inverts the mask (dark figure on a light background).
    pub invert: bool,
    /// Single color for every point; `None` samples the source pixel under each point.
    pub color: Option<(u8, u8, u8)>,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            threshold: 128,
            epsilon: 1.5,
            max_points: 4000,
            invert: false,
            color: Some((255, 255, 255)),
        }
    }
}

/// Moore neighborhood clockwise (the image has Y going down).
const D: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

struct Grid<'a> {
    m: &'a [bool],
    w: i32,
    h: i32,
}

impl Grid<'_> {
    #[inline]
    fn at(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h && self.m[(y * self.w + x) as usize]
    }
}

/// Mask by integer BT.601 luma. One pass only: 2 Mpx in a 1080p frame, and the mask is half
/// the cost of the frame.
fn mask(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<bool> {
    rgba[..w * h * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let l = ((77 * p[0] as u32 + 150 * p[1] as u32 + 29 * p[2] as u32) >> 8) as u8;
            (l > o.threshold) != o.invert
        })
        .collect()
}

/// Next boundary pixel clockwise from the arrival direction `bdir`.
/// Returns the pixel and the new arrival direction.
#[inline]
fn step(g: &Grid, p: (i32, i32), bdir: usize) -> Option<((i32, i32), usize)> {
    for k in 1..=8 {
        let d = (bdir + k) % 8;
        let q = (p.0 + D[d].0, p.1 + D[d].1);
        if g.at(q.0, q.1) {
            return Some((q, (d + 4) % 8));
        }
    }
    None
}

/// Moore outer contour from `start` (whose left neighbor is off).
/// Stops by Jacob's criterion: same pixel arriving from the same direction. Comes out closed.
fn contour(g: &Grid, start: (i32, i32)) -> Vec<(i32, i32)> {
    let mut c = vec![start];
    let (mut p, mut bdir) = match step(g, start, 4) {
        Some(s) => s,
        None => return c, // isolated pixel
    };
    let first = (p, bdir);
    let cap = (g.w as usize * g.h as usize) * 2 + 8;
    loop {
        c.push(p);
        let (q, nb) = match step(g, p, bdir) {
            Some(s) => s,
            None => break,
        };
        p = q;
        bdir = nb;
        if (p, bdir) == first || c.len() >= cap {
            break;
        }
    }
    if c.last() != c.first() {
        c.push(c[0]);
    }
    c
}

/// Iterative Ramer-Douglas-Peucker (explicit stack: a 1080p contour goes past 10 000 points
/// and the recursive version blows the stack). A distance tie keeps the smallest index.
fn rdp(pts: &[(i32, i32)], eps: f32) -> Vec<(i32, i32)> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let e2 = eps as f64 * eps as f64;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let (ax, ay) = (pts[a].0 as f64, pts[a].1 as f64);
        let (dx, dy) = (pts[b].0 as f64 - ax, pts[b].1 as f64 - ay);
        let len2 = dx * dx + dy * dy;
        let (mut best, mut bd) = (a, -1.0f64);
        for (i, q) in pts.iter().enumerate().take(b).skip(a + 1) {
            let (qx, qy) = (q.0 as f64 - ax, q.1 as f64 - ay);
            // closed path: a == b, so the "line" degenerates and the distance is to the point
            let d = if len2 == 0.0 {
                qx * qx + qy * qy
            } else {
                let cr = qx * dy - qy * dx;
                cr * cr / len2
            };
            if d > bd {
                bd = d;
                best = i;
            }
        }
        if bd > e2 {
            keep[best] = true;
            stack.push((a, best));
            stack.push((best, b));
        }
    }
    pts.iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(p, _)| *p)
        .collect()
}

/// Proportional cut: each path keeps `len * max / total` points, by uniform sampling that
/// preserves both ends (and therefore the closing of the contour).
// ponytail: the minimum of 2 points per path can exceed `max` when there are more than
// max/2 paths ; MAX_PATHS caps the worst case, and whoever wants exactness filters by
// length first (the design's `Limit: keep the N longest`).
fn decimate(paths: &mut [Vec<(i32, i32)>], max: usize) {
    let total: usize = paths.iter().map(|p| p.len()).sum();
    if max == 0 || total <= max {
        return;
    }
    for p in paths.iter_mut() {
        let keep = ((p.len() as u64 * max as u64) / total as u64).max(2) as usize;
        if keep >= p.len() {
            continue;
        }
        let last = p.len() - 1;
        let src = std::mem::take(p);
        *p = (0..keep).map(|i| src[i * last / (keep - 1)]).collect();
    }
}

/// Orders the paths by nearest neighbor starting from the top-left corner, to shorten the
/// blanked jumps.
// ponytail: greedy O(n^2) without reversing a path ; MAX_PATHS=2000 is the cost ceiling
fn order(paths: &mut [Vec<(i32, i32)>]) {
    let mut cur = (0i64, 0i64);
    for i in 0..paths.len() {
        // `rotate_right` keeps the original order among the unchosen ones, so a tie still
        // goes to the smallest input index
        let best = (i..paths.len())
            .min_by_key(|&j| {
                let s = paths[j][0];
                (s.0 as i64 - cur.0).pow(2) + (s.1 as i64 - cur.1).pow(2)
            })
            .expect("the range starts at i and is never empty");
        paths[i..=best].rotate_right(1);
        let e = paths[i][paths[i].len() - 1];
        cur = (e.0 as i64, e.1 as i64);
    }
}

/// Vectorized paths in ILDA coordinates (-32767..32767, Y up), BEFORE blanking, `optimize`
/// and `safety`. One path per object, closed.
///
/// The image goes in whole and centered, with the aspect ratio preserved (the longer side
/// takes the whole range). A buffer smaller than `w * h * 4` returns empty, and a `w * h * 4`
/// that overflows `usize` too: the dimension comes from outside (command line, frame
/// producer), not from the buffer itself.
pub fn paths(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Vec<Point>> {
    let Some(n) = w.checked_mul(h).and_then(|n| n.checked_mul(4)) else {
        return Vec::new();
    };
    if w == 0 || h == 0 || rgba.len() < n {
        return Vec::new();
    }
    let m = mask(rgba, w, h, o);
    let g = Grid {
        m: &m,
        w: w as i32,
        h: h as i32,
    };
    let mut seen = vec![false; w * h];
    let mut raw: Vec<Vec<(i32, i32)>> = Vec::new();
    'fora: for y in 0..g.h {
        for x in 0..g.w {
            let i = (y * g.w + x) as usize;
            // candidate = lit pixel, not traced yet, with the left neighbor off
            if !m[i] || seen[i] || (x > 0 && m[i - 1]) {
                continue;
            }
            let p = (x, y);
            let c = contour(&g, p);
            for &(cx, cy) in &c {
                seen[(cy * g.w + cx) as usize] = true;
            }
            // The OUTER contour of a component is the only one whose top-left pixel is the
            // candidate itself: the raster reaches it first. Starting from the edge of a
            // hole, the cycle has a minimum smaller than `p` - it is a re-trace of the same
            // component and is dropped. It replaces the flood fill of the whole component,
            // which cost ~40 ns per lit pixel (80 ms in a fully lit 1080p frame).
            if c.iter().min_by_key(|&&(px, py)| (py, px)) == Some(&p) {
                raw.push(c);
                if raw.len() >= MAX_PATHS {
                    break 'fora;
                }
            }
        }
    }
    for c in raw.iter_mut() {
        *c = rdp(c, o.epsilon);
    }
    decimate(&mut raw, o.max_points);
    order(&mut raw);

    let s = 2.0 * LIM as f64 / (w.max(h) - 1).max(1) as f64;
    let (cx, cy) = ((w - 1) as f64 / 2.0, (h - 1) as f64 / 2.0);
    raw.iter()
        .map(|c| {
            c.iter()
                .map(|&(x, y)| {
                    let (r, gc, b) = o.color.unwrap_or_else(|| {
                        let i = (y as usize * w + x as usize) * 4;
                        (rgba[i], rgba[i + 1], rgba[i + 2])
                    });
                    Point::new((x as f64 - cx) * s, (cy - y as f64) * s, r, gc, b, false)
                })
                .collect()
        })
        .collect()
}

/// An RGBA frame becomes an ILDA frame ready for the `Feed`: paths, a blanked jump between
/// them, `optimize` and the default `safety`.
// ponytail: default safety in here ; a custom zone and limit apply afterwards, with
// `frame::safety` - the `Feed` reapplies its own on every send anyway.
pub fn trace(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Point> {
    let ps = paths(rgba, w, h, o);
    if ps.is_empty() {
        return Vec::new();
    }
    let mut src = Vec::with_capacity(ps.iter().map(|p| p.len() + 1).sum());
    for (i, p) in ps.iter().enumerate() {
        if i > 0 {
            // blanked point at the start of the next path: `optimize` opens the gap and
            // interpolates the jump on its own
            src.push(Point {
                blank: true,
                ..p[0]
            });
        }
        src.extend_from_slice(p);
    }
    let mut out = Vec::with_capacity(src.len() * 2);
    optimize_into(&src, &mut out, DWELL, BLANK_GAP, MAX_STEP, ANGLE);
    Safety::default().apply(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdp_reduces_line_and_keeps_corner() {
        let reta: Vec<(i32, i32)> = (0..20).map(|i| (i, 0)).collect();
        assert_eq!(rdp(&reta, 1.0), vec![(0, 0), (19, 0)]);
        let l = [(0, 0), (5, 0), (10, 0), (10, 5), (10, 10)];
        assert_eq!(rdp(&l, 1.0), vec![(0, 0), (10, 0), (10, 10)]);
    }

    #[test]
    fn decimate_cuts_proportionally() {
        let mut p = vec![(0..100).map(|i| (i, 0)).collect::<Vec<_>>()];
        decimate(&mut p, 10);
        assert_eq!(p[0].len(), 10);
        assert_eq!(p[0][0], (0, 0));
        assert_eq!(p[0][9], (99, 0));
    }
}
