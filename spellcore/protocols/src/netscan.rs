//! Network analysis: network interfaces, Art-Net nodes (ArtPoll), sACN sources (discovery),
//! Ether Dream (UDP beacon 7654) and configuration suggestions.
//! Mirrors `spellcaster/protocols/netscan.py`. All output text is ASCII (cp1252 console).

use std::io::{self, Read};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream, UdpSocket};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Serialize;
use socket2::{Domain, Protocol, Socket, Type};

pub const ETHERDREAM_PORT: u16 = 7654;
/// Point stream port, used here only to ask for the status when the beacon does not arrive.
/// It is the same as `laser::dac::etherdream::TCP_PORT` - `laser` depends on `protocols`, not
/// the other way around, and the number comes from the protocol, not from the other crate.
pub const ETHERDREAM_TCP_PORT: u16 = 7765;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Iface {
    pub name: String,
    pub ip: String,
    /// Empty when the system did not report it (`suggest` assumes /24, like Python).
    pub mask: String,
    pub gateway: Option<String>,
}

// ------------------------------------------------------------------ IP utils

fn to_u32(ip: &str) -> u32 {
    ip.parse::<Ipv4Addr>().map(u32::from).unwrap_or(0)
}

fn from_u32(v: u32) -> String {
    Ipv4Addr::from(v).to_string()
}

/// CIDR prefix -> dotted mask.
pub fn mask_from_prefix(prefix: u32) -> String {
    let m = if prefix == 0 {
        0
    } else {
        0xFFFF_FFFFu32 << (32 - prefix.min(32))
    };
    from_u32(m)
}

/// Network address.
pub fn net_of(ip: &str, mask: &str) -> String {
    from_u32(to_u32(ip) & to_u32(mask))
}

/// Broadcast address of the subnet.
pub fn bcast_of(ip: &str, mask: &str) -> String {
    from_u32(to_u32(ip) | !to_u32(mask))
}

/// First IPv4 literal in the text (equivalent to Python's `\d{1,3}(\.\d{1,3}){3}`).
fn first_ipv4(s: &str) -> Option<String> {
    for tok in s.split(|c: char| !c.is_ascii_digit() && c != '.') {
        let parts: Vec<&str> = tok.split('.').collect();
        if parts.len() == 4
            && parts
                .iter()
                .all(|p| (1..=3).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit()))
        {
            return Some(tok.to_string());
        }
    }
    None
}

fn only_ipv4(line: &str) -> Option<String> {
    let t = line.trim();
    match first_ipv4(t) {
        Some(ip) if ip == t => Some(ip),
        _ => None,
    }
}

// ------------------------------------------------------------- interfaces

fn skip_words(s: &str, n: usize) -> &str {
    let mut r = s;
    for _ in 0..n {
        r = match r.find(char::is_whitespace) {
            Some(i) => r[i..].trim_start(),
            None => "",
        };
    }
    r
}

/// Network interface name taken from the `ipconfig` header line
/// ("Ethernet adapter X:", "Adaptador de Rede sem Fio X:", "Adaptador Ethernet X:").
fn iface_name(line: &str) -> String {
    let t = line.trim();
    let body = t.strip_suffix(':').unwrap_or(t).trim();
    if let Some(p) = body.find(" adapter ") {
        let n = body[p + " adapter ".len()..].trim();
        if !n.is_empty() {
            return n.to_string();
        }
    }
    if let Some(rest) = body.strip_prefix("Adaptador ") {
        let rest = rest.trim_start();
        let n = if let Some(r) = rest.strip_prefix("de Rede sem Fio ") {
            r
        } else if rest.starts_with("de T") {
            skip_words(rest, 2) // "de Tunel <name>" (the accent may arrive mangled)
        } else {
            skip_words(rest, 1)
        };
        let n = n.trim();
        if !n.is_empty() {
            return n.to_string();
        }
    }
    body.to_string()
}

/// `ipconfig` output (pt-BR or en, accents may arrive mangled) -> interfaces with IPv4.
pub fn parse_ipconfig(text: &str) -> Vec<Iface> {
    let mut ifaces: Vec<Iface> = Vec::new();
    let mut want_gw = false;
    for line in text.lines() {
        let head = !line.is_empty()
            && !line.starts_with(|c: char| c.is_whitespace())
            && line.trim_end().ends_with(':');
        if head {
            ifaces.push(Iface {
                name: iface_name(line),
                ip: String::new(),
                mask: String::new(),
                gateway: None,
            });
            want_gw = false;
            continue;
        }
        let Some(cur) = ifaces.last_mut() else {
            continue;
        };
        if want_gw {
            if let Some(ip) = only_ipv4(line) {
                cur.gateway = Some(ip); // IPv4 on the line after the IPv6 gateway
                want_gw = false;
                continue;
            }
        }
        let Some((label, value)) = line.split_once(':') else {
            continue;
        };
        want_gw = false;
        if label.contains("IPv4") {
            if let Some(ip) = first_ipv4(value) {
                cur.ip = ip;
            }
        } else if label.contains("scara") || label.contains("Mask") {
            if let Some(m) = first_ipv4(value) {
                cur.mask = m;
            }
        } else if label.contains("Gateway") {
            cur.gateway = first_ipv4(value);
            want_gw = cur.gateway.is_none();
        }
    }
    ifaces.retain(|i| !i.ip.is_empty());
    ifaces
}

