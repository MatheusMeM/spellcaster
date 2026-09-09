//! Leitura/escrita de arquivos .ild (ILDA Image Data Transfer Format).
//!
//! Formatos: 0 3D indexado, 1 2D indexado, 2 paleta, 4 3D true color, 5 2D true color.
//! Header de 32 bytes big-endian; status bit7 = ultimo ponto, bit6 = apagado.
//! Porte 1:1 de `spellcaster/protocols/ilda/ild.py` — `shows/medgrupo_laser.ild` lido e
//! regravado tem que dar os MESMOS bytes (teste `ild_medgrupo_byte_a_byte`).

use std::path::Path;

use crate::frame::{Frame, Point};

pub const LAST: u8 = 0x80;
pub const BLANK: u8 = 0x40;
const HDR: usize = 32;

/// Tamanho do registro de cada formato. `None` = formato invalido.
pub fn rec_size(fmt: u8) -> Option<usize> {
    match fmt {
        0 => Some(8),  // >hhhBB
        1 => Some(6),  // >hhBB
        2 => Some(3),  // >BBB
        4 => Some(10), // >hhhBBBB
        5 => Some(8),  // >hhBBBB
        _ => None,
    }
}

/// Paleta padrao ILDA (64 cores).
pub const DEFAULT_PALETTE: [(u8, u8, u8); 64] = [
    (255, 0, 0),
    (255, 16, 0),
    (255, 32, 0),
    (255, 48, 0),
    (255, 64, 0),
    (255, 80, 0),
    (255, 96, 0),
    (255, 112, 0),
    (255, 128, 0),
    (255, 144, 0),
    (255, 160, 0),
    (255, 176, 0),
    (255, 192, 0),
    (255, 208, 0),
    (255, 224, 0),
    (255, 240, 0),
    (255, 255, 0),
    (224, 255, 0),
    (192, 255, 0),
    (160, 255, 0),
    (128, 255, 0),
    (96, 255, 0),
    (64, 255, 0),
    (32, 255, 0),
    (0, 255, 0),
    (0, 255, 36),
    (0, 255, 73),
    (0, 255, 109),
    (0, 255, 146),
    (0, 255, 182),
    (0, 255, 219),
    (0, 255, 255),
    (0, 227, 255),
    (0, 198, 255),
    (0, 170, 255),
    (0, 142, 255),
    (0, 113, 255),
    (0, 85, 255),
    (0, 56, 255),
    (0, 28, 255),
    (0, 0, 255),
    (32, 0, 255),
    (64, 0, 255),
    (96, 0, 255),
    (128, 0, 255),
    (160, 0, 255),
    (192, 0, 255),
    (224, 0, 255),
    (255, 0, 255),
    (255, 32, 255),
    (255, 64, 255),
    (255, 96, 255),
    (255, 128, 255),
    (255, 160, 255),
    (255, 192, 255),
    (255, 224, 255),
    (255, 255, 255),
    (255, 224, 224),
    (255, 192, 192),
    (255, 160, 160),
    (255, 128, 128),
    (255, 96, 96),
    (255, 64, 64),
    (255, 32, 32),
];

#[inline]
fn be16(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([b[i], b[i + 1]])
}

#[inline]
fn bei16(b: &[u8], i: usize) -> i16 {
    i16::from_be_bytes([b[i], b[i + 1]])
}

/// `name.rstrip(b"\0 ").decode("ascii", "replace")` do Python.
fn decode_name(b: &[u8]) -> String {
    let end = b
        .iter()
        .rposition(|&c| c != 0 && c != b' ')
        .map_or(0, |i| i + 1);
    b[..end]
        .iter()
        .map(|&c| {
            if c < 0x80 {
                c as char
            } else {
                char::REPLACEMENT_CHARACTER
            }
        })
        .collect()
}

/// `s.encode("ascii", "replace")[:8].ljust(8)` do Python: nao-ASCII vira '?', corta em 8,
/// completa com espaco.
fn encode_name(s: &str) -> [u8; 8] {
    let mut out = [b' '; 8];
    for (i, c) in s.chars().take(8).enumerate() {
        out[i] = if c.is_ascii() { c as u8 } else { b'?' };
    }
    out
}

/// Le .ild. Secao de paleta (fmt 2) troca a paleta dos frames indexados seguintes.
pub fn read(path: &Path) -> Result<Vec<Frame>, String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    read_bytes(&data)
}

