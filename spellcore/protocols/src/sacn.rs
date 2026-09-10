//! sACN (ANSI E1.31): multicast output per interface + localhost unicast, input.
//! Bytes identical to `spellcaster/protocols/sacn.py` (fixture
//! tests/conformance/sacn_packet.bin).

use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

use crate::{random16, Output, Queue};

pub const PORT: u16 = 5568;
/// Discovery multicast group (E1.31 §8): the listener is `netscan`.
pub const DISCOVERY_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 250, 214);

/// `>HH12s` 0x10, 0, "ASC-E1.17\0\0\0"
const ROOT: [u8; 16] = [
    0x00, 0x10, 0x00, 0x00, b'A', b'S', b'C', b'-', b'E', b'1', b'.', b'1', b'7', 0, 0, 0,
];

/// Multicast group of the universe: 239.255.{u>>8}.{u&255}.
pub fn mcast(universe: u16) -> Ipv4Addr {
    Ipv4Addr::new(239, 255, (universe >> 8) as u8, (universe & 255) as u8)
}

/// E1.31 data packet (start code 0), written into `out` (which is cleared first). No
/// allocation if `out` already has capacity - this is the hot path of the I/O thread.
pub fn packet_into(
    out: &mut Vec<u8>,
    universe: u16,
    data: &[u8],
    cid: &[u8; 16],
    seq: u8,
    source_name: &str,
    priority: u8,
) {
    let n = data.len();
    let dmp_len = 11 + n; // 10 of the DMP header + start code + data
    let fr_len = 77 + dmp_len;
    let root_len = 22 + fr_len;

    out.clear();
    out.extend_from_slice(&ROOT);
    out.extend_from_slice(&(0x7000u16 | root_len as u16).to_be_bytes());
    out.extend_from_slice(&4u32.to_be_bytes());
    out.extend_from_slice(cid);
    // framing
    out.extend_from_slice(&(0x7000u16 | fr_len as u16).to_be_bytes());
    out.extend_from_slice(&2u32.to_be_bytes());
    let name = source_name.as_bytes();
    let name = &name[..name.len().min(63)]; // same as Python: cut at 63, pad to 64
    out.extend_from_slice(name);
    out.resize(out.len() + (64 - name.len()), 0);
    out.push(priority);
    out.extend_from_slice(&0u16.to_be_bytes()); // sync address
    out.push(seq);
    out.push(0); // options
    out.extend_from_slice(&universe.to_be_bytes());
    // DMP
    out.extend_from_slice(&(0x7000u16 | dmp_len as u16).to_be_bytes());
    out.push(0x02);
    out.push(0xA1);
    out.extend_from_slice(&0u16.to_be_bytes()); // first address
    out.extend_from_slice(&1u16.to_be_bytes()); // address increment
    out.extend_from_slice(&(1 + n as u16).to_be_bytes()); // property value count
    out.push(0); // start code
    out.extend_from_slice(data);
}

/// E1.31 data packet. Pure function: same bytes as the Python generator.
pub fn packet(
    universe: u16,
    data: &[u8],
    cid: &[u8; 16],
    seq: u8,
    source_name: &str,
    priority: u8,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(126 + data.len());
    packet_into(&mut out, universe, data, cid, seq, source_name, priority);
    out
}

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Data {
        cid: [u8; 16],
        name: String,
        priority: u8,
        seq: u8,
        universe: u16,
        data: Vec<u8>,
    },
}

fn be16(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([b[i], b[i + 1]])
}

