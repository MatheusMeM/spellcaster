//! Art-Net 4 (Artistic Licence): ArtDmx out, ArtPoll/ArtPollReply, ArtSync.
//!
//! Universos no Spellcaster sao 1-based (como sACN). Conversao para Art-Net:
//!     port_address = universe - 1   (15 bits: net[7] | subnet[4] | universe[4])
//! Ex.: universe 1 -> port-address 0 (net 0, subnet 0, uni 0); universe 17 -> subnet 1, uni 0.
//! OpCode e little-endian; ProtVer, Length e o resto do cabecalho sao big-endian.

use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::thread::JoinHandle;

use socket2::{Domain, Protocol, Socket, Type};

use crate::{Output, Queue};

pub const PORT: u16 = 6454;
pub const HEADER: &[u8; 8] = b"Art-Net\0";
pub const PROT_VER: u16 = 14;
pub const OP_POLL: u16 = 0x2000;
pub const OP_POLL_REPLY: u16 = 0x2100;
pub const OP_DMX: u16 = 0x5000;
pub const OP_SYNC: u16 = 0x5200;
pub const BROADCASTS: [&str; 3] = ["2.255.255.255", "10.255.255.255", "255.255.255.255"];

/// Universo 1-based -> port-address de 15 bits.
///
/// Fora de faixa e erro (igual ao `ValueError` do Python): universo 0 ou > 0x8000 nao existe
/// em Art-Net. A assinatura do README (`-> u16`) e mantida; o erro vira panic.
// ponytail: panic em vez de Result para nao mudar o contrato do README ; a saida (`ArtNetOut`)
// filtra o universo antes de chamar, entao a thread de I/O nunca panica com show valido.
pub fn port_address(universe: u16) -> u16 {
    assert!(
        (1..=0x8000).contains(&universe),
        "universo fora de faixa: {universe}"
    );
    universe - 1
}

fn valid(universe: u16) -> bool {
    (1..=0x8000).contains(&universe)
}

/// ArtDmx escrito em `out` (limpo antes). Sem alocacao com `out` ja dimensionado.
/// `data`: 2..512 bytes, comprimento par (Art-Net exige).
pub fn artdmx_into(out: &mut Vec<u8>, universe: u16, data: &[u8], sequence: u8, physical: u8) {
    let pa = port_address(universe);
    let n = data.len().min(512);
    let odd = n % 2 == 1;
    let len = if n == 0 { 2 } else { n + odd as usize };
    out.clear();
    out.extend_from_slice(HEADER);
    out.extend_from_slice(&OP_DMX.to_le_bytes()); // OpCode little-endian
    out.extend_from_slice(&PROT_VER.to_be_bytes());
    out.push(sequence);
    out.push(physical);
    out.push((pa & 0xFF) as u8); // SubUni
    out.push((pa >> 8) as u8); // Net
    out.extend_from_slice(&(len as u16).to_be_bytes());
    out.extend_from_slice(&data[..n]);
    out.resize(18 + len, 0); // impar -> um zero no fim; vazio -> dois zeros
}

pub fn artdmx(universe: u16, data: &[u8], sequence: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(530);
    artdmx_into(&mut out, universe, data, sequence, 0);
    out
}

/// ArtPoll. O `netscan` manda `artpoll(0, 0)`; a saida manda TalkToMe 0x06, prioridade 0x10.
pub fn artpoll(flags: u8, priority: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity(14);
    v.extend_from_slice(HEADER);
    v.extend_from_slice(&OP_POLL.to_le_bytes());
    v.extend_from_slice(&PROT_VER.to_be_bytes());
    v.push(flags);
    v.push(priority);
    v
}

