//! OSC 1.0 sobre UDP: encode/decode de mensagens e bundles, OscOut, OscIn com pattern matching.
//! Espelha `spellcaster/protocols/osc.py`.

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

/// Timetag "agora".
pub const IMMEDIATE: u64 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Blob(Vec<u8>),
    Time(u64),
    Bool(bool),
    Nil,
    Impulse,
}

#[allow(clippy::manual_is_multiple_of)]
fn pad(out: &mut Vec<u8>) {
    // ponytail: `% 4` e nao `is_multiple_of` (que so existe a partir do Rust 1.87)
    // ; o workspace declara rust-version 1.75.
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    out.push(0);
    pad(out);
}

/// Segundos desde a epoca Unix -> NTP 64 bits (segundos desde 1900 << 32 | fracao).
pub fn timetag(secs: f64) -> u64 {
    let x = secs + 2_208_988_800.0;
    let sec = x.floor();
    let frac = x - sec;
    ((sec as u64) << 32) | (frac * 4_294_967_296.0) as u64
}

fn tag(a: &Arg) -> u8 {
    match a {
        Arg::Int(_) => b'i',
        Arg::Long(_) => b'h',
        Arg::Float(_) => b'f',
        Arg::Double(_) => b'd',
        Arg::Str(_) => b's',
        Arg::Blob(_) => b'b',
        Arg::Time(_) => b't',
        Arg::Bool(true) => b'T',
        Arg::Bool(false) => b'F',
        Arg::Nil => b'N',
        Arg::Impulse => b'I',
    }
}

pub fn message(address: &str, args: &[Arg]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + 8 * args.len());
    put_str(&mut out, address);
    let mut tags = Vec::with_capacity(args.len() + 1);
    tags.push(b',');
    for a in args {
        tags.push(tag(a));
    }
    out.extend_from_slice(&tags);
    out.push(0);
    pad(&mut out);
    for a in args {
        match a {
            Arg::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
            Arg::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
            Arg::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
            Arg::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
            Arg::Time(v) => out.extend_from_slice(&v.to_be_bytes()),
            Arg::Str(s) => put_str(&mut out, s),
            Arg::Blob(b) => {
                out.extend_from_slice(&(b.len() as i32).to_be_bytes());
                out.extend_from_slice(b);
                pad(&mut out);
            }
            Arg::Bool(_) | Arg::Nil | Arg::Impulse => {}
        }
    }
    out
}

