// bench `jitter`: PRD gate on the Clock -- p99 < 1 ms and drift 0 frames. The numbers come from
// Clock::stats() (the Clock itself already measures the deviation of each tick); only the gate
// lives here. Pure ASCII output (cp1252 console).

use bench::arg_f64;
use engine::clock::Clock;

fn main() {
    let fps = arg_f64("--fps", 60.0).max(1.0);
    let secs = arg_f64("--secs", 10.0).max(0.1);

    let clock = Clock::new(fps as u32);
    clock.run(|_t| {}, Some(secs));
    let s = clock.stats();

    println!(
        "jitter {}Hz {}s: frames={} p50={:.3}ms p99={:.3}ms max={:.3}ms drift={} frames",
        fps as u32,
        secs,
        s.frames,
        s.p50 * 1e3,
        s.p99 * 1e3,
        s.max * 1e3,
        s.drift
    );

    let ok = s.frames >= 2 && s.p99 < 1e-3 && s.drift == 0;
    println!(
        "target: p99 < 1.000ms, drift = 0  -> {}",
        if ok { "OK" } else { "FAIL" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
