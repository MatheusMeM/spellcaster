//! sACN (ANSI E1.31): saida multicast por interface + unicast localhost, entrada.
//! Bytes identicos a `spellcaster/protocols/sacn.py` (fixture tests/conformance/sacn_packet.bin).

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
pub const DISCOVERY_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 250, 214);

/// `>HH12s` 0x10, 0, "ASC-E1.17\0\0\0"
const ROOT: [u8; 16] = [
    0x00, 0x10, 0x00, 0x00, b'A', b'S', b'C', b'-', b'E', b'1', b'.', b'1', b'7', 0, 0, 0,
];

/// Grupo multicast do universo: 239.255.{u>>8}.{u&255}.
pub fn mcast(universe: u16) -> Ipv4Addr {
    Ipv4Addr::new(239, 255, (universe >> 8) as u8, (universe & 255) as u8)
}

/// Pacote E1.31 data (start code 0), escrito em `out` (que e limpo antes). Sem alocacao se
/// `out` ja tiver capacidade — e o caminho quente da thread de I/O.
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
    let dmp_len = 11 + n; // 10 do cabecalho DMP + start code + dados
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
    let name = &name[..name.len().min(63)]; // igual ao Python: corta em 63, preenche 64
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

/// Pacote E1.31 data. Funcao pura: mesmos bytes do gerador Python.
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
    Discovery {
        cid: [u8; 16],
        name: String,
        universes: Vec<u16>,
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

/// Pacote data (vector 4) ou discovery (vector 8); `None` se nao for E1.31.
pub fn parse(pk: &[u8]) -> Option<Packet> {
    if pk.len() < 48 || pk[..16] != ROOT {
        return None;
    }
    let vec = be32(pk, 18);
    let mut cid = [0u8; 16];
    cid.copy_from_slice(&pk[22..38]);
    let name = cstr(&pk[44..108]);
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
    if vec == 8 && pk.len() >= 120 && be32(pk, 40) == 2 {
        let n = (pk.len() - 120) / 2;
        let universes = (0..n).map(|i| be16(pk, 120 + 2 * i)).collect();
        return Some(Packet::Discovery {
            cid,
            name,
            universes,
        });
    }
    None
}

/// IPv4 locais (resolvendo o hostname, como o `getaddrinfo` do Python) + loopback.
/// Ordenado por texto, igual ao `sorted()` do Python: o primeiro e o que manda o unicast.
pub fn interfaces() -> Vec<Ipv4Addr> {
    use std::collections::BTreeSet;
    use std::net::ToSocketAddrs;
    let mut set: BTreeSet<String> = BTreeSet::new();
    set.insert("127.0.0.1".to_string());
    let host = std::process::Command::new("hostname")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|h| !h.is_empty());
    if let Some(h) = host {
        if let Ok(addrs) = (h.as_str(), 0u16).to_socket_addrs() {
            for a in addrs {
                if let std::net::SocketAddr::V4(v4) = a {
                    set.insert(v4.ip().to_string());
                }
            }
        }
    }
    if set.len() == 1 {
        // sem hostname resolvivel: cai para a varredura de interfaces do netscan
        for i in crate::netscan::interfaces() {
            set.insert(i.ip);
        }
    }
    set.iter().filter_map(|s| s.parse().ok()).collect()
}

fn out_socket(ip: Ipv4Addr) -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_multicast_ttl_v4(1)?;
    s.set_multicast_if_v4(&ip)?;
    Ok(s.into())
}

/// Socket de escuta em 0.0.0.0:PORT com SO_REUSEADDR, inscrito nos grupos pedidos.
fn listener(groups: &[Ipv4Addr]) -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_reuse_address(true)?;
    s.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT).into())?;
    for g in groups {
        // ponytail: falha de IP_ADD_MEMBERSHIP e ignorada ; maquina sem placa ativa ainda
        // recebe o unicast em 127.0.0.1, que e o que os testes usam.
        let _ = s.join_multicast_v4(g, &Ipv4Addr::UNSPECIFIED);
    }
    s.set_read_timeout(Some(Duration::from_millis(200)))?;
    Ok(s.into())
}

// ------------------------------------------------------------------ SacnOut