/// Linux fallback: `ip addr` output (+ `ip route` for the gateway).
pub fn parse_ip_addr(text: &str, route_text: &str) -> Vec<Iface> {
    let mut ifaces = Vec::new();
    let mut cur = String::new();
    for line in text.lines() {
        if !line.starts_with(char::is_whitespace) {
            // "2: eth0: <UP>" -> eth0
            if let Some((idx, rest)) = line.split_once(':') {
                if idx.trim().bytes().all(|b| b.is_ascii_digit()) && !idx.trim().is_empty() {
                    cur = rest
                        .trim()
                        .split([':', '@'])
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                }
            }
            continue;
        }
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("inet ") {
            let addr = rest.split_whitespace().next().unwrap_or("");
            if let Some((ip, pfx)) = addr.split_once('/') {
                if let (Some(ip), Ok(p)) = (first_ipv4(ip), pfx.parse::<u32>()) {
                    ifaces.push(Iface {
                        name: cur.clone(),
                        ip,
                        mask: mask_from_prefix(p),
                        gateway: None,
                    });
                }
            }
        }
    }
    for line in route_text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("default via ") {
            let mut it = rest.split_whitespace();
            let (Some(gw), Some("dev"), Some(dev)) = (it.next(), it.next(), it.next()) else {
                continue;
            };
            for i in ifaces.iter_mut().filter(|i| i.name == dev) {
                i.gateway = Some(gw.to_string());
            }
        }
    }
    ifaces
}

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn interfaces_linux() -> Option<Vec<Iface>> {
    let addr: serde_json::Value = serde_json::from_str(&run("ip", &["-j", "addr"])).ok()?;
    let route: serde_json::Value = serde_json::from_str(&run("ip", &["-j", "route"])).ok()?;
    let gw = |dev: &str| -> Option<String> {
        route.as_array()?.iter().find_map(|r| {
            (r.get("dst")?.as_str()? == "default" && r.get("dev")?.as_str()? == dev)
                .then(|| r.get("gateway")?.as_str().map(String::from))
                .flatten()
        })
    };
    let mut out = Vec::new();
    for d in addr.as_array()? {
        let name = d.get("ifname").and_then(|v| v.as_str()).unwrap_or("");
        for a in d
            .get("addr_info")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if a.get("family").and_then(|v| v.as_str()) != Some("inet") {
                continue;
            }
            let (Some(ip), Some(p)) = (
                a.get("local").and_then(|v| v.as_str()),
                a.get("prefixlen").and_then(|v| v.as_u64()),
            ) else {
                continue;
            };
            out.push(Iface {
                name: name.to_string(),
                ip: ip.to_string(),
                mask: mask_from_prefix(p as u32),
                gateway: gw(name),
            });
        }
    }
    Some(out)
}

/// Active IPv4 network interfaces (loopback excluded).
pub fn interfaces() -> Vec<Iface> {
    let mut ifaces = if cfg!(windows) {
        parse_ipconfig(&run("ipconfig", &[]))
    } else {
        interfaces_linux()
            .unwrap_or_else(|| parse_ip_addr(&run("ip", &["addr"]), &run("ip", &["route"])))
    };
    ifaces.retain(|i| !i.ip.starts_with("127."));
    ifaces
}