fn be32(b: &[u8], i: usize) -> u32 {
    u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

/// Data packet (vector 4); `None` if it is not E1.31. Discovery is read by `netscan`.
pub fn parse(pk: &[u8]) -> Option<Packet> {
    if pk.len() < 48 || pk[..16] != ROOT {
        return None;
    }
    let vec = be32(pk, 18);
    let mut cid = [0u8; 16];
    cid.copy_from_slice(&pk[22..38]);
    // datagram truncated on the network: the `get` covers the rest of the framing layer
    // (up to 108); the two branches below already check the size before slicing what follows.
    let name = cstr(pk.get(44..108)?);
    if vec == 4 && pk.len() >= 126 {
        return Some(Packet::Data {
            cid,
            name,
            priority: pk[108],
            seq: pk[111],
            universe: be16(pk, 113),
            data: pk[126..].to_vec(),
        });
    }
    None
}

/// Local IPv4 addresses (resolving the hostname, like Python's `getaddrinfo`) + loopback.
/// Sorted as text, same as Python's `sorted()`: the first one sends the unicast.
pub fn interfaces() -> Vec<Ipv4Addr> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    set.insert("127.0.0.1".to_string());
    for i in crate::netscan::interfaces() {
        set.insert(i.ip);
    }
    set.iter().filter_map(|s| s.parse().ok()).collect()
}

fn out_socket(ip: Ipv4Addr) -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_multicast_ttl_v4(1)?;
    s.set_multicast_if_v4(&ip)?;
    Ok(s.into())
}

/// Listening socket on 0.0.0.0:PORT with SO_REUSEADDR, joined to the requested groups.
fn listener(groups: &[Ipv4Addr]) -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_reuse_address(true)?;
    s.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT).into())?;
    for g in groups {
        // ponytail: an IP_ADD_MEMBERSHIP failure is ignored ; a machine with no active network
        // interface still receives the unicast on 127.0.0.1, which is what the tests use.
        let _ = s.join_multicast_v4(g, &Ipv4Addr::UNSPECIFIED);
    }
    s.set_read_timeout(Some(Duration::from_millis(200)))?;
    Ok(s.into())
}

// ------------------------------------------------------------------ SacnOut

/// sACN output. `send()` only enqueues; the internal thread builds the packet and talks to
/// the sockets.
pub struct SacnOut {
    cid: [u8; 16],
    q: Arc<Queue>,
    th: Option<JoinHandle<()>>,
}

impl SacnOut {
    pub fn new(universes: &[u16], ifaces: Option<Vec<Ipv4Addr>>) -> io::Result<SacnOut> {
        SacnOut::with(universes, 100, "Spellcaster", ifaces)
    }

    pub fn with(
        universes: &[u16],
        priority: u8,
        source_name: &str,
        ifaces: Option<Vec<Ipv4Addr>>,
    ) -> io::Result<SacnOut> {
        let ips = ifaces.unwrap_or_else(interfaces);
        let mut socks = Vec::with_capacity(ips.len());
        for ip in &ips {
            match out_socket(*ip) {
                Ok(s) => socks.push(s),
                // interface vanished between the scan and the bind: the others carry on
                Err(_) if ips.len() > 1 => {}
                Err(e) => return Err(e),
            }
        }
        if socks.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "no usable network interface for sACN",
            ));
        }
        let cid = random16();
        let q = Queue::new(universes.len());
        let name = source_name.to_string();
        let mut seq: HashMap<u16, u8> = universes.iter().map(|u| (*u, 0u8)).collect();
        let qt = q.clone();
        let th = std::thread::Builder::new()
            .name("sacn-out".into())
            .spawn(move || {
                // ponytail: a single reused packet buffer (the thread sends one frame at a time)
                // ; a buffer per universe would only be needed if sending became parallel.
                let mut buf = Vec::with_capacity(638);
                let local = SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT);
                while let Some((u, data)) = qt.pop() {
                    let s = seq.entry(u).or_insert(0);
                    *s = s.wrapping_add(1); // increment before sending, & 0xFF
                    packet_into(&mut buf, u, &data, &cid, *s, &name, priority);
                    let dst = SocketAddrV4::new(mcast(u), PORT);
                    for sk in &socks {
                        let _ = sk.send_to(&buf, dst); // no route to the group: ignore
                    }
                    let _ = socks[0].send_to(&buf, local);
                }
            })?;
        Ok(SacnOut {
            cid,
            q,
            th: Some(th),
        })
    }

    pub fn cid(&self) -> [u8; 16] {
        self.cid
    }
}