pub fn read_bytes(data: &[u8]) -> Result<Vec<Frame>, String> {
    let mut frames = Vec::new();
    let mut palette: Vec<(u8, u8, u8)> = DEFAULT_PALETTE.to_vec();
    let mut pos = 0usize;
    while pos + HDR <= data.len() {
        let h = pos;
        let fmt = data[h + 7];
        let name = decode_name(&data[h + 8..h + 16]);
        let n = be16(data, h + 24) as usize;
        pos += HDR;
        let rsize = match rec_size(fmt) {
            Some(s) if data[h..h + 4] == *b"ILDA" => s,
            _ => {
                return Err(format!(
                    "header invalido em {h}: {:?} fmt={fmt}",
                    &data[h..h + 4]
                ))
            }
        };
        if n == 0 {
            break; // frame final vazio = fim
        }
        if pos + n * rsize > data.len() {
            return Err(format!("registros truncados em {pos}"));
        }
        let recs = &data[pos..pos + n * rsize];
        pos += n * rsize;
        if fmt == 2 {
            palette = (0..n)
                .map(|i| (recs[i * 3], recs[i * 3 + 1], recs[i * 3 + 2]))
                .collect();
            continue;
        }
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let r = &recs[i * rsize..];
            let (x, y) = (bei16(r, 0), bei16(r, 2));
            let z = if fmt == 0 || fmt == 4 { 2 } else { 0 }; // fmt 3D tem Z antes do status
            let (st, col) = match fmt {
                0 | 1 => {
                    let ci = r[5 + z] as usize;
                    (r[4 + z], *palette.get(ci).unwrap_or(&(255, 255, 255)))
                }
                _ => (r[4 + z], (r[7 + z], r[6 + z], r[5 + z])), // status, B, G, R
            };
            pts.push(Point::new(
                x as f64,
                y as f64,
                col.0,
                col.1,
                col.2,
                st & BLANK != 0,
            ));
        }
        frames.push(Frame { points: pts, name });
    }
    Ok(frames)
}

fn section(
    out: &mut Vec<u8>,
    fmt: u8,
    name: &str,
    company: &str,
    n: usize,
    idx: usize,
    total: usize,
) {
    out.extend_from_slice(b"ILDA");
    out.extend_from_slice(&[0, 0, 0, fmt]);
    out.extend_from_slice(&encode_name(name));
    out.extend_from_slice(&encode_name(company));
    out.extend_from_slice(&(n as u16).to_be_bytes());
    out.extend_from_slice(&(idx as u16).to_be_bytes());
    out.extend_from_slice(&(total as u16).to_be_bytes());
    out.extend_from_slice(&[0, 0]); // projector + pad
}

/// Indice da cor mais proxima na paleta (menor distancia quadratica; empate fica no menor indice).
fn index_of(col: (u8, u8, u8), pal: &[(u8, u8, u8)]) -> u8 {
    let d = |c: &(u8, u8, u8)| {
        let f = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
        f(col.0, c.0) + f(col.1, c.1) + f(col.2, c.2)
    };
    pal.iter()
        .enumerate()
        .min_by_key(|(_, c)| d(c))
        .map_or(0, |(i, _)| i as u8)
}