// ---------------------------------------------------------------- Art-Net

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Port {
    pub dir: String,
    pub universe: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node {
    pub ip: String,
    pub port: u16,
    pub short_name: String,
    pub long_name: String,
    pub mac: Option<String>,
    pub ports: Vec<Port>,
    pub from: String,
}

/// NUL-terminated string of the announcements (Art-Net and E1.31): latin-1 by the letter of
/// the standard, but what the nodes send in practice is UTF-8 once it leaves ASCII.
fn latin1(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

pub fn parse_artpollreply(data: &[u8]) -> Option<Node> {
    if data.len() < 194
        || &data[..8] != b"Art-Net\0"
        || u16::from_le_bytes([data[8], data[9]]) != 0x2100
    {
        return None;
    }
    let (net, sub) = (data[18] as u16, data[19] as u16);
    let nports = u16::from_be_bytes([data[172], data[173]]).min(4) as usize;
    let (types, swin, swout) = (&data[174..178], &data[186..190], &data[190..194]);
    let mut ports = Vec::new();
    for i in 0..nports {
        if types[i] & 0x80 != 0 {
            ports.push(Port {
                dir: "out".into(),
                universe: net << 8 | sub << 4 | (swout[i] & 0xF) as u16,
            });
        }
        if types[i] & 0x40 != 0 {
            ports.push(Port {
                dir: "in".into(),
                universe: net << 8 | sub << 4 | (swin[i] & 0xF) as u16,
            });
        }
    }
    Some(Node {
        ip: format!("{}.{}.{}.{}", data[10], data[11], data[12], data[13]),
        port: u16::from_le_bytes([data[14], data[15]]),
        short_name: latin1(&data[26..44]),
        long_name: latin1(&data[44..108]),
        mac: (data.len() >= 207).then(|| {
            data[201..207]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(":")
        }),
        ports,
        from: String::new(),
    })
}

fn udp_on(ip: Ipv4Addr, port: u16, broadcast: bool) -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_reuse_address(true)?;
    if broadcast {
        s.set_broadcast(true)?;
    }
    s.bind(&SocketAddrV4::new(ip, port).into())?;
    s.set_read_timeout(Some(Duration::from_millis(50)))?;
    Ok(s.into())
}

/// IPs to try on bind, in order: the wildcard (receives from every network interface) and then
/// the IP of each interface. On Windows the wildcard bind is REFUSED with WSAEACCES (10013)
/// when another program already holds the port - Ether Dream Sitter sits on `0.0.0.0:7654` -
/// and `SO_REUSEADDR` on our side does not help, because Windows only shares if BOTH sockets
/// ask for it. Binding to the interface IP works in that case AND still receives the beacon
/// broadcast (measured: Sitter open, bind on 169.254.86.236:7654, four beacons from
/// 169.254.207.140 in 4 s).
/// Loopback closes the list (`interfaces()` does not return it): it is where the beacon of a
/// DAC emulated on the PC itself arrives, which the wildcard bind would catch and the
/// interface bind does not.
fn bind_ips(ifaces: &[Iface]) -> Vec<Ipv4Addr> {
    std::iter::once(Ipv4Addr::UNSPECIFIED)
        .chain(ifaces.iter().filter_map(|i| i.ip.parse().ok()))
        .chain(std::iter::once(Ipv4Addr::LOCALHOST))
        .collect()
}

fn udp(port: u16, broadcast: bool, ifaces: &[Iface]) -> io::Result<UdpSocket> {
    bind_ips(ifaces)
        .into_iter()
        .find_map(|ip| udp_on(ip, port, broadcast).ok())
        // last resort: ephemeral port, where only a unicast reply to the sender arrives
        .map_or_else(|| udp_on(Ipv4Addr::UNSPECIFIED, 0, broadcast), Ok)
}

/// Broadcast ArtPoll (global + 2.x + 10.x + subnet of each interface) and collect ArtPollReply.
pub fn scan_artnet(timeout: Duration, ifaces: &[Iface]) -> Vec<Node> {
    let Ok(sock) = udp(crate::artnet::PORT, true, ifaces) else {
        return Vec::new();
    };
    let mut targets: Vec<String> = ["255.255.255.255", "2.255.255.255", "10.255.255.255"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for i in ifaces.iter().filter(|i| !i.mask.is_empty()) {
        let b = bcast_of(&i.ip, &i.mask);
        if !targets.contains(&b) {
            targets.push(b);
        }
    }
    for dst in &targets {
        if let Ok(ip) = dst.parse::<Ipv4Addr>() {
            let _ = sock.send_to(
                &crate::artnet::artpoll(0, 0),
                SocketAddrV4::new(ip, crate::artnet::PORT),
            );
        }
    }
    let mut found: Vec<Node> = Vec::new();
    let mut buf = [0u8; 1024];
    let end = Instant::now() + timeout;
    while Instant::now() < end {
        let Ok((n, addr)) = sock.recv_from(&mut buf) else {
            continue;
        };
        if let Some(mut r) = parse_artpollreply(&buf[..n]) {
            if !found.iter().any(|f| f.ip == r.ip) {
                r.from = addr.ip().to_string();
                found.push(r);
            }
        }
    }
    found
}

// ------------------------------------------------------------------- sACN

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Source {
    pub cid: String,
    pub ip: String,
    pub source_name: String,
    pub universes: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Discovery {
    pub cid: String,
    pub source_name: String,
    pub universes: Vec<u16>,
}

pub fn parse_sacn_discovery(data: &[u8]) -> Option<Discovery> {
    if data.len() < 120 || &data[4..16] != b"ASC-E1.17\0\0\0" {
        return None;
    }
    let be32 = |i: usize| u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
    if be32(18) != 8 || be32(40) != 2 {
        return None; // root vector EXTENDED, framing vector DISCOVERY
    }
    let flen = (u16::from_be_bytes([data[112], data[113]]) & 0x0FFF) as usize;
    let n = flen.saturating_sub(8) / 2;
    let n = n.min((data.len() - 120) / 2);
    Some(Discovery {
        cid: data[22..38].iter().map(|b| format!("{b:02x}")).collect(),
        source_name: latin1(&data[44..108]),
        universes: (0..n)
            .map(|i| u16::from_be_bytes([data[120 + 2 * i], data[121 + 2 * i]]))
            .collect(),
    })
}

/// Joins the discovery multicast group (239.255.250.214:5568) and lists the announced sources.
pub fn scan_sacn(timeout: Duration, ifaces: &[Iface]) -> Vec<Source> {
    let s = match Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let _ = s.set_reuse_address(true);
    if s.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, crate::sacn::PORT).into())
        .is_err()
    {
        return Vec::new();
    }
    let ips: Vec<Ipv4Addr> = ifaces.iter().filter_map(|i| i.ip.parse().ok()).collect();
    let ips = if ips.is_empty() {
        vec![Ipv4Addr::UNSPECIFIED]
    } else {
        ips
    };
    for ip in ips {
        let _ = s.join_multicast_v4(&crate::sacn::DISCOVERY_IP, &ip);
    }
    let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
    let sock: UdpSocket = s.into();
    let mut found: Vec<Source> = Vec::new();
    let mut buf = [0u8; 2048];
    let end = Instant::now() + timeout;
    while Instant::now() < end {
        let Ok((n, addr)) = sock.recv_from(&mut buf) else {
            continue;
        };
        if let Some(r) = parse_sacn_discovery(&buf[..n]) {
            match found.iter_mut().find(|s| s.cid == r.cid) {
                Some(s) => {
                    for u in r.universes {
                        if !s.universes.contains(&u) {
                            s.universes.push(u);
                        }
                    }
                    s.universes.sort_unstable();
                }
                None => found.push(Source {
                    cid: r.cid,
                    ip: addr.ip().to_string(),
                    source_name: r.source_name,
                    universes: r.universes,
                }),
            }
        }
    }
    found
}

// ------------------------------------------------------------ Ether Dream

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Status {
    pub protocol: u8,
    pub light_engine_state: u8,
    pub playback_state: u8,
    pub source: u8,
    pub light_engine_flags: u16,
    pub playback_flags: u16,
    pub source_flags: u16,
    pub buffer_fullness: u16,
    pub point_rate: u32,
    pub point_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dac {
    pub ip: String,
    pub mac: String,
    pub hw_rev: u16,
    pub sw_rev: u16,
    pub buffer_capacity: u16,
    pub max_point_rate: u32,
    pub status: Status,
    /// How it was found: `beacon` (UDP 7654) or `tcp` (status asked on 7765).
    pub via: String,
}

/// Ether Dream beacon: `<6sHHHI` (16 bytes) + status `<BBBBHHHHII` (20 bytes). All
/// little-endian.
pub fn parse_beacon(b: &[u8]) -> Option<(String, u16, u16, u16, u32, Status)> {
    if b.len() < 36 {
        return None;
    }
    let le16 = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let le32 = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let mac = b[..6]
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect::<Vec<_>>()
        .join(":");
    Some((
        mac,
        le16(6),
        le16(8),
        le16(10),
        le32(12),
        parse_status(&b[16..])?,
    ))
}

/// `dac_status`, 20 little-endian bytes - the same block in the UDP beacon and in the TCP reply.
pub fn parse_status(b: &[u8]) -> Option<Status> {
    if b.len() < 20 {
        return None;
    }
    let le16 = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let le32 = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    Some(Status {
        protocol: b[0],
        light_engine_state: b[1],
        playback_state: b[2],
        source: b[3],
        light_engine_flags: le16(4),
        playback_flags: le16(6),
        source_flags: le16(8),
        buffer_fullness: le16(10),
        point_rate: le32(12),
        point_count: le32(16),
    })
}

/// IPv4 neighbors `(ip, mac)` from the ARP table: output of `arp -a` (Windows, any language) or
/// of `ip neigh` (Linux). Pure function - the text is READ, never run with an outside argument.
/// The key is the physical address on the line, not the column name: the header changes with
/// the language and arrives with mangled accents on a cp1252 console. Broadcast and multicast
/// are left out.
pub fn parse_arp(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let toks: Vec<&str> = line.split_whitespace().collect();
        let Some(mac) = toks.iter().find_map(|t| mac_of(t)) else {
            continue; // header, "Interface: ..." line, entry with no physical address
        };
        if mac == "ff:ff:ff:ff:ff:ff" || mac.starts_with("01:00:5e") {
            continue;
        }
        let Some(ip) = toks.iter().find_map(|t| only_ipv4(t)) else {
            continue;
        };
        if !out.iter().any(|(i, _)| *i == ip) {
            out.push((ip, mac));
        }
    }
    out
}

/// `8a-9e-36-98-8c-ce` or `8a:9e:36:98:8c:ce` -> lowercase `:` form.
fn mac_of(tok: &str) -> Option<String> {
    let sep = if tok.contains('-') { '-' } else { ':' };
    let parts: Vec<&str> = tok.split(sep).collect();
    (parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit())))
    .then(|| tok.replace('-', ":").to_ascii_lowercase())
}