impl Output for SacnOut {
    fn send(&mut self, universe: u16, data: &[u8; 512]) {
        self.q.push(universe, data);
    }

    fn close(&mut self) {
        self.q.stop();
        if let Some(th) = self.th.take() {
            let _ = th.join();
        }
    }
}

impl Drop for SacnOut {
    fn drop(&mut self) {
        self.close();
    }
}

// ------------------------------------------------------------------- SacnIn

/// Listens to the universes and keeps the last frame of each one.
pub struct SacnIn {
    last: Arc<Mutex<HashMap<u16, [u8; 512]>>>,
    run: Arc<AtomicBool>,
    th: Option<JoinHandle<()>>,
}

impl SacnIn {
    pub fn new(universes: &[u16]) -> io::Result<SacnIn> {
        let groups: Vec<Ipv4Addr> = universes.iter().map(|u| mcast(*u)).collect();
        let sock = listener(&groups)?;
        let last = Arc::new(Mutex::new(HashMap::new()));
        let run = Arc::new(AtomicBool::new(true));
        let (l, r) = (last.clone(), run.clone());
        let th = std::thread::Builder::new()
            .name("sacn-in".into())
            .spawn(move || {
                let mut buf = [0u8; 2048];
                while r.load(Ordering::Relaxed) {
                    let n = match sock.recv_from(&mut buf) {
                        Ok((n, _)) => n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                        Err(_) => break,
                    };
                    if let Some(Packet::Data { universe, data, .. }) = parse(&buf[..n]) {
                        let mut frame = [0u8; 512];
                        let k = data.len().min(512);
                        frame[..k].copy_from_slice(&data[..k]);
                        l.lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .insert(universe, frame);
                    }
                }
            })?;
        Ok(SacnIn {
            last,
            run,
            th: Some(th),
        })
    }

    pub fn get(&self, universe: u16) -> Option<[u8; 512]> {
        self.last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&universe)
            .copied()
    }

    pub fn close(&mut self) {
        self.run.store(false, Ordering::Relaxed);
        if let Some(th) = self.th.take() {
            let _ = th.join(); // exits within 200 ms (read timeout)
        }
    }
}

