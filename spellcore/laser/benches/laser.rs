// Criterion: optimize + safety de um frame de 1000 pontos, que e o caminho quente do
// track de laser (o Feed roda os dois por frame, a 25-50 fps por DAC).
// Saida ASCII pura (console cp1252).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use laser::dac::etherdream::encode_data;
use laser::feed::Transform;
use laser::frame::{optimize_into, Frame, Point, Safety, ANGLE, BLANK_GAP, DWELL, MAX_STEP};

/// Figura de 1000 pontos com vertices e um salto apagado: exercita dwell, blank_gap e
/// interpolacao de passo no mesmo frame.
fn figura(n: usize) -> Frame {
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let u = i as f64 / n as f64;
        let a = u * std::f64::consts::TAU * 3.0;
        let r = 8000.0 + 6000.0 * (a * 2.0).sin();
        // um salto no meio: pula para o outro canto apagado
        let jump = (n / 2..n / 2 + 3).contains(&i);
        pts.push(Point::new(
            r * a.cos() + if jump { 25000.0 } else { 0.0 },
            r * a.sin(),
            255,
            (200.0 * u) as u8,
            0,
            jump,
        ));
    }
    Frame::new(pts, "bench")
}

fn bench(c: &mut Criterion) {
    let f = figura(1000);
    let mut out: Vec<Point> = Vec::with_capacity(8192);
    let safety = Safety {
        min_size: 2000,
        max_intensity: 200,
        zone: Some((-20000.0, -20000.0, 20000.0, 20000.0)),
    };

    c.bench_function("optimize 1000 pontos", |b| {
        b.iter(|| {
            optimize_into(
                black_box(&f.points),
                &mut out,
                DWELL,
                BLANK_GAP,
                MAX_STEP,
                ANGLE,
            );
            black_box(out.len())
        })
    });

    optimize_into(&f.points, &mut out, DWELL, BLANK_GAP, MAX_STEP, ANGLE);
    let otimizado = out.clone();
    let mut work = otimizado.clone();
    c.bench_function("safety do frame otimizado", |b| {
        b.iter(|| {
            work.copy_from_slice(&otimizado);
            safety.apply(black_box(&mut work));
            black_box(work.len())
        })
    });

    c.bench_function("optimize+safety 1000 pontos", |b| {
        b.iter(|| {
            optimize_into(
                black_box(&f.points),
                &mut out,
                DWELL,
                BLANK_GAP,
                MAX_STEP,
                ANGLE,
            );
            safety.apply(&mut out);
            black_box(out.len())
        })
    });

    // exatamente o que a thread do Feed faz por frame, sem o I/O: e o piso de CPU do
    // aceite dos 4 feeds a 30 kpps (600 pontos por frame, 50 frames/s por DAC).
    let f600 = figura(600);
    let tf = Transform {
        x: 100.0,
        y: -50.0,
        scale: 0.9,
        rot: 33.0,
        color: (1.0, 0.8, 0.5),
    };
    let mut work: Vec<Point> = Vec::with_capacity(1024);
    let mut wire: Vec<u8> = Vec::with_capacity(16384);
    c.bench_function("feed: transform+safety+encode 600 pontos", |b| {
        b.iter(|| {
            tf.apply(black_box(&f600.points), &mut work);
            safety.apply(&mut work);
            encode_data(&work, &mut wire);
            black_box(wire.len())
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
