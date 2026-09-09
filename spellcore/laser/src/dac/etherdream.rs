//! Ether Dream DAC: beacon UDP 7654 (1 Hz), stream de pontos TCP 7765. Tudo little-endian.
//!
//! Fluxo: connect -> `p` prepare -> `d` data -> `b` begin -> `d` enquanto o buffer estiver
//! abaixo da capacidade. Porte de `spellcaster/protocols/ilda/etherdream.py`.
//!
//! O beacon (descoberta) ja existe em `protocols::netscan`; aqui so se reexporta.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::dac::Dac;
use crate::frame::Point;

pub use protocols::netscan::{parse_beacon, parse_status, Status};

pub const TCP_PORT: u16 = 7765;
/// ack + comando ecoado + dac_status.
pub const RESP_LEN: usize = 22;
/// `<HhhHHHHHH`: control x y r g b i u1 u2.
pub const POINT_LEN: usize = 18;
/// Teto de espera pelo buffer do DAC antes de desistir. O Python esperava para sempre.
// ponytail: teto fixo de 2 s ; um DAC que nao drena em 2 s esta morto e a thread do Feed
// tem que voltar a viver para o show poder parar. Vira parametro se aparecer DAC lento.
const WAIT_LIMIT: Duration = Duration::from_secs(2);

fn pack_status(s: &Status, out: &mut Vec<u8>) {
    out.extend_from_slice(&[s.protocol, s.light_engine_state, s.playback_state, s.source]);
    out.extend_from_slice(&s.light_engine_flags.to_le_bytes());
    out.extend_from_slice(&s.playback_flags.to_le_bytes());
    out.extend_from_slice(&s.source_flags.to_le_bytes());
    out.extend_from_slice(&s.buffer_fullness.to_le_bytes());
    out.extend_from_slice(&s.point_rate.to_le_bytes());
    out.extend_from_slice(&s.point_count.to_le_bytes());
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    /// `a` ok, `F` cheio, `I` invalido, `!` stop.
    pub ack: u8,
    pub status: Status,
}

pub fn parse_response(b: &[u8]) -> Option<Response> {
    if b.len() < RESP_LEN {
        return None;
    }
    Some(Response { ack: b[0], status: parse_status(&b[2..])? })
}