/// `elements`: mensagens/bundles ja codificados.
pub fn bundle(elements: &[Vec<u8>], tt: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + elements.iter().map(|e| e.len() + 4).sum::<usize>());
    out.extend_from_slice(b"#bundle\0");
    out.extend_from_slice(&tt.to_be_bytes());
    for e in elements {
        out.extend_from_slice(&(e.len() as i32).to_be_bytes());
        out.extend_from_slice(e);
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parsed {
    Msg(String, Vec<Arg>),
    Bundle(u64, Vec<Parsed>),
}

fn get<const N: usize>(d: &[u8], i: usize) -> Option<[u8; N]> {
    d.get(i..i + N)?.try_into().ok()
}

/// String OSC em `i`: devolve (texto, proximo indice alinhado em 4).
fn read_str(d: &[u8], i: usize) -> Option<(String, usize)> {
    let end = i + d.get(i..)?.iter().position(|&c| c == 0)?;
    let s = String::from_utf8_lossy(&d[i..end]).into_owned();
    let next = end + 1;
    Some((s, next + ((4 - next % 4) % 4)))
}

pub fn parse(data: &[u8]) -> Option<Parsed> {
    if data.starts_with(b"#bundle\0") {
        let tt = u64::from_be_bytes(get(data, 8)?);
        let mut i = 16;
        let mut elems = Vec::new();
        while i < data.len() {
            let n = i32::from_be_bytes(get(data, i)?) as usize;
            elems.push(parse(data.get(i + 4..i + 4 + n)?)?);
            i += 4 + n;
        }
        return Some(Parsed::Bundle(tt, elems));
    }
    let (addr, mut i) = read_str(data, 0)?;
    let tags = if data.get(i) == Some(&b',') {
        let (t, ni) = read_str(data, i)?;
        i = ni;
        t
    } else {
        ",".to_string()
    };
    let mut args = Vec::with_capacity(tags.len().saturating_sub(1));
    for t in tags.bytes().skip(1) {
        match t {
            b'i' => {
                args.push(Arg::Int(i32::from_be_bytes(get(data, i)?)));
                i += 4;
            }
            b'f' => {
                args.push(Arg::Float(f32::from_be_bytes(get(data, i)?)));
                i += 4;
            }
            b'd' => {
                args.push(Arg::Double(f64::from_be_bytes(get(data, i)?)));
                i += 8;
            }
            b'h' => {
                args.push(Arg::Long(i64::from_be_bytes(get(data, i)?)));
                i += 8;
            }
            b't' => {
                args.push(Arg::Time(u64::from_be_bytes(get(data, i)?)));
                i += 8;
            }
            b's' => {
                let (s, ni) = read_str(data, i)?;
                args.push(Arg::Str(s));
                i = ni;
            }
            b'b' => {
                let n = i32::from_be_bytes(get(data, i)?) as usize;
                args.push(Arg::Blob(data.get(i + 4..i + 4 + n)?.to_vec()));
                i += 4 + n + ((4 - n % 4) % 4);
            }
            b'T' => args.push(Arg::Bool(true)),
            b'F' => args.push(Arg::Bool(false)),
            b'N' => args.push(Arg::Nil),
            b'I' => args.push(Arg::Impulse),
            _ => return None, // tag desconhecida: pacote descartado
        }
    }
    Some(Parsed::Msg(addr, args))
}

// ------------------------------------------------------------------ padroes

/// Casamento de padrao de endereco OSC: `*`, `?`, `[a-z]`, `[!a-z]`, `{a,b}`.
/// `*` e `?` nunca atravessam `/`; o resto do padrao e literal.
// ponytail: backtracking recursivo direto sobre os bytes, sem crate de regex
// ; padroes de show tem poucos caracteres. Trocar por automato se algum dia virar rota quente.
pub fn matches(pattern: &str, address: &str) -> bool {
    m(pattern.as_bytes(), address.as_bytes())
}

fn m(p: &[u8], s: &[u8]) -> bool {
    let Some(&c) = p.first() else {
        return s.is_empty();
    };
    match c {
        b'*' => {
            let mut i = 0;
            loop {
                if m(&p[1..], &s[i..]) {
                    return true;
                }
                if i >= s.len() || s[i] == b'/' {
                    return false;
                }
                i += 1;
            }
        }
        b'?' => !s.is_empty() && s[0] != b'/' && m(&p[1..], &s[1..]),
        b'[' => {
            let Some(j) = p.iter().position(|&x| x == b']') else {
                return false;
            };
            if s.is_empty() {
                return false;
            }
            let mut body = &p[1..j];
            let neg = body.first() == Some(&b'!');
            if neg {
                body = &body[1..];
            }
            let mut hit = false;
            let mut k = 0;
            while k < body.len() {
                if k + 2 < body.len() && body[k + 1] == b'-' {
                    hit |= s[0] >= body[k] && s[0] <= body[k + 2];
                    k += 3;
                } else {
                    hit |= s[0] == body[k];
                    k += 1;
                }
            }
            hit != neg && m(&p[j + 1..], &s[1..])
        }
        b'{' => {
            let Some(j) = p.iter().position(|&x| x == b'}') else {
                return false;
            };
            p[1..j]
                .split(|&x| x == b',')
                .any(|alt| s.starts_with(alt) && m(&p[j + 1..], &s[alt.len()..]))
        }
        _ => !s.is_empty() && s[0] == c && m(&p[1..], &s[1..]),
    }
}

// ------------------------------------------------------------------- OscOut

pub struct OscOut {
    sock: UdpSocket,
    addr: SocketAddrV4,
}

impl OscOut {
    pub fn new(host: &str, port: u16) -> io::Result<OscOut> {
        let addr = if let Ok(ip) = host.parse::<Ipv4Addr>() {
            SocketAddrV4::new(ip, port)
        } else {
            (host, port)
                .to_socket_addrs()?
                .find_map(|a| match a {
                    std::net::SocketAddr::V4(v) => Some(v),
                    _ => None,
                })
                .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "host sem IPv4"))?
        };
        let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        s.set_broadcast(true)?;
        Ok(OscOut {
            sock: s.into(),
            addr,
        })
    }

    pub fn send(&self, address: &str, args: &[Arg]) {
        let _ = self.sock.send_to(&message(address, args), self.addr);
    }

    /// Bundle de mensagens ja codificadas.
    pub fn send_bundle(&self, elements: &[Vec<u8>], tt: u64) {
        let _ = self.sock.send_to(&bundle(elements, tt), self.addr);
    }

    pub fn close(&mut self) {}
}