fn arp_table() -> String {
    if cfg!(windows) {
        run("arp", &["-a"])
    } else {
        run("ip", &["neigh"])
    }
}

/// Asks each neighbor for the status over TCP: an Ether Dream sends the 22 bytes (`ack` +
/// echoed command + `dac_status`) as soon as it accepts the connection. Finds the DAC when the
/// beacon does not arrive - another program holding port 7654, blocked broadcast, or a DAC
/// still silent. Up to 32 connections at a time; each one gives up after `wait`.
pub fn probe_etherdream(neigh: &[(String, String)], port: u16, wait: Duration) -> Vec<Dac> {
    let mut out = Vec::new();
    for lote in neigh.chunks(32) {
        let achados: Vec<Option<Dac>> = std::thread::scope(|s| {
            let hs: Vec<_> = lote
                .iter()
                .map(|(ip, mac)| s.spawn(move || probe_one(ip, mac, port, wait)))
                .collect();
            hs.into_iter().map(|h| h.join().unwrap_or(None)).collect()
        });
        out.extend(achados.into_iter().flatten());
    }
    out
}

fn probe_one(ip: &str, mac: &str, port: u16, wait: Duration) -> Option<Dac> {
    let addr = SocketAddrV4::new(ip.parse().ok()?, port);
    let mut s = TcpStream::connect_timeout(&addr.into(), wait).ok()?;
    s.set_read_timeout(Some(wait)).ok()?;
    let mut b = [0u8; 22];
    s.read_exact(&mut b).ok()?;
    if b[0] != b'a' {
        return None; // another service on the same port
    }
    Some(Dac {
        ip: ip.to_string(),
        mac: mac.to_string(),
        // ponytail: the TCP status does not carry hw/sw/buffer/max_pps (only the beacon does)
        // and no number is invented ; goes away when `laser_open` can ask for the extended
        // `dac_status`.
        hw_rev: 0,
        sw_rev: 0,
        buffer_capacity: 0,
        max_point_rate: 0,
        status: parse_status(&b[2..])?,
        via: "tcp".into(),
    })
}