/// Comando `d`: cabecalho + um registro de 18 bytes por ponto. Cores 0-255 viram 0-65535
/// (x257); ponto apagado sai preto. Reusa `out`, nao aloca depois do primeiro frame.
pub fn encode_data(points: &[Point], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(3 + points.len() * POINT_LEN);
    out.push(b'd');
    out.extend_from_slice(&(points.len() as u16).to_le_bytes());
    for p in points {
        let (r, g, b) = if p.blank {
            (0u16, 0u16, 0u16)
        } else {
            (p.r as u16 * 257, p.g as u16 * 257, p.b as u16 * 257)
        };
        out.extend_from_slice(&0u16.to_le_bytes()); // control
        out.extend_from_slice(&p.x.to_le_bytes());
        out.extend_from_slice(&p.y.to_le_bytes());
        for v in [r, g, b, r.max(g).max(b), 0, 0] {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
}

fn begin_cmd(low_water: u16, pps: u32) -> [u8; 7] {
    let mut c = [0u8; 7];
    c[0] = b'b';
    c[1..3].copy_from_slice(&low_water.to_le_bytes());
    c[3..7].copy_from_slice(&pps.to_le_bytes());
    c
}

pub struct EtherDream {
    addr: String,
    sock: Option<TcpStream>,
    pub capacity: u16,
    pub status: Status,
    pps: u32,
    chunk: usize,
    chunk_fixo: Option<usize>,
    begun: bool,
    /// Instante do ultimo ack: base do modelo local do buffer (`est_fullness`).
    ack_at: Instant,
    buf: Vec<u8>,
}

impl EtherDream {
    /// `addr` = "ip" ou "ip:porta" (porta padrao 7765). O DAC manda um status ao conectar.
    pub fn connect(addr: &str, capacity: u16) -> io::Result<EtherDream> {
        let full = if addr.contains(':') { addr.to_string() } else { format!("{addr}:{TCP_PORT}") };
        let sa = full
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "endereco invalido"))?;
        let sock = TcpStream::connect_timeout(&sa, Duration::from_secs(2))?;
        sock.set_nodelay(true)?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut d = EtherDream {
            addr: full,
            sock: Some(sock),
            capacity,
            status: Status::default(),
            pps: 20_000,
            chunk: 400,
            chunk_fixo: None,
            begun: false,
            ack_at: Instant::now(),
            buf: Vec::new(),
        };
        d.read_resp()?; // status de boas-vindas
        Ok(d)
    }

    fn read_resp(&mut self) -> io::Result<Response> {
        let s = self
            .sock
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Ether Dream fechou"))?;
        let mut b = [0u8; RESP_LEN];
        s.read_exact(&mut b)?;
        let r = parse_response(&b)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "resposta curta"))?;
        self.status = r.status.clone();
        self.ack_at = Instant::now();
        Ok(r)
    }

    /// Envia bytes crus e le a resposta.
    pub fn cmd(&mut self, data: &[u8]) -> io::Result<Response> {
        let s = self
            .sock
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Ether Dream fechou"))?;
        s.write_all(data)?;
        self.read_resp()
    }

    pub fn prepare(&mut self) -> io::Result<Response> {
        self.cmd(b"p")
    }

    /// Pontos por comando `d`. Padrao: 2/3 da capacidade do DAC. E o botao de calibracao
    /// do DAC: bloco maior = menos idas e voltas na rede; bloco menor = menos latencia.
    pub fn set_chunk(&mut self, n: usize) {
        self.chunk_fixo = Some(n.max(1));
        self.chunk = n.max(1);
    }

    /// Quanto o buffer do DAC deve ter AGORA: o valor do ultimo ack menos o que o DAC
    /// consumiu desde entao. Evita um `?` (ping) por bloco so para reler o buffer_fullness:
    /// a 30 kpps com 4 feeds isso era 2/3 das idas e voltas na rede.
    // ponytail: modelo local do buffer, ressincronizado a cada ack e com o ack 'F' como rede
    // de seguranca ; trocar pelo low_water do proprio DAC quando houver hardware na bancada
    // para medir a deriva do relogio dele.
    fn est_fullness(&self) -> u32 {
        let f = self.status.buffer_fullness as u32;
        if self.status.playback_state != 2 {
            return f;
        }
        let rate = if self.status.point_rate == 0 { self.pps } else { self.status.point_rate };
        f.saturating_sub((self.ack_at.elapsed().as_secs_f64() * rate as f64) as u32)
    }

    fn data(&mut self, block: &[Point]) -> io::Result<Response> {
        let mut buf = std::mem::take(&mut self.buf);
        encode_data(block, &mut buf);
        let r = self.cmd(&buf);
        self.buf = buf; // devolve a capacidade: proximo envio nao aloca
        r
    }
}

impl Dac for EtherDream {
    fn name(&self) -> String {
        format!("etherdream:{}", self.addr)
    }

    fn begin(&mut self, pps: u32) -> io::Result<()> {
        self.pps = pps.max(1);
        // Bloco = 2/3 do buffer do DAC: e o maior comando que ainda deixa um terco do
        // buffer como folga contra underrun, e cada comando e uma ida e volta na rede.
        // ponytail: o Python usava pps/50 (~20 ms de pontos) sem olhar a capacidade ; a 30 kpps
        // isso dava 50 idas e voltas por segundo por DAC e ~1,5 % de um nucleo com 4 feeds
        // so de troca de contexto. Trocar por set_chunk() se um DAC real reclamar do bloco.
        self.chunk = self.chunk_fixo.unwrap_or(((self.capacity as usize * 2) / 3).max(1));
        if self.status.playback_state != 0 {
            self.cmd(b"s")?;
        }
        self.prepare()?;
        self.begun = false;
        Ok(())
    }