pub fn artsync() -> Vec<u8> {
    let mut v = Vec::with_capacity(14);
    v.extend_from_slice(HEADER);
    v.extend_from_slice(&OP_SYNC.to_le_bytes());
    v.extend_from_slice(&PROT_VER.to_be_bytes());
    v.push(0);
    v.push(0);
    v
}

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Dmx {
        universe: u16,
        port_address: u16,
        sequence: u8,
        physical: u8,
        data: Vec<u8>,
    },
    Poll {
        flags: u8,
    },
    Sync,
    Other(u16),
}

/// Decodifica ArtDmx / ArtPoll / ArtSync. Quem le ArtPollReply e' o `netscan`.
pub fn parse(pkt: &[u8]) -> Option<Packet> {
    if pkt.len() < 10 || &pkt[..8] != HEADER {
        return None;
    }
    let op = u16::from_le_bytes([pkt[8], pkt[9]]);
    if op == OP_DMX && pkt.len() >= 18 {
        let length = u16::from_be_bytes([pkt[16], pkt[17]]) as usize;
        let pa = ((pkt[15] as u16) << 8) | pkt[14] as u16;
        let end = (18 + length).min(pkt.len());
        return Some(Packet::Dmx {
            universe: pa + 1,
            port_address: pa,
            sequence: pkt[12],
            physical: pkt[13],
            data: pkt[18..end].to_vec(),
        });
    }
    if op == OP_POLL {
        return Some(Packet::Poll {
            flags: if pkt.len() > 12 { pkt[12] } else { 0 },
        });
    }
    if op == OP_SYNC {
        return Some(Packet::Sync);
    }
    Some(Packet::Other(op))
}

fn bcast_socket() -> io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_broadcast(true)?;
    Ok(s.into())
}

fn resolve(t: &str, port: u16) -> Option<SocketAddrV4> {
    if let Ok(ip) = t.parse::<Ipv4Addr>() {
        return Some(SocketAddrV4::new(ip, port));
    }
    (t, port).to_socket_addrs().ok()?.find_map(|a| match a {
        std::net::SocketAddr::V4(v) => Some(v),
        _ => None,
    })
}

// ----------------------------------------------------------------- ArtNetOut

/// Saida Art-Net por broadcast (2.x, 10.x, limited) ou unicast para `targets`.
/// `send()` so enfileira; a thread interna monta o ArtDmx e fala com o socket.
pub struct ArtNetOut {
    q: Arc<Queue>,
    th: Option<JoinHandle<()>>,
}

impl ArtNetOut {
    pub fn new(targets: Option<Vec<String>>, broadcast: bool) -> io::Result<ArtNetOut> {
        ArtNetOut::with_port(targets, broadcast, PORT)
    }

    /// Porta alternativa (testes de loopback quando a 6454 esta ocupada).
    pub fn with_port(
        targets: Option<Vec<String>>,
        broadcast: bool,
        port: u16,
    ) -> io::Result<ArtNetOut> {
        let list: Vec<String> = match targets {
            Some(t) if !t.is_empty() => t,
            _ if broadcast => BROADCASTS.iter().map(|s| s.to_string()).collect(),
            _ => Vec::new(),
        };
        let tt: Vec<SocketAddrV4> = list.iter().filter_map(|t| resolve(t, port)).collect();
        let sock = bcast_socket()?;
        let q = Queue::new(8);
        let qt = q.clone();
        let th = std::thread::Builder::new()
            .name("artnet-out".into())
            .spawn(move || {
                // ponytail: um buffer de ArtDmx reutilizado (um frame por vez na thread).
                let mut buf = Vec::with_capacity(530);
                let mut seq: HashMap<u16, u8> = HashMap::new();
                while let Some((u, data)) = qt.pop() {
                    if !valid(u) {
                        continue; // universo invalido: descarta em vez de derrubar a thread
                    }
                    let s = seq.entry(u).or_insert(0);
                    *s = *s % 255 + 1; // 1..255, 0 = sem sequencia
                    artdmx_into(&mut buf, u, &data, *s, 0);
                    for t in &tt {
                        let _ = sock.send_to(&buf, t); // rede sem rota para 2.x/10.x: ignora
                    }
                }
            })?;
        Ok(ArtNetOut { q, th: Some(th) })
    }
}