/// Listens for UDP 7654 beacons (1 Hz per DAC) on every network interface; if nothing arrives
/// by half the deadline, actively probes TCP 7765 on the ARP table neighbors.
pub fn scan_etherdream(timeout: Duration, ifaces: &[Iface]) -> Vec<Dac> {
    let socks: Vec<UdpSocket> = bind_ips(ifaces)
        .into_iter()
        .filter_map(|ip| udp_on(ip, ETHERDREAM_PORT, true).ok())
        .collect();
    let mut found: Vec<Dac> = Vec::new();
    let mut buf = [0u8; 256];
    let inicio = Instant::now();
    let end = inicio + timeout;
    let mut tentou_tcp = false;
    while Instant::now() < end {
        for sock in &socks {
            let Ok((n, addr)) = sock.recv_from(&mut buf) else {
                continue;
            };
            if let Some((mac, hw, sw, cap, rate, status)) = parse_beacon(&buf[..n]) {
                if let Some(d) = found.iter_mut().find(|d| d.mac == mac) {
                    d.status = status;
                    continue;
                }
                found.push(Dac {
                    ip: addr.ip().to_string(),
                    mac,
                    hw_rev: hw,
                    sw_rev: sw,
                    buffer_capacity: cap,
                    max_point_rate: rate,
                    status,
                    via: "beacon".into(),
                });
            }
        }
        if !tentou_tcp && found.is_empty() && inicio.elapsed() * 2 >= timeout {
            tentou_tcp = true;
            found = probe_etherdream(
                &parse_arp(&arp_table()),
                ETHERDREAM_TCP_PORT,
                Duration::from_millis(150),
            );
            if !found.is_empty() {
                return found;
            }
        }
    }
    found
}

// ------------------------------------------------------------- suggestions

/// Rules: Art-Net prefers 2.x/8 or 10.x/8; sACN takes any; warning for interfaces on the same
/// subnet. On Windows it includes the ready `netsh` command - TEXT, never executed.
pub fn suggest(ifaces: &[Iface]) -> Vec<String> {
    suggest_with(ifaces, cfg!(windows))
}

pub fn suggest_with(ifaces: &[Iface], windows: bool) -> Vec<String> {
    if ifaces.is_empty() {
        return vec!["No active IPv4 network interface: plug the cable or set a static IP.".into()];
    }
    let mut out = Vec::new();
    let mut nets: Vec<(String, Vec<String>)> = Vec::new();
    for i in ifaces {
        let mask = if i.mask.is_empty() {
            "255.255.255.0"
        } else {
            &i.mask
        };
        let net = net_of(&i.ip, mask);
        match nets.iter_mut().find(|(n, _)| *n == net) {
            Some((_, v)) => v.push(i.name.clone()),
            None => nets.push((net, vec![i.name.clone()])),
        }
        let first = i.ip.split('.').next().unwrap_or("");
        if first == "2" || first == "10" {
            out.push(format!(
                "{} {}/{}: Art-Net ok (network {}.x.x.x), sACN ok.",
                i.name, i.ip, mask, first
            ));
        } else {
            out.push(format!(
                "{} {}/{}: sACN ok; Art-Net prefers 2.x.x.x/8 or 10.x.x.x/8 \
                 (factory nodes on 2.x cannot see this interface).",
                i.name, i.ip, mask
            ));
            if windows {
                let last = i.ip.rsplit('.').next().unwrap_or("1");
                out.push(format!(
                    "  netsh interface ip set address name=\"{}\" static 2.0.0.{} 255.0.0.0\
                     \x20  (as administrator; not executed)",
                    i.name, last
                ));
            }
        }
    }
    for (net, names) in &nets {
        if names.len() > 1 {
            out.push(format!(
                "Warning: {} on the same subnet {}: the system sends through one only; \
                 disable the other or set the source IP.",
                names.join(", "),
                net
            ));
        }
    }
    if ifaces.len() > 1 {
        out.push(
            "sACN multicast leaves through the default route interface (gateway); \
             for another interface, set the source IP (IP_MULTICAST_IF)."
                .into(),
        );
    }
    out
}