    fn send(&mut self, points: &[Point]) -> io::Result<()> {
        for block in points.chunks(self.chunk) {
            let t0 = Instant::now();
            loop {
                let falta = block.len() as i64 + self.est_fullness() as i64 - self.capacity as i64;
                if falta > 0 {
                    // dorme exatamente o tempo que o DAC leva para abrir espaco: nenhum ping
                    if t0.elapsed() > WAIT_LIMIT {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "buffer do DAC nao drena",
                        ));
                    }
                    std::thread::sleep(Duration::from_secs_f64(falta as f64 / self.pps as f64));
                    continue;
                }
                match self.data(block)?.ack {
                    b'a' => break,
                    b'F' => {
                        // o DAC estava mais cheio que a estimativa; o proprio ack acabou de
                        // ressincronizar o modelo, entao espera um bloco e repete
                        if t0.elapsed() > WAIT_LIMIT {
                            return Err(io::Error::new(
                                io::ErrorKind::TimedOut,
                                "buffer do DAC cheio",
                            ));
                        }
                        std::thread::sleep(Duration::from_secs_f64(
                            block.len() as f64 / self.pps as f64,
                        ));
                    }
                    a => {
                        return Err(io::Error::other(
                            format!("Ether Dream respondeu {:?} ao comando d", a as char),
                        ))
                    }
                }
            }
            if !self.begun || self.status.playback_state == 0 {
                // inicio ou underrun
                self.cmd(&begin_cmd(0, self.pps))?;
                self.begun = true;
            }
        }
        Ok(())
    }

    fn stop(&mut self) {
        if self.sock.is_some() {
            let _ = self.cmd(b"s");
            self.begun = false;
        }
    }
}

impl Drop for EtherDream {
    fn drop(&mut self) {
        self.stop();
        self.sock = None;
    }
}

// ------------------------------------------------------------------ emulador

/// Um ponto como o DAC recebeu (u1/u2 sempre 0, nao guardados).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmuPoint {
    pub control: u16,
    pub x: i16,
    pub y: i16,
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub i: u16,
}

struct EmuState {
    status: Status,
    t: Instant,
    points: Vec<EmuPoint>,
    commands: Vec<u8>,
}

struct Emu {
    capacity: u16,
    record: AtomicBool,
    run: AtomicBool,
    count: AtomicU64,
    st: Mutex<EmuState>,
}

/// Servidor TCP minimo que responde ack e simula o buffer; para testes e bench sem DAC.
pub struct Emulator {
    pub port: u16,
    emu: Arc<Emu>,
    handle: Option<JoinHandle<()>>,
}

impl Emulator {
    pub fn start(capacity: u16) -> io::Result<Emulator> {
        let lis = TcpListener::bind(("127.0.0.1", 0))?;
        let port = lis.local_addr()?.port();
        let status = Status { protocol: 1, ..Default::default() };
        let emu = Arc::new(Emu {
            capacity,
            record: AtomicBool::new(true),
            run: AtomicBool::new(true),
            count: AtomicU64::new(0),
            st: Mutex::new(EmuState {
                status,
                t: Instant::now(),
                points: Vec::new(),
                commands: Vec::new(),
            }),
        });
        let e = emu.clone();
        let handle = std::thread::spawn(move || serve(lis, e));
        Ok(Emulator { port, emu, handle: Some(handle) })
    }

    /// Guardar cada ponto recebido custa memoria e CPU: desligue no bench.
    pub fn record(&self, on: bool) {
        self.emu.record.store(on, Ordering::Relaxed);
    }

    pub fn points(&self) -> Vec<EmuPoint> {
        self.emu.st.lock().unwrap().points.clone()
    }

    /// Total de pontos recebidos (contado mesmo com `record(false)`).
    pub fn count(&self) -> u64 {
        self.emu.count.load(Ordering::Relaxed)
    }

    pub fn commands(&self) -> Vec<u8> {
        self.emu.st.lock().unwrap().commands.clone()
    }