impl Output for ArtNetOut {
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

impl Drop for ArtNetOut {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/conformance/artnet_packet.bin"
    );

    fn data512() -> Vec<u8> {
        (0..512u16).map(|i| (i % 256) as u8).collect()
    }

    #[test]
    fn artdmx_bate_com_o_fixture() {
        let want = std::fs::read(FIXTURE).expect("fixture artnet_packet.bin");
        let got = artdmx(1, &data512(), 0);
        assert_eq!(got.len(), want.len(), "tamanho do ArtDmx");
        assert_eq!(got, want, "bytes do ArtDmx");
    }

    /// Cabecalho (18 bytes) gerado pelo Python para universo 17, seq 200:
    /// SubUni 0x10, Net 0x00, Length 0x0200. Trava o par little/big-endian.
    #[test]
    fn cabecalho_universo_17() {
        let p = artdmx(17, &data512(), 200);
        assert_eq!(p.len(), 530);
        assert_eq!(
            &p[..18],
            &[
                0x41, 0x72, 0x74, 0x2d, 0x4e, 0x65, 0x74, 0x00, 0x00, 0x50, 0x00, 0x0e, 0xc8, 0x00,
                0x10, 0x00, 0x02, 0x00
            ]
        );
    }

    #[test]
    fn round_trip() {
        let d = data512();
        match parse(&artdmx(17, &d, 200)).expect("parse") {
            Packet::Dmx {
                universe,
                port_address,
                sequence,
                physical,
                data,
            } => {
                assert_eq!(universe, 17);
                assert_eq!(port_address, 16);
                assert_eq!(sequence, 200);
                assert_eq!(physical, 0);
                assert_eq!(data, d);
            }
            _ => panic!("esperava ArtDmx"),
        }
        assert_eq!(parse(&artsync()), Some(Packet::Sync));
        assert_eq!(
            parse(&artpoll(0x06, 0x10)),
            Some(Packet::Poll { flags: 0x06 })
        );
        assert!(parse(b"nao e art-net").is_none());
    }

    #[test]
    fn dados_impares_e_vazios() {
        let p = artdmx(1, &[1, 2, 3], 1);
        assert_eq!(u16::from_be_bytes([p[16], p[17]]), 4);
        assert_eq!(&p[18..], &[1, 2, 3, 0]);
        let p = artdmx(1, &[], 1);
        assert_eq!(u16::from_be_bytes([p[16], p[17]]), 2);
        assert_eq!(&p[18..], &[0, 0]);
    }

    #[test]
    fn port_address_1_based() {
        assert_eq!(port_address(1), 0);
        let pa = port_address(17);
        assert_eq!(((pa >> 4) & 0xF, pa & 0xF), (1, 0)); // subnet 1, uni 0
        assert_eq!(port_address(0x8000), 0x7FFF);
    }

    #[test]
    #[should_panic(expected = "universo fora de faixa")]
    fn port_address_zero_e_erro() {
        port_address(0);
    }

    #[test]
    #[should_panic(expected = "universo fora de faixa")]
    fn port_address_acima_da_faixa_e_erro() {
        port_address(0x8001);
    }

    #[test]
    fn send_nao_bloqueia() {
        let mut tx = match ArtNetOut::with_port(Some(vec!["127.0.0.1".into()]), false, 6455) {
            Ok(t) => t,
            Err(e) => return println!("pulado: socket de saida falhou: {:?}", e.kind()),
        };
        let frame = [3u8; 512];
        let t0 = Instant::now();
        for _ in 0..100 {
            tx.send(1, &frame);
        }
        let dt = t0.elapsed();
        tx.close();
        assert!(dt < Duration::from_millis(500), "100 sends levaram {dt:?}");
    }
}