// ----------------------------------------------------------------- report

#[derive(Debug, Clone, Serialize)]
pub struct Scan {
    pub interfaces: Vec<Iface>,
    pub suggestions: Vec<String>,
    pub artnet: Vec<Node>,
    pub sacn: Vec<Source>,
    pub etherdream: Vec<Dac>,
}

/// The three scans run in parallel; each one has its own deadline.
pub fn scan_all(timeout: Duration) -> Scan {
    let ifaces = interfaces();
    let suggestions = suggest(&ifaces);
    let (artnet, sacn, etherdream) = std::thread::scope(|s| {
        let a = s.spawn(|| scan_artnet(timeout, &ifaces));
        let b = s.spawn(|| scan_sacn(timeout + Duration::from_secs(1), &ifaces));
        let c = s.spawn(|| scan_etherdream(timeout, &ifaces));
        (
            a.join().unwrap_or_default(),
            b.join().unwrap_or_default(),
            c.join().unwrap_or_default(),
        )
    });
    Scan {
        interfaces: ifaces,
        suggestions,
        artnet,
        sacn,
        etherdream,
    }
}

/// Text report, ASCII only.
pub fn report(s: &Scan) -> String {
    let mut ln: Vec<String> = Vec::new();
    if s.interfaces.is_empty() {
        ln.push("Interfaces: (none)".into());
    } else {
        ln.push("Interfaces:".into());
        for i in &s.interfaces {
            ln.push(format!(
                "  {}: {} / {}{}",
                i.name,
                i.ip,
                i.mask,
                i.gateway
                    .as_ref()
                    .map(|g| format!("  gw {g}"))
                    .unwrap_or_default()
            ));
        }
    }
    ln.push(String::new());
    ln.push("Suggestions:".into());
    ln.extend(s.suggestions.iter().map(|x| format!("  {x}")));

    let mut section = |title: &str, items: Vec<String>| {
        ln.push(String::new());
        ln.push(title.into());
        if items.is_empty() {
            ln.push("  (nothing found)".into());
        } else {
            ln.extend(items.into_iter().map(|x| format!("  {x}")));
        }
    };
    section(
        "Art-Net (ArtPollReply):",
        s.artnet
            .iter()
            .map(|n| {
                let ports: Vec<String> = n
                    .ports
                    .iter()
                    .map(|p| format!("{} U{}", p.dir, p.universe))
                    .collect();
                format!(
                    "{}  '{}'  '{}'  [{}]",
                    n.ip,
                    n.short_name,
                    n.long_name,
                    if ports.is_empty() {
                        "no ports".to_string()
                    } else {
                        ports.join(", ")
                    }
                )
            })
            .collect(),
    );
    section(
        "sACN (discovery):",
        s.sacn
            .iter()
            .map(|x| format!("{}  '{}'  universes {:?}", x.ip, x.source_name, x.universes))
            .collect(),
    );
    section(
        "Ether Dream:",
        s.etherdream
            .iter()
            .map(|e| {
                format!(
                    "{}  mac {}  hw {} sw {}  buffer {}  max {} pps  playback {}  via {}",
                    e.ip,
                    e.mac,
                    e.hw_rev,
                    e.sw_rev,
                    e.buffer_capacity,
                    e.max_point_rate,
                    e.status.playback_state,
                    e.via
                )
            })
            .collect(),
    );
    let txt = ln.join("\n");
    debug_assert!(txt.is_ascii(), "the report must be ASCII");
    txt
}

#[cfg(test)]
mod tests {
    use super::*;

    const IPCONFIG_PTBR: &str = "\
Configuracao de IP do Windows


Adaptador Ethernet Ethernet:

   Estado da midia. . . . . . . . . . . . . .  : midia desconectada
   Sufixo DNS especifico de conexao. . . . . . :

Adaptador de Rede sem Fio Wi-Fi:

   Sufixo DNS especifico de conexao. . . . . . : lan
   Endereco IPv6 de link local . . . . . . . . : fe80::cfcb:2c7e:3705:89a0%15
   Endereco IPv4. . . . . . . .  . . . . . . . : 192.168.0.132
   Mascara de Sub-rede . . . . . . . . . . . . : 255.255.255.0
   Gateway Padrao. . . . . . . . . . . . . . . : fe80::763a:efff:fe76:d266%15
                                                 192.168.0.1

Adaptador Ethernet Ethernet 2:

   Sufixo DNS especifico de conexao. . . . . . :
   Endereco IPv4. . . . . . . . . . . . . . . . : 2.0.0.10
   Mascara de Sub-rede . . . . . . . . . . . . : 255.0.0.0
   Gateway Padrao. . . . . . . . . . . . . . . :
";

    const IPCONFIG_EN: &str = "\
Windows IP Configuration


Ethernet adapter Ethernet 2:

   Connection-specific DNS Suffix  . :
   IPv4 Address. . . . . . . . . . . : 10.0.0.5(Preferred)
   Subnet Mask . . . . . . . . . . . : 255.0.0.0
   Default Gateway . . . . . . . . . :

Wireless LAN adapter Wi-Fi:

