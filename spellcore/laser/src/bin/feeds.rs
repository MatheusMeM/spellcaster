// R4 acceptance: 4 feeds at 30 kpps against 4 Ether Dream emulators on loopback.
// Measures process CPU with GetProcessTimes and the CPU of each feed thread with
// GetThreadTimes (the emulator lives in the SAME process and its cost is not the feed's).
// PRD target: 4 laser feeds at 30 kpps < 1 % of a core. Pure ASCII output (cp1252).
//
//   cargo run --release -p laser --bin feeds -- --secs 10 --feeds 4 --pps 30000

use std::time::{Duration, Instant};

use laser::feed::Transform;
use laser::frame::{Point, Safety};
use laser::{Emulator, EtherDream, Feed};

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> isize;
    fn GetProcessTimes(h: isize, c: *mut u64, e: *mut u64, k: *mut u64, u: *mut u64) -> i32;
}

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(p: u32) -> u32;
    fn timeEndPeriod(p: u32) -> u32;
}

/// Process CPU time (user + kernel), in seconds.
#[cfg(windows)]
fn cpu_secs() -> Option<f64> {
    let (mut c, mut e, mut k, mut u) = (0u64, 0u64, 0u64, 0u64);
    unsafe {
        if GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) == 0 {
            return None;
        }
    }
    Some((k + u) as f64 * 1e-7) // FILETIME counts 100 ns
}

#[cfg(not(windows))]
fn cpu_secs() -> Option<f64> {
    // ponytail: fields 14/15 of /proc/self/stat, tick fixed at 100 Hz ; same as bench/throughput
    let s = std::fs::read_to_string("/proc/self/stat").ok()?;
    let f: Vec<&str> = s.rsplit(')').next()?.split_whitespace().collect();
    let ut: f64 = f.get(11)?.parse().ok()?;
    let st: f64 = f.get(12)?.parse().ok()?;
    Some((ut + st) / 100.0)
}

fn arg(name: &str, default: f64) -> f64 {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == name)
        .and_then(|i| a.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Figure of `n` points that passes safety (bbox well above min_size) and has vertices.
fn figura(n: usize) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * 3.0 * i as f64 / n as f64;
            let r = 9000.0 + 6000.0 * (a * 2.0).sin();
            Point::new(r * a.cos(), r * a.sin(), 255, (i & 0xff) as u8, 80, false)
        })
        .collect()
}