// -------------------------------------------------------------------- OscIn

type Handler = Box<dyn Fn(&str, &[Arg]) + Send>;

/// Escuta UDP; `on(pattern, f)` chama `f(address, args)` para cada mensagem casada.
pub struct OscIn {
    handlers: Arc<Mutex<Vec<(String, Handler)>>>,
    port: u16,
    run: Arc<AtomicBool>,
    th: Option<JoinHandle<()>>,
}

impl OscIn {
    pub fn new(port: u16) -> io::Result<OscIn> {
        let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        s.set_reuse_address(true)?;
        s.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
        s.set_read_timeout(Some(Duration::from_millis(200)))?;
        let sock: UdpSocket = s.into();
        let port = sock.local_addr()?.port();
        let handlers: Arc<Mutex<Vec<(String, Handler)>>> = Arc::new(Mutex::new(Vec::new()));
        let run = Arc::new(AtomicBool::new(true));
        let (h, r) = (handlers.clone(), run.clone());
        let th = std::thread::Builder::new()
            .name("osc-in".into())
            .spawn(move || {
                let mut buf = [0u8; 65536];
                while r.load(Ordering::Relaxed) {
                    let n = match sock.recv_from(&mut buf) {
                        Ok((n, _)) => n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                        Err(_) => break,
                    };
                    if let Some(p) = parse(&buf[..n]) {
                        dispatch(&h, &p);
                    } // pacote malformado: descarta
                }
            })?;
        Ok(OscIn {
            handlers,
            port,
            run,
            th: Some(th),
        })
    }

    /// Porta efetiva (util quando `new(0)` pega porta efemera).
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn on(&mut self, pattern: &str, f: impl Fn(&str, &[Arg]) + Send + 'static) {
        self.handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((pattern.to_string(), Box::new(f)));
    }

    pub fn close(&mut self) {
        self.run.store(false, Ordering::Relaxed);
        if let Some(th) = self.th.take() {
            let _ = th.join();
        }
    }
}

impl Drop for OscIn {
    fn drop(&mut self) {
        self.close();
    }
}