   Connection-specific DNS Suffix  . : home
   Link-local IPv6 Address . . . . . : fe80::1%12
   IPv4 Address. . . . . . . . . . . : 192.168.1.20
   Subnet Mask . . . . . . . . . . . : 255.255.255.0
   Default Gateway . . . . . . . . . : 192.168.1.1
";

    #[test]
    fn ipconfig_ptbr() {
        let ifs = parse_ipconfig(IPCONFIG_PTBR);
        assert_eq!(
            ifs.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            ["Wi-Fi", "Ethernet 2"]
        );
        assert_eq!(ifs[0].ip, "192.168.0.132");
        assert_eq!(ifs[0].mask, "255.255.255.0");
        // IPv4 gateway on the line after the IPv6 one
        assert_eq!(ifs[0].gateway.as_deref(), Some("192.168.0.1"));
        assert_eq!(
            ifs[1],
            Iface {
                name: "Ethernet 2".into(),
                ip: "2.0.0.10".into(),
                mask: "255.0.0.0".into(),
                gateway: None
            }
        );
    }

    #[test]
    fn ipconfig_ptbr_with_mangled_accent() {
        // cp1252 console read as utf-8: the accent becomes U+FFFD; the parser must cope
        let txt = IPCONFIG_PTBR
            .replace("Mascara", "M\u{fffd}scara")
            .replace("Endereco", "Endere\u{fffd}o")
            .replace("Padrao", "Padr\u{fffd}o");
        let ifs = parse_ipconfig(&txt);
        assert_eq!(ifs.len(), 2);
        assert_eq!(ifs[0].mask, "255.255.255.0");
        assert_eq!(ifs[1].ip, "2.0.0.10");
    }

    #[test]
    fn ipconfig_en() {
        let ifs = parse_ipconfig(IPCONFIG_EN);
        assert_eq!(
            ifs[0],
            Iface {
                name: "Ethernet 2".into(),
                ip: "10.0.0.5".into(),
                mask: "255.0.0.0".into(),
                gateway: None
            }
        );
        assert_eq!(ifs[1].name, "Wi-Fi");
        assert_eq!(ifs[1].gateway.as_deref(), Some("192.168.1.1"));
    }

    #[test]
    fn ip_addr_linux() {
        let txt = "1: lo: <LOOPBACK>\n    inet 127.0.0.1/8 scope host lo\n\
                   2: eth0: <UP>\n    inet 10.1.2.3/24 brd 10.1.2.255\n";
        let ifs = parse_ip_addr(txt, "default via 10.1.2.1 dev eth0\n");
        assert_eq!(
            ifs[1],
            Iface {
                name: "eth0".into(),
                ip: "10.1.2.3".into(),
                mask: "255.255.255.0".into(),
                gateway: Some("10.1.2.1".into())
            }
        );
        assert_eq!(ifs[0].name, "lo");
    }

    fn iface(name: &str, ip: &str, mask: &str, gw: Option<&str>) -> Iface {
        Iface {
            name: name.into(),
            ip: ip.into(),
            mask: mask.into(),
            gateway: gw.map(String::from),
        }
    }

    #[test]
    fn suggest_rules() {
        let ifs = [
            iface(
                "Wi-Fi",
                "192.168.0.10",
                "255.255.255.0",
                Some("192.168.0.1"),
            ),
            iface("Ethernet", "192.168.0.7", "255.255.255.0", None),
            iface("Laser", "2.0.0.10", "255.0.0.0", None),
        ];
        let txt = suggest_with(&ifs, true).join("\n");
        assert!(
            txt.contains("Laser 2.0.0.10/255.0.0.0: Art-Net ok"),
            "{txt}"
        );
        assert!(
            txt.contains("Wi-Fi 192.168.0.10/255.255.255.0: sACN ok; Art-Net prefers"),
            "{txt}"
        );
        assert!(
            txt.contains("netsh interface ip set address name=\"Wi-Fi\" static 2.0.0.10 255.0.0.0"),
            "{txt}"
        );
        assert!(
            txt.contains("Wi-Fi, Ethernet on the same subnet 192.168.0.0"),
            "{txt}"
        );
        assert!(!suggest_with(&ifs, false).join("\n").contains("netsh"));
        assert_eq!(suggest_with(&[], true).len(), 1);
        assert!(txt.is_ascii());
    }

    #[test]
    fn masks_and_broadcast() {
        assert_eq!(mask_from_prefix(24), "255.255.255.0");
        assert_eq!(mask_from_prefix(8), "255.0.0.0");
        assert_eq!(net_of("192.168.0.132", "255.255.255.0"), "192.168.0.0");
        assert_eq!(bcast_of("2.0.0.10", "255.0.0.0"), "2.255.255.255");
    }

