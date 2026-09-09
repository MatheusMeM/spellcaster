//! IDN-Stream (ILDA Digital Network) sobre UDP 7255.
//!
//! Implementado com confianca: o cabecalho IDN-Hello (4 bytes), ping e o par
//! scan request / scan response — e a descoberta que a GUI e o `spell net` precisam.
//!
//! O envio de frames (`IDNCMD_MESSAGE` com um canal LaserProjector) esta escrito abaixo
//! seguindo a spec publica, mas NAO foi conferido contra hardware.
//
// ponytail: hello/ping/scan verificados, canal LPGRF montado da spec publica e nao testado
// em projetor real ; conferir os descritores de amostra e o cabecalho de chunk com um
// Laserworld/Showtacle na bancada antes de anunciar suporte a IDN.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

use crate::dac::Dac;
use crate::frame::Point;

pub const PORT: u16 = 7255;

// --- comandos IDN-Hello ---
pub const CMD_VOID: u8 = 0x00;
pub const CMD_PING_REQUEST: u8 = 0x08;
pub const CMD_PING_RESPONSE: u8 = 0x09;
pub const CMD_SCAN_REQUEST: u8 = 0x10;
pub const CMD_SCAN_RESPONSE: u8 = 0x11;
pub const CMD_SERVICEMAP_REQUEST: u8 = 0x12;
pub const CMD_SERVICEMAP_RESPONSE: u8 = 0x13;
pub const CMD_MESSAGE: u8 = 0x40;
pub const CMD_MESSAGE_ACKREQ: u8 = 0x41;
pub const CMD_MESSAGE_CLOSE: u8 = 0x44;

// --- contentID da mensagem de canal ---
const CID_CONFIG: u16 = 0x8000; // cabecalho de configuracao de canal presente
const CID_CHANNELMSG: u16 = 0x4000;
const CNK_LPGRF_FRAME: u16 = 0x02; // frame de laser (grafico discreto)

/// serviceMode 1 = "laser projector, graphic discrete".
const SERVICE_MODE_LPGRF: u8 = 0x01;

// Descritores de amostra: os 12 bits baixos das cores sao o comprimento de onda em nm.
pub const SMP_X: u16 = 0x4200;
pub const SMP_Y: u16 = 0x4210;
pub const SMP_R: u16 = 0x527E; // 638 nm
pub const SMP_G: u16 = 0x5214; // 532 nm
pub const SMP_B: u16 = 0x51CC; // 460 nm

/// Cabecalho IDN-Hello: comando, flags, sequencia (big-endian).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hello {
    pub command: u8,
    pub flags: u8,
    pub sequence: u16,
}

pub fn hello(command: u8, flags: u8, sequence: u16) -> [u8; 4] {
    let s = sequence.to_be_bytes();
    [command, flags, s[0], s[1]]
}

pub fn parse_hello(b: &[u8]) -> Option<Hello> {
    if b.len() < 4 {
        return None;
    }
    Some(Hello { command: b[0], flags: b[1], sequence: u16::from_be_bytes([b[2], b[3]]) })
}

/// Um servidor IDN visto por scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub ip: String,
    pub unit_id: [u8; 16],
    pub name: String,
    pub protocol_version: u8,
    pub status: u8,
}

/// Corpo do `IDNCMD_SCAN_RESPONSE`: structSize, protocolVersion, status, reservado,
/// unitID[16], hostName[20].
pub fn parse_scan_response(b: &[u8], ip: &str) -> Option<Unit> {
    if b.len() < 24 {
        return None;
    }
    let mut unit_id = [0u8; 16];
    unit_id.copy_from_slice(&b[4..20]);
    let raw = &b[20..b.len().min(40)];
    let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
    let name = raw[..end]
        .iter()
        .map(|&c| if c < 0x80 { c as char } else { char::REPLACEMENT_CHARACTER })
        .collect();
    Some(Unit { ip: ip.to_string(), unit_id, name, protocol_version: b[1], status: b[2] })
}

/// Manda scan request para o broadcast e junta as respostas ate estourar `timeout`.
pub fn scan(timeout: Duration) -> Vec<Unit> {
    scan_to(Ipv4Addr::BROADCAST, timeout)
}