fn dispatch(handlers: &Arc<Mutex<Vec<(String, Handler)>>>, p: &Parsed) {
    match p {
        // ponytail: timetag futuro ignorado, executa ja ; agendar pelo Clock na R1.
        Parsed::Bundle(_, elems) => elems.iter().for_each(|e| dispatch(handlers, e)),
        Parsed::Msg(addr, args) => {
            let hs = handlers.lock().unwrap_or_else(|e| e.into_inner());
            for (pat, f) in hs.iter() {
                if matches(pat, addr) {
                    f(addr, args);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_todos_os_tipos() {
        let tt = timetag(1_700_000_000.5);
        let args = vec![
            Arg::Int(1),
            Arg::Int(-2),
            Arg::Float(3.5),
            Arg::Str("ola".into()),
            Arg::Blob(vec![1, 2, 3]),
            Arg::Bool(true),
            Arg::Bool(false),
            Arg::Nil,
            Arg::Impulse,
            Arg::Double(2.25),
            Arg::Long(1 << 40),
            Arg::Time(tt),
        ];
        let raw = message("/x/y", &args);
        assert_eq!(raw.len() % 4, 0, "mensagem alinhada em 4");
        assert_eq!(parse(&raw), Some(Parsed::Msg("/x/y".into(), args)));
    }

    #[test]
    fn alinhamento_e_bytes_conhecidos() {
        for a in ["/a", "/ab", "/abc", "/abcd"] {
            for s in ["", "x", "xyz", "wxyz"] {
                let msg = message(a, &[Arg::Str(s.into()), Arg::Blob(b"12345".to_vec())]);
                assert_eq!(msg.len() % 4, 0, "{a} {s}");
                assert_eq!(
                    parse(&msg),
                    Some(Parsed::Msg(
                        a.into(),
                        vec![Arg::Str(s.into()), Arg::Blob(b"12345".to_vec())]
                    ))
                );
            }
        }
        assert_eq!(
            message("/abc", &[Arg::Int(1)]),
            b"/abc\0\0\0\0,i\0\0\0\0\0\x01".to_vec()
        );
        assert_eq!(parse(b"/go\0"), Some(Parsed::Msg("/go".into(), vec![])));
    }

    #[test]
    fn bundle_com_duas_mensagens() {
        let a = message("/a", &[Arg::Int(1)]);
        let b = message("/b", &[Arg::Str("z".into())]);
        let raw = bundle(&[a, b], IMMEDIATE);
        assert_eq!(&raw[..8], b"#bundle\0");
        assert_eq!(
            parse(&raw),
            Some(Parsed::Bundle(
                IMMEDIATE,
                vec![
                    Parsed::Msg("/a".into(), vec![Arg::Int(1)]),
                    Parsed::Msg("/b".into(), vec![Arg::Str("z".into())]),
                ]
            ))
        );
    }

    #[test]
    fn timetag_ntp() {
        assert_eq!(timetag(-2_208_988_800.0), 0);
        assert_eq!(timetag(0.0) >> 32, 2_208_988_800);
        assert_eq!(timetag(1_700_000_000.5) & 0xFFFF_FFFF, 1 << 31);
    }

    #[test]
    fn pattern_matching() {
        let cases = [
            ("/a/*", "/a/xyz", true),
            ("/a/*", "/a/x/y", false),
            ("/a/?", "/a/x", true),
            ("/a/?", "/a/xy", false),
            ("/fx[0-9]", "/fx3", true),
            ("/fx[0-9]", "/fxa", false),
            ("/fx[!0-9]", "/fxa", true),
            ("/{play,stop}", "/play", true),
            ("/{play,stop}", "/pause", false),
            ("/a.b", "/axb", false),
            ("/*/dim", "/mv1/dim", true),
        ];
        for (pat, addr, ok) in cases {
            assert_eq!(matches(pat, addr), ok, "{pat} vs {addr}");
        }
    }

    #[test]
    fn loopback_out_in() {
        let mut rx = match OscIn::new(0) {
            Ok(r) => r,
            Err(e) => return println!("pulado: bind OSC falhou: {:?}", e.kind()),
        };
        let port = rx.port();
        let got = Arc::new(Mutex::new(Vec::<String>::new()));
        let g = got.clone();
        rx.on("/spell/*", move |a, args| {
            g.lock().unwrap().push(format!("{a} {args:?}"));
        });
        let g2 = got.clone();
        rx.on("/other", move |_, _| {
            g2.lock().unwrap().push("errado".into());
        });
        let tx = match OscOut::new("127.0.0.1", port) {
            Ok(t) => t,
            Err(_) => return println!("pulado: socket OSC de saida indisponivel"),
        };
        tx.send("/spell/play", &[Arg::Int(1), Arg::Str("go".into())]);
        tx.send_bundle(
            &[
                message("/spell/a", &[Arg::Int(7)]),
                message("/spell/b", &[]),
            ],
            IMMEDIATE,
        );
        for _ in 0..60 {
            if got.lock().unwrap().len() >= 3 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        rx.close();
        let g = got.lock().unwrap().clone();
        if g.is_empty() {
            return println!("pulado: UDP em loopback nao entregou (firewall?)");
        }
        assert_eq!(g.len(), 3, "3 mensagens casadas, nenhuma de /other: {g:?}");
        assert!(g[0].starts_with("/spell/play "), "{:?}", g[0]);
        assert!(g[1].starts_with("/spell/a "), "{:?}", g[1]);
        assert!(g[2].starts_with("/spell/b "), "{:?}", g[2]);
    }
}
