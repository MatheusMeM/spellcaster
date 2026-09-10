// Criterion: optimize + safety of a 1000-point frame, which is the hot path of the laser
// track (the Feed runs both per frame, at 25-50 fps per DAC).
// Pure ASCII output (cp1252 console).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use laser::dac::etherdream::encode_data;
use laser::feed::Transform;
use laser::frame::{optimize_into, Frame, Point, Safety, ANGLE, BLANK_GAP, DWELL, MAX_STEP};

/// Figure of 1000 points with vertices and a blanked jump: exercises dwell, blank_gap and
/// step interpolation in the same frame.
fn figura(n: usize) -> Frame {
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let u = i as f64 / n as f64;
        let a = u * std::f64::consts::TAU * 3.0;
        let r = 8000.0 + 6000.0 * (a * 2.0).sin();
        // one jump in the middle: hops blanked to the other corner
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

    c.bench_function("optimize 1000 points", |b| {
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
    c.bench_function("safety of the optimized frame", |b| {
        b.iter(|| {
            work.copy_from_slice(&otimizado);
            safety.apply(black_box(&mut work));
            black_box(work.len())
        })
    });

    c.bench_function("optimize+safety 1000 points", |b| {
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

    // exactly what the Feed thread does per frame, without the I/O: it is the CPU floor of
    // the 4-feed acceptance at 30 kpps (600 points per frame, 50 frames/s per DAC).
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
    c.bench_function("feed: transform+safety+encode 600 points", |b| {
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