pub fn scan_to(target: Ipv4Addr, timeout: Duration) -> Vec<Unit> {
    let Ok(sock) = UdpSocket::bind(("0.0.0.0", 0)) else {
        return Vec::new();
    };
    let _ = sock.set_broadcast(true);
    let _ = sock.set_read_timeout(Some(Duration::from_millis(200)));
    if sock.send_to(&hello(CMD_SCAN_REQUEST, 0, 1), SocketAddr::from((target, PORT))).is_err() {
        return Vec::new();
    }
    let mut found: Vec<Unit> = Vec::new();
    let mut buf = [0u8; 512];
    let end = Instant::now() + timeout;
    while Instant::now() < end {
        let Ok((n, from)) = sock.recv_from(&mut buf) else {
            continue;
        };
        let Some(h) = parse_hello(&buf[..n]) else { continue };
        if h.command != CMD_SCAN_RESPONSE {
            continue;
        }
        if let Some(u) = parse_scan_response(&buf[4..n], &from.ip().to_string()) {
            if !found.iter().any(|x| x.unit_id == u.unit_id) {
                found.push(u);
            }
        }
    }
    found
}

/// Monta a mensagem de canal com um frame LaserProjector.
///
/// `config` inclui o cabecalho de configuracao do canal (obrigatorio no primeiro frame e
/// sempre que o layout de amostra mudar); depois disso ele pode sair.
///
/// ```text
/// IDN-Hello           : cmd flags seq(u16 BE)
/// Channel message     : totalSize(u16 BE) contentID(u16 BE) timestamp(u32 BE, us)
/// [Channel config]    : wordCount flags serviceID serviceMode + wordCount*2 descritores u16 BE
/// Sample chunk        : flags reservado duration(u16 BE, us)
/// Amostras            : X(i16 BE) Y(i16 BE) R G B  = 7 bytes por ponto
/// ```
#[allow(clippy::too_many_arguments)] // ponytail: espelha o cabecalho IDN campo a campo ; struct se ganhar mais campos
pub fn encode_frame(
    points: &[Point],
    channel: u8,
    service_id: u8,
    seq: u16,
    timestamp_us: u32,
    duration_us: u16,
    config: bool,
    out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(24 + points.len() * 7);
    out.extend_from_slice(&hello(CMD_MESSAGE, 0, seq));
    let head = out.len();
    out.extend_from_slice(&[0, 0]); // totalSize, preenchido no fim
    let mut content = CID_CHANNELMSG | ((channel as u16 & 0x3F) << 8) | CNK_LPGRF_FRAME;
    if config {
        content |= CID_CONFIG;
    }
    out.extend_from_slice(&content.to_be_bytes());
    out.extend_from_slice(&timestamp_us.to_be_bytes());
    if config {
        // 5 descritores cabem em 3 palavras de 32 bits (a ultima meia palavra fica VOID)
        out.extend_from_slice(&[3, 0, service_id, SERVICE_MODE_LPGRF]);
        for d in [SMP_X, SMP_Y, SMP_R, SMP_G, SMP_B, 0] {
            out.extend_from_slice(&d.to_be_bytes());
        }
    }
    out.extend_from_slice(&[0, 0]); // sample chunk: flags, reservado
    out.extend_from_slice(&duration_us.to_be_bytes());
    for p in points {
        out.extend_from_slice(&p.x.to_be_bytes());
        out.extend_from_slice(&p.y.to_be_bytes());
        let (r, g, b) = if p.blank { (0, 0, 0) } else { (p.r, p.g, p.b) };
        out.extend_from_slice(&[r, g, b]);
    }
    let total = (out.len() - head) as u16;
    out[head..head + 2].copy_from_slice(&total.to_be_bytes());
}

/// Projetor IDN. UDP puro: sem handshake por frame, sem ack (o `Feed` regula o ritmo).
pub struct Idn {
    sock: UdpSocket,
    addr: SocketAddr,
    pub channel: u8,
    pub service_id: u8,
    seq: u16,
    pps: u32,
    /// Pontos por datagrama. 1400 bytes de payload / 7 = 200; fica em 180 por folga de MTU.
    chunk: usize,
    t0: Instant,
    config_sent: bool,
    buf: Vec<u8>,
}

impl Idn {
    pub fn connect(addr: &str, channel: u8) -> io::Result<Idn> {
        let full = if addr.contains(':') { addr.to_string() } else { format!("{addr}:{PORT}") };
        let sa = full
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "endereco invalido"))?;
        let bind = if sa.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
        let sock = UdpSocket::bind(bind)?;
        if let IpAddr::V4(v4) = sa.ip() {
            if v4.is_broadcast() {
                sock.set_broadcast(true)?;
            }
        }
        Ok(Idn {
            sock,
            addr: sa,
            channel,
            service_id: 0,
            seq: 0,
            pps: 30_000,
            chunk: 180,
            t0: Instant::now(),
            config_sent: false,
            buf: Vec::new(),
        })
    }

    /// `IDNCMD_PING_REQUEST`; devolve `true` se veio ping response dentro do timeout.
    pub fn ping(&mut self, timeout: Duration) -> io::Result<bool> {
        self.seq = self.seq.wrapping_add(1);
        self.sock.send_to(&hello(CMD_PING_REQUEST, 0, self.seq), self.addr)?;
        self.sock.set_read_timeout(Some(timeout))?;
        let mut b = [0u8; 64];
        match self.sock.recv_from(&mut b) {
            Ok((n, _)) => Ok(parse_hello(&b[..n]).map(|h| h.command) == Some(CMD_PING_RESPONSE)),
            Err(_) => Ok(false),
        }
    }
}

