// bench `jitter`: mede o desvio entre o instante real de cada tick do Clock e o alvo
// teorico t0 + n/fps. Gate do PRD: p99 < 1 ms, drift 0 frames.
// Saida ASCII pura (console cp1252).

use engine::clock::Clock;
use std::time::Instant;

fn arg_f64(name: &str, default: f64) -> f64 {
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

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = (((sorted.len() - 1) as f64) * p).round() as usize;
    sorted[i]
}

fn main() {
    let fps = arg_f64("--fps", 60.0).max(1.0);
    let secs = arg_f64("--secs", 10.0).max(0.1);
    let dt = 1.0 / fps;

    // ponytail: capacidade reservada uma vez, o push no laco nao realoca ;
    // trocar por buffer em anel se algum dia rodar sem duracao conhecida.
    let mut dev: Vec<f64> = Vec::with_capacity((fps * secs) as usize + 64);
    let mut t0: Option<Instant> = None;
    let mut last = Instant::now();

    let clock = Clock::new(fps as u32);
    clock.run(
        |_t| {
            let now = Instant::now();
            let base = *t0.get_or_insert(now);
            let n = dev.len() as f64;
            dev.push(now.duration_since(base).as_secs_f64() - n * dt);
            last = now;
        },
        Some(secs),
    );

    let frames = dev.len();
    if frames < 2 {
        println!("jitter: sem amostras (frames={}) -> FALHA", frames);
        std::process::exit(1);
    }

    let span = last.duration_since(t0.unwrap()).as_secs_f64();
    // drift = intervalos de frame que o relogio de parede andou, menos os que o Clock entregou.
    let drift = (span * fps).round() as i64 - (frames as i64 - 1);

    let mut abs: Vec<f64> = dev.iter().map(|d| d.abs()).collect();
    abs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = pct(&abs, 0.50) * 1e3;
    let p99 = pct(&abs, 0.99) * 1e3;
    let max = abs[abs.len() - 1] * 1e3;

    println!(
        "jitter {}Hz {}s: frames={} p50={:.3}ms p99={:.3}ms max={:.3}ms drift={} frames",
        fps as u32, secs, frames, p50, p99, max, drift
    );

    let s = clock.stats();
    println!(
        "clock.stats(): p50={:.3}ms p99={:.3}ms max={:.3}ms frames={} drift={}",
        s.p50 * 1e3,
        s.p99 * 1e3,
        s.max * 1e3,
        s.frames,
        s.drift
    );

    let ok = p99 < 1.0 && drift == 0;
    println!(
        "alvo: p99 < 1.000ms, drift = 0  -> {}",
        if ok { "OK" } else { "FALHA" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