/// Saida sACN. `send()` so enfileira; a thread interna monta o pacote e fala com os sockets.
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
                // interface sumiu entre o scan e o bind: as outras seguem
                Err(_) if ips.len() > 1 => {}
                Err(e) => return Err(e),
            }
        }
        if socks.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "nenhuma interface utilizavel para sACN",
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
                // ponytail: um unico buffer de pacote reutilizado (a thread envia um frame por vez)
                // ; um buffer por universo so faria falta se o envio virasse paralelo.
                let mut buf = Vec::with_capacity(638);
                let local = SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT);
                while let Some((u, data)) = qt.pop() {
                    let s = seq.entry(u).or_insert(0);
                    *s = s.wrapping_add(1); // incrementa antes do envio, & 0xFF
                    packet_into(&mut buf, u, &data, &cid, *s, &name, priority);
                    let dst = SocketAddrV4::new(mcast(u), PORT);
                    for sk in &socks {
                        let _ = sk.send_to(&buf, dst); // sem rota para o grupo: ignora
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

/// Escuta os universos e guarda o ultimo frame de cada um.
pub struct SacnIn {
    last: Arc<Mutex<HashMap<u16, [u8; 512]>>>,
    sources: Arc<Mutex<HashMap<u16, String>>>,
    run: Arc<AtomicBool>,
    th: Option<JoinHandle<()>>,
}

impl SacnIn {
    pub fn new(universes: &[u16]) -> io::Result<SacnIn> {
        let groups: Vec<Ipv4Addr> = universes.iter().map(|u| mcast(*u)).collect();
        let sock = listener(&groups)?;
        let last = Arc::new(Mutex::new(HashMap::new()));
        let sources = Arc::new(Mutex::new(HashMap::new()));
        let run = Arc::new(AtomicBool::new(true));
        let (l, s, r) = (last.clone(), sources.clone(), run.clone());
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
                    if let Some(Packet::Data {
                        universe,
                        data,
                        name,
                        ..
                    }) = parse(&buf[..n])
                    {
                        let mut frame = [0u8; 512];
                        let k = data.len().min(512);
                        frame[..k].copy_from_slice(&data[..k]);
                        l.lock().unwrap_or_else(|e| e.into_inner()).insert(universe, frame);
                        s.lock().unwrap_or_else(|e| e.into_inner()).insert(universe, name);
                    }
                }
            })?;
        Ok(SacnIn {
            last,
            sources,
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

    /// Nome da ultima fonte vista no universo.
    pub fn source(&self, universe: u16) -> Option<String> {
        self.sources
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&universe)
            .cloned()
    }

    pub fn close(&mut self) {
        self.run.store(false, Ordering::Relaxed);
        if let Some(th) = self.th.take() {
            let _ = th.join(); // sai em ate 200 ms (read timeout)
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
    fn packet_bate_com_o_fixture() {
        let want = std::fs::read(FIXTURE).expect("fixture sacn_packet.bin");
        let cid: [u8; 16] = std::array::from_fn(|i| i as u8);
        let got = packet(1, &data512(), &cid, 0, "Spellcaster", 100);
        assert_eq!(got.len(), want.len(), "tamanho do pacote");
        assert_eq!(got, want, "bytes do pacote E1.31");
    }

    /// Cabecalho (126 bytes) gerado pelo Python para universo 300, seq 42, prio 77,
    /// nome "Fonte X", CID `i ^ 0x5A`: trava os campos que o fixture (defaults) nao cobre.
    const HDR_300: &str = "001000004153432d45312e3137000000726e000000045a5b58595e5f5c5d5253505156575455725800000002466f6e746520580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004d00002a00012c720b02a100000001020100";

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn cabecalho_com_valores_nao_padrao() {
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
        match parse(&pk).expect("parse") {
            Packet::Data {
                cid: c,
                name,
                priority,
                seq,
                universe,
                data,
            } => {
                assert_eq!(c, cid);
                assert_eq!(name, "Fonte X");
                assert_eq!(priority, 77);
                assert_eq!(seq, 42);
                assert_eq!(universe, 300);
                assert_eq!(data, d);
            }
            _ => panic!("esperava Data"),
        }
        assert!(parse(b"nao e e1.31").is_none());
    }

    #[test]
    fn mcast_do_universo() {
        assert_eq!(mcast(1), Ipv4Addr::new(239, 255, 0, 1));
        assert_eq!(mcast(256), Ipv4Addr::new(239, 255, 1, 0));
        assert_eq!(mcast(300), Ipv4Addr::new(239, 255, 1, 44));
    }

    #[test]
    fn interfaces_tem_loopback() {
        assert!(interfaces().contains(&Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn loopback_out_in() {
        let mut rx = match SacnIn::new(&[1]) {
            Ok(r) => r,
            Err(e) => return println!("pulado: bind 5568 falhou: {:?}", e.kind()),
        };
        let mut tx = match SacnOut::new(&[1], Some(vec![Ipv4Addr::LOCALHOST])) {
            Ok(t) => t,
            Err(e) => return println!("pulado: socket de saida falhou: {:?}", e.kind()),
        };
        let mut ultimo = [0u8; 512];
        for f in 1..=3u8 {
            let frame: [u8; 512] = std::array::from_fn(|i| (i as u8).wrapping_add(f));
            tx.send(1, &frame);
            ultimo = frame;
            std::thread::sleep(Duration::from_millis(30)); // frames distintos, sem descarte
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
            None => println!("pulado: UDP em loopback nao entregou (firewall?)"),
            Some(g) => {
                assert_eq!(g, ultimo, "ultimo frame recebido byte a byte");
                assert_eq!(rx.source(1).as_deref(), Some("Spellcaster"));
            }
        }
    }

    #[test]
    fn send_nao_bloqueia_nem_cresce() {
        // sem consumidor real: 100 frames de uma vez, a fila fica em 2 por universo
        let q = crate::Queue::new(1);
        let t0 = Instant::now();
        for i in 0..100u16 {
            let mut d = [0u8; 512];
            d[0] = i as u8;
            q.push(1, &d);
        }
        assert!(t0.elapsed() < Duration::from_millis(200), "send bloqueou");
        assert_eq!(q.len(), crate::DEPTH);
        q.stop();
    }

    #[test]
    fn saida_nao_bloqueia_com_100_frames() {
        let mut tx = match SacnOut::new(&[1], Some(vec![Ipv4Addr::LOCALHOST])) {
            Ok(t) => t,
            Err(e) => return println!("pulado: socket de saida falhou: {:?}", e.kind()),
        };
        let frame = [7u8; 512];
        let t0 = Instant::now();
        for _ in 0..100 {
            tx.send(1, &frame);
        }
        let dt = t0.elapsed();
        tx.close();
        assert!(dt < Duration::from_millis(500), "100 sends levaram {dt:?}");
    }
}
