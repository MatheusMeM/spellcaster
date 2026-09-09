// Criterion: o caminho quente do pixel mapping — 100 000 pixels de um frame 1080p, nearest e
// bilinear, mais o custo de um universo sozinho (170 pixels) para separar rayon de amostragem.
// Saida ASCII pura (console cp1252).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixelmap::{grid, Frame, Mapper, Order};

fn frame_px(w: u32, h: u32) -> Vec<u8> {
    (0..w as usize * h as usize * 4)
        .map(|i| ((i * 37 + i / 991) & 0xff) as u8)
        .collect()
}

fn bench(c: &mut Criterion) {
    let (w, h) = (1920u32, 1080u32);
    let px = frame_px(w, h);
    let frame = Frame::rgba(w, h, &px).unwrap();

    let mut cem_mil = Mapper::new(&grid(317, 316, 1, Order::Rgb));
    c.bench_function("100k px nearest 1080p", |b| {
        b.iter(|| {
            cem_mil.render(black_box(&frame));
            black_box(cem_mil.universe(1).map(|d| d[0]))
        })
    });

    let mut bilin = Mapper::new(&grid(317, 316, 1, Order::Rgb));
    bilin.sampling = pixelmap::Sampling::Bilinear;
    c.bench_function("100k px bilinear 1080p", |b| {
        b.iter(|| {
            bilin.render(black_box(&frame));
            black_box(bilin.universe(1).map(|d| d[0]))
        })
    });

    let mut um = Mapper::new(&grid(170, 1, 1, Order::Rgb));
    c.bench_function("1 universo (170 px) nearest", |b| {
        b.iter(|| {
            um.render(black_box(&frame));
            black_box(um.universe(1).map(|d| d[0]))
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