impl Dac for Idn {
    fn name(&self) -> String {
        format!("idn:{}", self.addr)
    }

    fn begin(&mut self, pps: u32) -> io::Result<()> {
        self.pps = pps.max(1);
        self.config_sent = false;
        self.t0 = Instant::now();
        Ok(())
    }

    fn send(&mut self, points: &[Point]) -> io::Result<()> {
        let mut buf = std::mem::take(&mut self.buf);
        let mut res = Ok(());
        for block in points.chunks(self.chunk) {
            self.seq = self.seq.wrapping_add(1);
            let ts = self.t0.elapsed().as_micros() as u32;
            let dur = (block.len() as f64 * 1e6 / self.pps as f64).min(65535.0) as u16;
            let first = !self.config_sent;
            encode_frame(
                block,
                self.channel,
                self.service_id,
                self.seq,
                ts,
                dur,
                first,
                &mut buf,
            );
            self.config_sent = true;
            if let Err(e) = self.sock.send_to(&buf, self.addr) {
                res = Err(e);
                break;
            }
        }
        self.buf = buf;
        res
    }

    fn stop(&mut self) {
        self.seq = self.seq.wrapping_add(1);
        let _ = self.sock.send_to(&hello(CMD_MESSAGE_CLOSE, 0, self.seq), self.addr);
        self.config_sent = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_ida_e_volta() {
        let h = hello(CMD_SCAN_REQUEST, 0, 0x1234);
        assert_eq!(h, [0x10, 0x00, 0x12, 0x34]);
        assert_eq!(
            parse_hello(&h),
            Some(Hello { command: CMD_SCAN_REQUEST, flags: 0, sequence: 0x1234 })
        );
        assert_eq!(parse_hello(&h[..3]), None);
    }

    #[test]
    fn scan_response_parseia() {
        let mut b = vec![0x28, 0x01, 0x00, 0x00];
        b.extend_from_slice(&[9u8; 16]);
        b.extend_from_slice(b"projetor\0\0\0\0\0\0\0\0\0\0\0\0");
        let u = parse_scan_response(&b, "192.168.0.9").unwrap();
        assert_eq!(u.name, "projetor");
        assert_eq!(u.unit_id, [9u8; 16]);
        assert_eq!((u.protocol_version, u.status), (1, 0));
        assert_eq!(parse_scan_response(&b[..10], "x"), None);
    }

    #[test]
    fn frame_tem_cabecalho_e_tamanho_certos() {
        let pts = [
            Point::new(1.0, -2.0, 10, 20, 30, false),
            Point::new(3.0, 4.0, 255, 255, 255, true),
        ];
        let mut out = Vec::new();
        encode_frame(&pts, 2, 7, 5, 1000, 66, true, &mut out);
        assert_eq!(&out[..4], &[CMD_MESSAGE, 0, 0, 5][..]);
        // 8 (msg) + 4 + 12 (config) + 4 (chunk) + 14 (2 pontos) = 42
        assert_eq!(u16::from_be_bytes([out[4], out[5]]) as usize, out.len() - 4);
        assert_eq!(out.len(), 4 + 42);
        let content = u16::from_be_bytes([out[6], out[7]]);
        assert_eq!(content, CID_CONFIG | CID_CHANNELMSG | (2 << 8) | CNK_LPGRF_FRAME);
        assert_eq!(out[12..16], [3, 0, 7, SERVICE_MODE_LPGRF]);
        assert_eq!(u16::from_be_bytes([out[16], out[17]]), SMP_X);
        // ponto apagado sai preto
        assert_eq!(&out[out.len() - 3..], &[0, 0, 0][..]);
        // sem config o frame encolhe os 16 bytes do cabecalho de configuracao
        let n = out.len();
        encode_frame(&pts, 2, 7, 6, 1000, 66, false, &mut out);
        assert_eq!(out.len(), n - 16);
    }
}