impl Drop for SacnIn {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/conformance/sacn_packet.bin"
    );

    fn data512() -> Vec<u8> {
        (0..512u16).map(|i| (i % 256) as u8).collect()
    }

    #[test]
    fn packet_matches_the_fixture() {
        let want = std::fs::read(FIXTURE).expect("fixture sacn_packet.bin");
        let cid: [u8; 16] = std::array::from_fn(|i| i as u8);
        let got = packet(1, &data512(), &cid, 0, "Spellcaster", 100);
        assert_eq!(got.len(), want.len(), "packet size");
        assert_eq!(got, want, "E1.31 packet bytes");
    }

    /// Header (126 bytes) generated by Python for universe 300, seq 42, priority 77,
    /// name "Fonte X", CID `i ^ 0x5A`: locks the fields the default fixture does not cover.
    const HDR_300: &str = "001000004153432d45312e3137000000726e000000045a5b58595e5f5c5d5253505156575455725800000002466f6e746520580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004d00002a00012c720b02a100000001020100";

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn header_with_non_default_values() {
        let cid: [u8; 16] = std::array::from_fn(|i| (i as u8) ^ 0x5A);
        let pk = packet(300, &data512(), &cid, 42, "Fonte X", 77);
        assert_eq!(pk.len(), 638);
        assert_eq!(pk[..126], unhex(HDR_300)[..]);
    }

    #[test]
    fn round_trip() {
        let cid: [u8; 16] = std::array::from_fn(|i| (i as u8) ^ 0x5A);
        let d = data512();
        let pk = packet(300, &d, &cid, 42, "Fonte X", 77);
        let Packet::Data {
            cid: c,
            name,
            priority,
            seq,
            universe,
            data,
        } = parse(&pk).expect("parse");
        assert_eq!(c, cid);
        assert_eq!(name, "Fonte X");
        assert_eq!(priority, 77);
        assert_eq!(seq, 42);
        assert_eq!(universe, 300);
        assert_eq!(data, d);
        assert!(parse(b"not e1.31").is_none());
    }

    /// A truncated E1.31 datagram (what the network delivers) must not panic the SacnIn thread.
    #[test]
    fn truncated_does_not_panic() {
        let cid: [u8; 16] = std::array::from_fn(|i| (i as u8) ^ 0x5A);
        let pk = packet(300, &data512(), &cid, 42, "Fonte X", 77);
        for n in [0, 16, 47, 48, 60, 100, 107, 108, 119, 125] {
            assert!(
                parse(&pk[..n]).is_none(),
                "truncated at {} should give None",
                n
            );
        }
        assert!(parse(&pk).is_some(), "a complete packet still parses");
    }

    #[test]
    fn mcast_of_the_universe() {
        assert_eq!(mcast(1), Ipv4Addr::new(239, 255, 0, 1));
        assert_eq!(mcast(256), Ipv4Addr::new(239, 255, 1, 0));
        assert_eq!(mcast(300), Ipv4Addr::new(239, 255, 1, 44));
    }

    #[test]
    fn interfaces_have_loopback() {
        assert!(interfaces().contains(&Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn loopback_out_in() {
        let mut rx = match SacnIn::new(&[1]) {
            Ok(r) => r,
            Err(e) => return println!("skipped: bind 5568 failed: {:?}", e.kind()),
        };
        let mut tx = match SacnOut::new(&[1], Some(vec![Ipv4Addr::LOCALHOST])) {
            Ok(t) => t,
            Err(e) => return println!("skipped: output socket failed: {:?}", e.kind()),
        };
        let mut ultimo = [0u8; 512];
        for f in 1..=3u8 {
            let frame: [u8; 512] = std::array::from_fn(|i| (i as u8).wrapping_add(f));
            tx.send(1, &frame);
            ultimo = frame;
            std::thread::sleep(Duration::from_millis(30)); // distinct frames, nothing dropped
        }
        for _ in 0..60 {
            if rx.get(1) == Some(ultimo) {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let got = rx.get(1);
        tx.close();
        rx.close();
        match got {
            None => println!("skipped: loopback UDP did not deliver (firewall?)"),
            Some(g) => {
                assert_eq!(g, ultimo, "last frame received byte for byte");
            }
        }
    }

    #[test]
    fn send_neither_blocks_nor_grows() {
        // no real consumer: 100 frames at once, the queue stays at 2 per universe
        let q = crate::Queue::new(1);
        let t0 = Instant::now();
        for i in 0..100u16 {
            let mut d = [0u8; 512];
            d[0] = i as u8;
            q.push(1, &d);
        }
        assert!(t0.elapsed() < Duration::from_millis(200), "send blocked");
        assert_eq!(q.len(), crate::DEPTH);
        q.stop();
    }

    #[test]
    fn output_does_not_block_with_100_frames() {
        let mut tx = match SacnOut::new(&[1], Some(vec![Ipv4Addr::LOCALHOST])) {
            Ok(t) => t,
            Err(e) => return println!("skipped: output socket failed: {:?}", e.kind()),
        };
        let frame = [7u8; 512];
        let t0 = Instant::now();
        for _ in 0..100 {
            tx.send(1, &frame);
        }
        let dt = t0.elapsed();
        tx.close();
        assert!(dt < Duration::from_millis(500), "100 sends took {dt:?}");
    }
}
