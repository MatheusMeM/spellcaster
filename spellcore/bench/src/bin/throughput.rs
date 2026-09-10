// bench `throughput`: 16 sACN universes + 16 Art-Net at 60 Hz, %CPU of a core, RSS and boot
// until the first DMX frame. PRD gate: cpu < 3 %, rss < 60 MB, boot < 2 s.
// Pure ASCII output (cp1252 console).

use bench::arg_f64;
use engine::clock::Clock;
use engine::universe::Universes;
use protocols::artnet::ArtNetOut;
use protocols::sacn::SacnOut;
use protocols::Output;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// ---- CPU and RSS of the process itself, with no extra dependency ----

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> isize;
    fn GetProcessTimes(h: isize, c: *mut u64, e: *mut u64, k: *mut u64, u: *mut u64) -> i32;
}

#[cfg(windows)]
#[repr(C)]
// PROCESS_MEMORY_COUNTERS: only `cb` (size) and `working_set_size` are read; the rest is the
// right struct size so the API does not write past it.
struct ProcessMemoryCounters {
    cb: u32,
    _page_faults: u32,
    _peak_ws: usize,
    working_set_size: usize,
    _resto: [usize; 6],
}

#[cfg(windows)]
#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(h: isize, c: *mut ProcessMemoryCounters, cb: u32) -> i32;
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

#[cfg(windows)]
fn rss_bytes() -> Option<u64> {
    let mut m: ProcessMemoryCounters = unsafe { std::mem::zeroed() };
    m.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
    unsafe {
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut m, m.cb) == 0 {
            return None;
        }
    }
    Some(m.working_set_size as u64)
}

#[cfg(not(windows))]
fn cpu_secs() -> Option<f64> {
    // ponytail: fields 14 (utime) and 15 (stime) of /proc/self/stat, tick fixed at 100 Hz ;
    // switch to sysconf(_SC_CLK_TCK) if a kernel with a different HZ shows up.
    let s = std::fs::read_to_string("/proc/self/stat").ok()?;
    let f: Vec<&str> = s.rsplit(')').next()?.split_whitespace().collect();
    let ut: f64 = f.get(11)?.parse().ok()?;
    let st: f64 = f.get(12)?.parse().ok()?;
    Some((ut + st) / 100.0)
}

#[cfg(not(windows))]
fn rss_bytes() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = s.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4096)
}

fn main() {
    let boot0 = Instant::now();
    let secs = arg_f64("--secs", 10.0).max(0.1);
    let nu = arg_f64("--universes", 32.0).max(2.0) as u16;
    let half = nu / 2;
    let fps = 60u32;

    let sacn_u: Vec<u16> = (1..=half).collect();
    let mut sacn = match SacnOut::new(&sacn_u, Some(vec![Ipv4Addr::LOCALHOST])) {
        Ok(o) => o,
        Err(e) => {
            println!("throughput: sACN did not open: {} -> FAIL", e);
            std::process::exit(1);
        }
    };
    let mut art = match ArtNetOut::new(Some(vec!["127.0.0.1".to_string()]), false) {
        Ok(o) => o,
        Err(e) => {
            println!("throughput: Art-Net did not open: {} -> FAIL", e);
            std::process::exit(1);
        }
    };

    let mut universes = Universes::new();
    let mut pattern = [0u8; 512];
    for n in 1..=nu {
        for (i, b) in pattern.iter_mut().enumerate() {
            *b = ((i as u16 + n) & 0xff) as u8;
        }
        universes.get_or_create(n).set_bytes(1, &pattern);
    }

    let mut frames: u64 = 0;
    let mut boot: Option<Duration> = None;
    let cpu0 = cpu_secs();
    let wall0 = Instant::now();

    // ponytail: allocation-free loop - iter() over already filled buffers, send() copies to the
    // queue ; if some day the show changes per frame, Timeline::apply goes here before the sends.
    Clock::new(fps).run(
        |_t| {
            for (i, u) in universes.iter().enumerate() {
                if i < half as usize {
                    sacn.send(u.number, &u.data);
                } else {
                    art.send(u.number, &u.data);
                }
            }
            frames += 1;
            if boot.is_none() {
                boot = Some(boot0.elapsed());
            }
        },
        Some(secs),
    );

    let wall = wall0.elapsed().as_secs_f64();
    let cpu = match (cpu0, cpu_secs()) {
        (Some(a), Some(b)) => Some(b - a),
        _ => None,
    };
    let rss = rss_bytes();
    sacn.close();
    art.close();

    let (cpu_txt, pct_txt, ok_cpu) = match cpu {
        Some(c) => {
            let p = 100.0 * c / wall;
            (format!("{:.3}s", c), format!("{:.2}%", p), p < 3.0)
        }
        None => ("n/a".to_string(), "n/a".to_string(), true),
    };
    let (rss_txt, ok_rss) = match rss {
        Some(r) => {
            let mb = r as f64 / 1_048_576.0;
            (format!("{:.1}MB", mb), mb < 60.0)
        }
        None => ("n/a".to_string(), true),
    };
    let boot_s = boot.map(|b| b.as_secs_f64()).unwrap_or(f64::INFINITY);

    println!(
        "throughput {} sacn + {} artnet @{}Hz {}s: frames={} cpu={} wall={:.3}s = {} of a core",
        half,
        nu - half,
        fps,
        secs,
        frames,
        cpu_txt,
        wall,
        pct_txt
    );
    println!("rss={}  boot_to_first_frame={:.3}s", rss_txt, boot_s);

    let ok = ok_cpu && ok_rss && boot_s < 2.0;
    println!(
        "target: cpu < 3.00%, rss < 60MB, boot < 2.000s  -> {}",
        if ok { "OK" } else { "FAIL" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
