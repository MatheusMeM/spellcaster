// bench `map_bench`: pixel mapping de 100 000 pixels de um frame 1080p a 60 Hz.
// Gate do PRD (secao 3): < 2 ms por frame. Sai com erro se o p99 estourar.
// Saida ASCII pura (console cp1252).

use pixelmap::{grid, Frame, Mapper, Order};
use std::time::Instant;

fn arg(name: &str, default: f64) -> f64 {
    let a: Vec<String> = std::env::args().collect();
    for i in 0..a.len() {
        if a[i] == name {
            if let Some(v) = a.get(i + 1) {
                if let Ok(x) = v.parse() {
                    return x;
                }
            }
        }
    }
    default
}

fn flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

fn main() {
    let pixels = arg("--pixels", 100_000.0).max(1.0) as u32;
    let frames = arg("--frames", 600.0).max(1.0) as usize;
    let (w, h) = (arg("--width", 1920.0) as u32, arg("--height", 1080.0) as u32);
    let bilinear = flag("--bilinear");

    let cols = (pixels as f64).sqrt().ceil() as u32;
    let rows = pixels.div_ceil(cols);
    let mut m = Mapper::new(&grid(cols, rows, 1, Order::Rgb)).bilinear(bilinear);

    // Frame 1080p RGBA sintetico: gradiente + ruido, para nenhuma amostra cair sempre no
    // mesmo valor e o cache nao ficar irrealmente quente.
    let mut px = vec![0u8; w as usize * h as usize * 4];
    for (i, b) in px.iter_mut().enumerate() {
        *b = ((i * 37 + i / 991) & 0xff) as u8;
    }
    let frame = Frame::rgba(w, h, &px).unwrap();

    for _ in 0..30 {
        m.render(&frame); // aquece o pool do rayon e o cache
    }

    let mut ms: Vec<f64> = Vec::with_capacity(frames);
    for _ in 0..frames {
        let t0 = Instant::now();
        m.render(&frame);
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(m.universe(1).map(|d| d[0]));
    }
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |q: f64| ms[((ms.len() - 1) as f64 * q) as usize];
    let media: f64 = ms.iter().sum::<f64>() / ms.len() as f64;
    let p99 = p(0.99);

    println!(
        "map_bench {} px ({}x{}) em {} universos, frame {}x{} RGBA, {}, {} frames",
        m.pixels(),
        cols,
        rows,
        m.universes(),
        w,
        h,
        if bilinear { "bilinear" } else { "nearest" },
        frames
    );
    println!(
        "por frame: media={:.3}ms p50={:.3}ms p99={:.3}ms max={:.3}ms",
        media,
        p(0.5),
        p99,
        ms[ms.len() - 1]
    );
    let ok = p99 < 2.0;
    println!(
        "alvo: p99 < 2.000ms por frame -> {}",
        if ok { "OK" } else { "FALHA" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