    /// Beacon UDP de 36 bytes, para testar o parser sem hardware.
    pub fn beacon(&self, mac: [u8; 6]) -> Vec<u8> {
        let mut out = Vec::with_capacity(36);
        out.extend_from_slice(&mac);
        out.extend_from_slice(&1u16.to_le_bytes()); // hw_rev
        out.extend_from_slice(&2u16.to_le_bytes()); // sw_rev
        out.extend_from_slice(&self.emu.capacity.to_le_bytes());
        out.extend_from_slice(&100_000u32.to_le_bytes()); // max_point_rate
        pack_status(&self.emu.st.lock().unwrap().status, &mut out);
        out
    }

    pub fn stop(&mut self) {
        self.emu.run.store(false, Ordering::Relaxed);
        // acorda o accept bloqueado
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        self.stop();
    }
}

fn serve(lis: TcpListener, emu: Arc<Emu>) {
    while emu.run.load(Ordering::Relaxed) {
        let Ok((conn, _)) = lis.accept() else { return };
        if !emu.run.load(Ordering::Relaxed) {
            return;
        }
        handle(conn, &emu);
    }
}

/// Consome pontos pelo tempo decorrido quando tocando; buffer vazio = underrun -> idle.
fn drain(st: &mut EmuState) {
    let now = Instant::now();
    if st.status.playback_state == 2 {
        let gone = (now.duration_since(st.t).as_secs_f64() * st.status.point_rate as f64) as u32;
        st.status.buffer_fullness = st.status.buffer_fullness.saturating_sub(gone.min(65535) as u16);
        if st.status.buffer_fullness == 0 {
            st.status.playback_state = 0;
        }
    }
    st.t = now;
}