fn main() {
    let secs = arg("--secs", 10.0).max(0.5);
    let nfeeds = arg("--feeds", 4.0).max(1.0) as usize;
    let pps = arg("--pps", 30000.0).max(1000.0) as u32;
    let npts = arg("--points", 1200.0).max(50.0) as usize;
    let capacity = arg("--capacity", 1800.0) as u16;
    let chunk = arg("--chunk", 0.0) as usize;
    let fps = pps as f64 / npts as f64; // frames per second per feed
    let dt = Duration::from_secs_f64(1.0 / fps);

    #[cfg(windows)]
    unsafe {
        timeBeginPeriod(1);
    }

    let mut emus = Vec::new();
    let mut feeds = Vec::new();
    for _ in 0..nfeeds {
        let emu = match Emulator::start(capacity) {
            Ok(e) => e,
            Err(e) => {
                println!("feeds: emulator did not start: {e} -> FAIL");
                std::process::exit(1);
            }
        };
        emu.record(false); // keeping 1.2 M points/s would skew the measurement
        let dac = match EtherDream::connect(&format!("127.0.0.1:{}", emu.port), capacity) {
            Ok(mut d) => {
                if chunk > 0 {
                    d.set_chunk(chunk);
                }
                d
            }
            Err(e) => {
                println!("feeds: connection to the emulator failed: {e} -> FAIL");
                std::process::exit(1);
            }
        };
        match Feed::start(Box::new(dac), pps, 2, Safety::default()) {
            Ok(f) => feeds.push(f),
            Err(e) => {
                println!("feeds: Feed::start failed: {e} -> FAIL");
                std::process::exit(1);
            }
        }
        emus.push(emu);
    }

    let pts = figura(npts);
    let cpu0 = cpu_secs();
    let wall0 = Instant::now();
    let mut next = Instant::now();
    let mut pushed = 0u64;
    // ponytail: sleep-based pacer with timeBeginPeriod(1), no spin ; the one that has to be
    // precise is the engine Clock, here it is only about feeding the feeds at the right pace.
    while wall0.elapsed().as_secs_f64() < secs {
        next += dt;
        let rot = wall0.elapsed().as_secs_f64() * 45.0;
        for (i, f) in feeds.iter().enumerate() {
            f.set_transform(Transform {
                rot: rot + i as f64 * 90.0,
                ..Transform::default()
            });
            f.push(&pts);
        }
        pushed += 1;
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else {
            next = now;
        }
    }
    let wall = wall0.elapsed().as_secs_f64();
    let cpu = match (cpu0, cpu_secs()) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };

    for f in feeds.iter_mut() {
        f.stop();
    }
    let stats: Vec<_> = feeds
        .iter()
        .map(|f| (f.name().to_string(), f.stats()))
        .collect();
    let recebidos: u64 = emus.iter().map(|e| e.count()).sum();
    // round trips per command: that is what costs CPU (each one is a context ping-pong)
    let mut hist = std::collections::BTreeMap::new();
    for e in &emus {
        for c in e.commands() {
            *hist.entry(c).or_insert(0u64) += 1;
        }
    }
    let idas: u64 = hist.values().sum();
    drop(feeds);
    drop(emus);

    #[cfg(windows)]
    unsafe {
        timeEndPeriod(1);
    }

    println!(
        "feeds: {} DACs x {} pps x {} points/frame ({:.1} fps) for {:.1}s",
        nfeeds, pps, npts, fps, wall
    );
    let mut cpu_feeds = 0.0;
    for (name, s) in &stats {
        cpu_feeds += s.cpu;
        println!(
            "  {:<28} sent={:<6} drop={:<5} err={:<3} jitter p50={:.3}ms p99={:.3}ms max={:.3}ms cpu={:.3}s",
            name,
            s.sent,
            s.dropped,
            s.errors,
            s.p50 * 1e3,
            s.p99 * 1e3,
            s.max * 1e3,
            s.cpu
        );
    }
    let pontos: u64 = stats.iter().map(|(_, s)| s.sent).sum::<u64>() * npts as u64;
    println!(
        "push={} frames/feed  points delivered={} (emulators received {})",
        pushed, pontos, recebidos
    );
    let cmds: Vec<String> = hist
        .iter()
        .map(|(c, n)| format!("{}={}", *c as char, n))
        .collect();
    println!(
        "commands to the DAC: {}  total={} ({:.0}/s), {:.1} us of feed cpu per round trip",
        cmds.join(" "),
        idas,
        idas as f64 / wall,
        if idas > 0 {
            cpu_feeds * 1e6 / idas as f64
        } else {
            0.0
        }
    );

    let pct_feeds = 100.0 * cpu_feeds / wall;
    let (proc_txt, pct_proc) = match cpu {
        Some(c) => (format!("{c:.3}s"), 100.0 * c / wall),
        None => ("n/a".to_string(), f64::NAN),
    };
    println!(
        "cpu of the {} feed threads = {:.3}s = {:.2}% of a core",
        nfeeds, cpu_feeds, pct_feeds
    );
    println!(
        "cpu of the whole process (GetProcessTimes, INCLUDES the {} emulators and the pacer) = {} = {:.2}%",
        nfeeds, proc_txt, pct_proc
    );
    let erros: u64 = stats.iter().map(|(_, s)| s.errors).sum();
    let ok = pct_feeds < 1.0 && erros == 0 && pontos > 0;
    println!(
        "target: feed cpu < 1.00% of a core, 0 errors -> {}",
        if ok { "OK" } else { "FAIL" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
