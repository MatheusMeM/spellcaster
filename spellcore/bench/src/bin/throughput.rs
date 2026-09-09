// bench `throughput`: 16 universos sACN + 16 Art-Net a 60 Hz, %CPU de um nucleo, RSS e
// boot ate o primeiro frame DMX. Gate do PRD: cpu < 3 %, rss < 60 MB, boot < 2 s.
// Saida ASCII pura (console cp1252).

use bench::arg_f64;
use engine::clock::Clock;
use engine::universe::Universes;
use protocols::artnet::ArtNetOut;
use protocols::sacn::SacnOut;
use protocols::Output;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// ---- CPU e RSS do proprio processo, sem dependencia extra ----

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> isize;
    fn GetProcessTimes(h: isize, c: *mut u64, e: *mut u64, k: *mut u64, u: *mut u64) -> i32;
}

#[cfg(windows)]
#[repr(C)]
// PROCESS_MEMORY_COUNTERS: so' `cb` (tamanho) e `working_set_size` sao lidos; o resto e' o
// tamanho certo da struct para a API nao escrever fora.
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

/// Tempo de CPU do processo (usuario + kernel), em segundos.
#[cfg(windows)]
fn cpu_secs() -> Option<f64> {
    let (mut c, mut e, mut k, mut u) = (0u64, 0u64, 0u64, 0u64);
    unsafe {
        if GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) == 0 {
            return None;
        }
    }
    Some((k + u) as f64 * 1e-7) // FILETIME conta 100 ns
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
    // ponytail: campos 14 (utime) e 15 (stime) do /proc/self/stat, tick fixo em 100 Hz ;
    // trocar por sysconf(_SC_CLK_TCK) se aparecer kernel com HZ diferente.
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
            println!("throughput: sACN nao abriu: {} -> FALHA", e);
            std::process::exit(1);
        }
    };
    let mut art = match ArtNetOut::new(Some(vec!["127.0.0.1".to_string()]), false) {
        Ok(o) => o,
        Err(e) => {
            println!("throughput: Art-Net nao abriu: {} -> FALHA", e);
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

    // ponytail: laco sem alocacao - iter() sobre buffers ja preenchidos, send() copia p/ a fila ;
    // se um dia o show mudar por frame, entra Timeline::apply aqui antes dos sends.
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
        "throughput {} sacn + {} artnet @{}Hz {}s: frames={} cpu={} wall={:.3}s = {} de um nucleo",
        half,
        nu - half,
        fps,
        secs,
        frames,
        cpu_txt,
        wall,
        pct_txt
    );
    println!("rss={}  boot_ate_primeiro_frame={:.3}s", rss_txt, boot_s);

    let ok = ok_cpu && ok_rss && boot_s < 2.0;
    println!(
        "alvo: cpu < 3.00%, rss < 60MB, boot < 2.000s  -> {}",
        if ok { "OK" } else { "FALHA" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