pub fn write(
    path: &Path,
    frames: &[Frame],
    fmt: u8,
    name: &str,
    company: &str,
    palette: Option<&[(u8, u8, u8)]>,
) -> Result<(), String> {
    let bytes = write_bytes(frames, fmt, name, company, palette)?;
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// Grava frames em .ild. fmt 0/1 indexado usa `palette` (gravada antes como secao fmt 2)
/// ou a padrao.
pub fn write_bytes(
    frames: &[Frame],
    fmt: u8,
    name: &str,
    company: &str,
    palette: Option<&[(u8, u8, u8)]>,
) -> Result<Vec<u8>, String> {
    if !matches!(fmt, 0 | 1 | 4 | 5) {
        return Err("fmt deve ser 0, 1, 4 ou 5".into());
    }
    let rsize = rec_size(fmt).unwrap();
    let total = frames.len();
    let pal: &[(u8, u8, u8)] = palette.unwrap_or(&DEFAULT_PALETTE);
    let mut out = Vec::with_capacity(
        HDR * (total + 2) + frames.iter().map(|f| f.len().max(1) * rsize).sum::<usize>(),
    );
    if let (Some(p), 0 | 1) = (palette, fmt) {
        section(&mut out, 2, name, company, p.len(), 0, total);
        for c in p {
            out.extend_from_slice(&[c.0, c.1, c.2]);
        }
    }
    // frame vazio: 1 ponto apagado (n=0 seria fim de arquivo)
    let empty = [Point {
        blank: true,
        ..Point::default()
    }];
    for (idx, fr) in frames.iter().enumerate() {
        let pts: &[Point] = if fr.points.is_empty() {
            &empty
        } else {
            &fr.points
        };
        let fname = if fr.name.is_empty() { name } else { &fr.name };
        section(&mut out, fmt, fname, company, pts.len(), idx, total);
        for (i, p) in pts.iter().enumerate() {
            let st = if i == pts.len() - 1 { LAST } else { 0 } | if p.blank { BLANK } else { 0 };
            out.extend_from_slice(&p.x.to_be_bytes());
            out.extend_from_slice(&p.y.to_be_bytes());
            if fmt == 0 || fmt == 4 {
                out.extend_from_slice(&0i16.to_be_bytes());
            }
            out.push(st);
            if fmt == 0 || fmt == 1 {
                out.push(index_of((p.r, p.g, p.b), pal));
            } else {
                out.extend_from_slice(&[p.b, p.g, p.r]);
            }
        }
    }
    section(&mut out, fmt, name, company, 0, total, total); // terminador
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Point;

    fn frames() -> Vec<Frame> {
        let quad: Vec<Point> = [(-1i32, -1i32), (1, -1), (1, 1), (-1, 1)]
            .iter()
            .map(|(x, y)| Point::new((x * 10000) as f64, (y * 10000) as f64, 255, 0, 0, false))
            .collect();
        let circ: Vec<Point> = (0..9)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 8.0;
                Point::new(5000.0 * a.cos(), 5000.0 * a.sin(), 0, 255, 0, i == 0)
            })
            .collect();
        vec![
            Frame::new(quad, "quad"),
            Frame::new(circ, "circ"),
            Frame::default(),
        ]
    }

    fn roundtrip(fmt: u8, palette: Option<&[(u8, u8, u8)]>) -> Vec<Frame> {
        let src = frames();
        let raw = write_bytes(&src, fmt, "t", "spell", palette).unwrap();
        let back = read_bytes(&raw).unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back[0].name, "quad");
        assert_eq!(back[1].name, "circ");
        for (a, b) in src.iter().zip(&back).take(2) {
            let key = |f: &Frame| {
                f.points
                    .iter()
                    .map(|p| (p.x, p.y, p.blank))
                    .collect::<Vec<_>>()
            };
            assert_eq!(key(a), key(b));
        }
        assert_eq!(back[2].len(), 1); // frame vazio vira 1 ponto apagado
        assert!(back[2].points[0].blank);
        back
    }

    #[test]
    fn fmt5_true_color() {
        let b = roundtrip(5, None);
        assert_eq!(
            (b[0].points[0].r, b[0].points[0].g, b[0].points[0].b),
            (255, 0, 0)
        );
        assert_eq!(
            (b[1].points[1].r, b[1].points[1].g, b[1].points[1].b),
            (0, 255, 0)
        );
    }

    #[test]
    fn fmt1_paleta_padrao() {
        let b = roundtrip(1, None);
        assert_eq!(
            (b[0].points[0].r, b[0].points[0].g, b[0].points[0].b),
            (255, 0, 0)
        );
    }

    #[test]
    fn fmt1_com_secao_de_paleta() {
        let pal = [(1u8, 2u8, 3u8), (255, 0, 0), (0, 255, 0)];
        let b = roundtrip(1, Some(&pal));
        assert_eq!(
            (b[1].points[1].r, b[1].points[1].g, b[1].points[1].b),
            (0, 255, 0)
        );
    }

    #[test]
    fn fmt0_e_4() {
        roundtrip(0, None);
        roundtrip(4, None);
    }

    #[test]
    fn layout_do_seed() {
        // identico ao tests/test_ilda.py::test_seed_layout_matches
        let f = Frame::new(
            vec![
                Point::new(1.0, -2.0, 10, 20, 30, false),
                Point::new(3.0, 4.0, 0, 0, 0, true),
            ],
            "",
        );
        let raw = write_bytes(&[f], 5, "medgrupo", "feitic.", None).unwrap();
        assert_eq!(&raw[..4], b"ILDA".as_slice());
        assert_eq!(raw[7], 5);
        assert_eq!(&raw[8..16], b"medgrupo".as_slice());
        let esperado: Vec<u8> = vec![0, 1, 255, 254, 0, 30, 20, 10, 0, 3, 0, 4, 0xC0, 0, 0, 0];
        assert_eq!(&raw[32..48], &esperado[..]);
        assert_eq!(raw.len(), 32 + 16 + 32);
    }

    #[test]
    fn fmt_invalido_recusado() {
        assert!(write_bytes(&[], 2, "", "", None).is_err());
        assert!(read_bytes(b"NOPE\0\0\0\x05aaaaaaaabbbbbbbb\0\x01\0\0\0\x01\0\0").is_err());
    }
}