    fn artpollreply() -> Vec<u8> {
        let mut p = vec![0u8; 239];
        p[..8].copy_from_slice(b"Art-Net\0");
        p[8..10].copy_from_slice(&0x2100u16.to_le_bytes());
        p[10..14].copy_from_slice(&[2, 0, 0, 50]);
        p[14..16].copy_from_slice(&6454u16.to_le_bytes());
        p[18] = 0;
        p[19] = 1;
        p[26..31].copy_from_slice(b"Node1");
        p[44..53].copy_from_slice(b"Test node");
        p[172..174].copy_from_slice(&2u16.to_be_bytes());
        p[174] = 0x80 | 0x40; // port 0: out and in
        p[175] = 0x80; // port 1: out only
        p[186] = 5;
        p[190] = 2;
        p[191] = 3;
        p[201..207].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        p
    }

    #[test]
    fn artpollreply_parse() {
        let r = parse_artpollreply(&artpollreply()).expect("parse");
        assert_eq!(r.ip, "2.0.0.50");
        assert_eq!(r.short_name, "Node1");
        assert_eq!(r.long_name, "Test node");
        assert_eq!(r.mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert_eq!(
            r.ports,
            vec![
                Port {
                    dir: "out".into(),
                    universe: 0x12
                },
                Port {
                    dir: "in".into(),
                    universe: 0x15
                },
                Port {
                    dir: "out".into(),
                    universe: 0x13
                },
            ]
        );
        assert!(parse_artpollreply(&crate::artnet::artpoll(0, 0)).is_none());
    }

    #[test]
    fn sacn_discovery_parse() {
        let universes = [1u16, 2, 10];
        let mut p = vec![0u8; 120];
        p[..2].copy_from_slice(&0x0010u16.to_be_bytes());
        p[4..16].copy_from_slice(b"ASC-E1.17\0\0\0");
        p[18..22].copy_from_slice(&8u32.to_be_bytes());
        for (i, b) in p[22..38].iter_mut().enumerate() {
            *b = i as u8;
        }
        p[40..44].copy_from_slice(&2u32.to_be_bytes());
        p[44..52].copy_from_slice(b"Source X");
        p[112..114].copy_from_slice(&(0x7000u16 | (8 + 2 * universes.len() as u16)).to_be_bytes());
        p[114..118].copy_from_slice(&1u32.to_be_bytes());
        for u in universes {
            p.extend_from_slice(&u.to_be_bytes());
        }
        let r = parse_sacn_discovery(&p).expect("parse");
        assert_eq!(r.source_name, "Source X");
        assert_eq!(r.universes, universes);
        assert!(parse_sacn_discovery(&[b'x'; 200]).is_none());
    }

    #[test]
    fn report_ascii() {
        let s = Scan {
            interfaces: vec![iface(
                "Wi-Fi",
                "192.168.0.132",
                "255.255.255.0",
                Some("192.168.0.1"),
            )],
            suggestions: vec!["x".into()],
            artnet: vec![parse_artpollreply(&artpollreply()).unwrap()],
            sacn: Vec::new(),
            etherdream: Vec::new(),
        };
        let txt = report(&s);
        assert!(txt.is_ascii());
        assert!(txt.contains("2.0.0.50  'Node1'"), "{txt}");
        assert!(txt.contains("(nothing found)"), "{txt}");
        assert!(serde_json::to_string(&s)
            .unwrap()
            .contains("\"interfaces\""));
    }

    // `arp -a` on this machine, read from the cp1252 console (the accent becomes U+FFFD).
    const ARP_PTBR: &str = "\
Interface: 169.254.86.236 --- 0xc
  Endere\u{fffd}o IP           Endere\u{fffd}o f\u{fffd}sico       Tipo
  169.254.207.140       8a-9e-36-98-8c-ce     din\u{fffd}mico
  169.254.255.255       ff-ff-ff-ff-ff-ff     est\u{fffd}tico
  224.0.0.251           01-00-5e-00-00-fb     est\u{fffd}tico
  255.255.255.255       ff-ff-ff-ff-ff-ff     est\u{fffd}tico
";

    const ARP_EN: &str = "\
Interface: 192.168.0.132 --- 0xe
  Internet Address      Physical Address      Type
  192.168.0.1           74-3a-ef-76-d2-66     dynamic
  192.168.0.255         ff-ff-ff-ff-ff-ff     static
  239.255.255.250       01-00-5e-7f-ff-fa     static
";

    #[test]
    fn arp_ptbr_english_and_linux() {
        // the column changes name with the language; the key is the physical address, not the
        // header
        assert_eq!(
            parse_arp(ARP_PTBR),
            [(
                "169.254.207.140".to_string(),
                "8a:9e:36:98:8c:ce".to_string()
            )]
        );
        assert_eq!(
            parse_arp(ARP_EN),
            [("192.168.0.1".to_string(), "74:3a:ef:76:d2:66".to_string())]
        );
        // Linux `ip neigh`, same function
        let n = parse_arp(
            "169.254.207.140 dev eth0 lladdr 8a:9e:36:98:8c:ce REACHABLE\n\
             10.0.0.9 dev eth0  FAILED\n",
        );
        assert_eq!(n.len(), 1, "{n:?}");
        assert_eq!(n[0].0, "169.254.207.140");
        assert!(parse_arp("").is_empty());
    }

    #[test]
    fn interfaces_does_not_hang() {
        // with no active interface the list comes back empty; what it must not do is panic or
        // hang
        let _ = interfaces();
    }
}
