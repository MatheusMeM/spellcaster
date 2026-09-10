// Criterion: hot path of the engine (curves, keyframe eval, Timeline::apply) and the packet
// assembly of both protocols. harness = false in Cargo.toml.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::timeline::{Curve, Keyframe, Keys, Timeline, Value, BEZ};
use engine::universe::Universes;
use std::path::Path;

const SHOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo_r0.spell");

fn curve(c: &mut Criterion) {
    let us = [0.0, 0.13, 0.27, 0.41, 0.5, 0.66, 0.78, 0.91, 1.0];
    let mut g = c.benchmark_group("curve");
    for (name, k) in [
        ("linear", Curve::Linear),
        ("hold", Curve::Hold),
        ("in", Curve::In),
        ("out", Curve::Out),
        ("inout", Curve::InOut),
        ("bezier", Curve::Bezier),
    ] {
        g.bench_function(name, |b| {
            b.iter(|| {
                let mut s = 0.0;
                for u in us {
                    s += k.ease(black_box(u), BEZ);
                }
                s
            })
        });
    }
    g.finish();
}

fn keys_eval(c: &mut Criterion) {
    // 3000 keyframes, alternating curves: the same shape as the medgrupo tracks.
    let curves = [
        Curve::Linear,
        Curve::In,
        Curve::Out,
        Curve::InOut,
        Curve::Bezier,
    ];
    let keys = Keys::new(
        (0..3000)
            .map(|i| Keyframe {
                t: i as f64 * 0.01,
                value: Value::Num((i % 256) as f64),
                curve: curves[i % curves.len()],
                c: BEZ,
            })
            .collect(),
    );
    // scattered points: forces the binary search to land in different ranges.
    let ts: Vec<f64> = (0..64).map(|i| i as f64 * 29.999 / 64.0).collect();
    let mut out: Vec<f64> = Vec::with_capacity(8);

    c.bench_function("keys_eval", |b| {
        b.iter(|| {
            for t in &ts {
                keys.eval(black_box(*t), &mut out);
            }
        })
    });
}

fn timeline_apply(c: &mut Criterion) {
    // ponytail: if the .spell is not generated the bench disappears instead of breaking the
    // suite ; run tests/conformance/gen.py to get it back.
    let show = match engine::show::load(Path::new(SHOW)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("timeline_apply skipped: {}", e);
            return;
        }
    };
    let mut tl = match Timeline::new(&show) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("timeline_apply skipped: {}", e);
            return;
        }
    };
    let mut u = Universes::new();
    let dur = show.duration.unwrap_or(60.0);
    let mut n = 0u32;

    c.bench_function("timeline_apply", |b| {
        b.iter(|| {
            n = n.wrapping_add(1);
            let t = (n as f64 / 60.0) % dur;
            tl.apply(&mut u, black_box(t));
        })
    });
}

fn packets(c: &mut Criterion) {
    let mut data = [0u8; 512];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i & 0xff) as u8;
    }
    let cid: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    c.bench_function("sacn_packet", |b| {
        b.iter(|| protocols::sacn::packet(1, black_box(&data), &cid, 0, "Spellcaster", 100))
    });
    c.bench_function("artdmx", |b| {
        b.iter(|| protocols::artnet::artdmx(1, black_box(&data), 0))
    });
}

criterion_group!(benches, curve, keys_eval, timeline_apply, packets);
criterion_main!(benches);