fn handle(conn: TcpStream, emu: &Arc<Emu>) {
    let _ = conn.set_nodelay(true);
    let _ = conn.set_read_timeout(Some(Duration::from_millis(250)));
    // um comando 'd' de 1200 pontos sao 21603 bytes: sem buffer isso viraria 3 syscalls
    // (1 + 2 + 21600) e 3 acordadas de thread por frame, que aparecem no CPU do feed.
    let mut rd = std::io::BufReader::with_capacity(64 * 1024, &conn);
    let mut resp = Vec::with_capacity(RESP_LEN);
    let mut raw: Vec<u8> = Vec::new();
    {
        let st = emu.st.lock().unwrap();
        resp.extend_from_slice(b"a?");
        pack_status(&st.status, &mut resp);
    }
    if (&conn).write_all(&resp).is_err() {
        return;
    }
    let mut one = [0u8; 1];
    while emu.run.load(Ordering::Relaxed) {
        match rd.read(&mut one) {
            Ok(0) => return,
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                continue // ocioso: so volta para conferir o run
            }
            Err(_) => return,
        }
        let c = one[0];
        let mut ack = b'a';
        // le o corpo do comando fora do lock
        let mut args = [0u8; 6];
        let mut npts = 0usize;
        match c {
            b'b' => {
                if rd.read_exact(&mut args).is_err() {
                    return;
                }
            }
            b'q' => {
                if rd.read_exact(&mut args[..4]).is_err() {
                    return;
                }
            }
            b'd' => {
                if rd.read_exact(&mut args[..2]).is_err() {
                    return;
                }
                npts = u16::from_le_bytes([args[0], args[1]]) as usize;
                raw.resize(npts * POINT_LEN, 0);
                if rd.read_exact(&mut raw).is_err() {
                    return;
                }
            }
            _ => {}
        }
        {
            let mut st = emu.st.lock().unwrap();
            drain(&mut st);
            match c {
                b'p' => st.status.playback_state = 1,
                b'b' => {
                    st.status.point_rate = u32::from_le_bytes([args[2], args[3], args[4], args[5]]);
                    st.status.playback_state = 2;
                }
                b'q' => {
                    st.status.point_rate = u32::from_le_bytes([args[0], args[1], args[2], args[3]])
                }
                b'd' => {
                    if st.status.buffer_fullness as usize + npts > emu.capacity as usize {
                        ack = b'F';
                    } else {
                        if emu.record.load(Ordering::Relaxed) {
                            for i in 0..npts {
                                let p = &raw[i * POINT_LEN..];
                                let le16 = |k: usize| u16::from_le_bytes([p[k], p[k + 1]]);
                                st.points.push(EmuPoint {
                                    control: le16(0),
                                    x: i16::from_le_bytes([p[2], p[3]]),
                                    y: i16::from_le_bytes([p[4], p[5]]),
                                    r: le16(6),
                                    g: le16(8),
                                    b: le16(10),
                                    i: le16(12),
                                });
                            }
                        }
                        emu.count.fetch_add(npts as u64, Ordering::Relaxed);
                        st.status.buffer_fullness += npts as u16;
                        st.status.point_count = st.status.point_count.wrapping_add(npts as u32);
                    }
                }
                b's' => {
                    st.status.playback_state = 0;
                    st.status.buffer_fullness = 0;
                }
                b'0' | 0 => {
                    st.status.playback_state = 0;
                    st.status.light_engine_state = 3; // e-stop
                }
                b'c' => st.status.light_engine_state = 0,
                b'?' => {}
                _ => ack = b'I',
            }
            st.commands.push(c);
            resp.clear();
            resp.push(ack);
            resp.push(c);
            pack_status(&st.status, &mut resp);
        }
        if (&conn).write_all(&resp).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;

    #[test]
    fn encode_data_igual_ao_python() {
        let mut buf = Vec::new();
        encode_data(
            &[
                Point::new(-1.0, 2.0, 255, 0, 128, false),
                Point::new(5.0, 6.0, 255, 255, 255, true),
            ],
            &mut buf,
        );
        assert_eq!(&buf[..3], &[b'd', 2, 0][..]);
        let mut esperado = Vec::new();
        for v in [0u16, 0, 0, 65535, 0, 32896, 65535, 0, 0] {
            esperado.extend_from_slice(&v.to_le_bytes());
        }
        esperado[2..4].copy_from_slice(&(-1i16).to_le_bytes());
        esperado[4..6].copy_from_slice(&2i16.to_le_bytes());
        assert_eq!(&buf[3..21], &esperado[..]);
        let mut zero = [0u8; POINT_LEN];
        zero[2..4].copy_from_slice(&5i16.to_le_bytes());
        zero[4..6].copy_from_slice(&6i16.to_le_bytes());
        assert_eq!(&buf[21..39], &zero[..]);
    }

    #[test]
    fn beacon_do_emulador_parseia() {
        let emu = Emulator::start(1800).unwrap();
        let b = emu.beacon([1, 2, 3, 4, 5, 6]);
        let (mac, hw, sw, cap, rate, st) = parse_beacon(&b).unwrap();
        assert_eq!(mac, "01:02:03:04:05:06");
        assert_eq!((hw, sw, cap, rate), (1, 2, 1800, 100_000));
        assert_eq!(st.playback_state, 0);
        assert_eq!(parse_status(&b[16..]).unwrap(), st);
    }

    #[test]
    fn loopback_com_emulador() {
        let emu = Emulator::start(1800).unwrap();
        {
            let mut dac = EtherDream::connect(&format!("127.0.0.1:{}", emu.port), 1800).unwrap();
            dac.begin(20_000).unwrap();
            let f = Frame::new(
                (0..100).map(|i| Point::new(i as f64 * 10.0, -(i as f64) * 10.0, 255, 0, 0, false)).collect(),
                "",
            );
            for _ in 0..6 {
                dac.send(&f.points).unwrap();
            }
            dac.stop();
        }
        assert_eq!(emu.count(), 600);
        let pts = emu.points();
        assert_eq!((pts[1].x, pts[1].y, pts[1].r, pts[1].g, pts[1].b), (10, -10, 65535, 0, 0));
        let cmds = emu.commands();
        assert_eq!(&cmds[..3], b"pdb".as_slice());
        assert_eq!(*cmds.last().unwrap(), b's');
        assert!(cmds.iter().all(|c| b"pdb?s".contains(c)));
    }
}
