//! FOSFORO: bitmap RGBA -> contornos -> pontos ILDA.
//!
//! A metade do NDI->ILDA que nao depende de SDK (`design/FUNCOES/ndi-ilda.md`, algoritmo
//! "Contornos"): mascara por luma, seguimento de borda de Moore, simplificacao
//! Ramer-Douglas-Peucker, ordenacao dos caminhos por vizinho mais proximo, blanking entre
//! caminhos e as MESMAS `optimize`/`safety` do resto do crate. Puro Rust, sem dependencia
//! nova; a entrada e um buffer RGBA cru, venha ele de onde vier.
//!
//! Deterministico: mesma entrada, mesmos bytes de saida (empate de distancia sempre fica
//! com o menor indice, e nao ha iteracao sobre hash).
//!
//! ponytail: sem NDI ; entra quando o NDI SDK estiver instalado (ROADMAP R2) — so falta
//! quem preencha o `rgba`.
//! ponytail: sem Spout e sem captura de tela ; mesma porta de entrada, mesmo motivo.
//! ponytail: sem suavizacao temporal entre frames ; Damping/Lag/OneEuro sao nos do graph
//! em `script` (design/FUNCOES/ndi-ilda.md secao 1), nao parametro escondido daqui.
//! ponytail: so o algoritmo "Contornos" ; esqueleto (Zhang-Suen) e raster entram quando o
//! material de origem for traco ou texto cheio.
//! ponytail: um caminho por objeto, so o contorno externo ; buraco interno vira caminho
//! quando alguem pedir anel — hoje custaria um segundo passe de bordas.
//! ponytail: mascara so por luma ; a mascara pelo alfa (fundo transparente com RGB preto)
//! entra junto com o NDI, que e quem produz esse quadro — hoje nao ha produtor.

use crate::frame::{optimize_into, Point, Safety, ANGLE, BLANK_GAP, DWELL, LIM, MAX_STEP};

/// Teto de caminhos por quadro, do design (MadMapper: "o engine desiste acima de 2 000").
/// Segura tambem o custo O(n^2) da ordenacao por vizinho mais proximo.
pub const MAX_PATHS: usize = 2000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opts {
    /// Corte da mascara: pixel entra quando `luma > threshold`.
    pub threshold: u8,
    /// Tolerancia do Ramer-Douglas-Peucker, EM PIXELS da imagem de entrada.
    pub epsilon: f32,
    /// Teto de pontos ANTES de `optimize` (que ainda acrescenta dwell e blanking).
    pub max_points: usize,
    /// Inverte a mascara (figura escura sobre fundo claro).
    pub invert: bool,
    /// Cor unica para todos os pontos; `None` amostra o pixel de origem sob cada ponto.
    pub color: Option<(u8, u8, u8)>,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            threshold: 128,
            epsilon: 1.5,
            max_points: 4000,
            invert: false,
            color: Some((255, 255, 255)),
        }
    }
}

/// Vizinhanca de Moore em sentido horario (imagem tem Y para baixo).
const D: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

struct Grid<'a> {
    m: &'a [bool],
    w: i32,
    h: i32,
}

impl Grid<'_> {
    #[inline]
    fn at(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h && self.m[(y * self.w + x) as usize]
    }
}

/// Mascara por luma BT.601 em inteiro. Uma passada so: 2 Mpx num quadro 1080p, e a
/// mascara e metade do custo do quadro.
fn mask(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<bool> {
    rgba[..w * h * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let l = ((77 * p[0] as u32 + 150 * p[1] as u32 + 29 * p[2] as u32) >> 8) as u8;
            (l > o.threshold) != o.invert
        })
        .collect()
}

/// Proximo pixel de borda no sentido horario a partir da direcao de chegada `bdir`.
/// Devolve o pixel e a nova direcao de chegada.
#[inline]
fn step(g: &Grid, p: (i32, i32), bdir: usize) -> Option<((i32, i32), usize)> {
    for k in 1..=8 {
        let d = (bdir + k) % 8;
        let q = (p.0 + D[d].0, p.1 + D[d].1);
        if g.at(q.0, q.1) {
            return Some((q, (d + 4) % 8));
        }
    }
    None
}

/// Contorno externo de Moore a partir de `start` (que tem o vizinho da esquerda apagado).
/// Para pelo criterio de Jacob: mesmo pixel chegando pela mesma direcao. Sai fechado.
fn contour(g: &Grid, start: (i32, i32)) -> Vec<(i32, i32)> {
    let mut c = vec![start];
    let (mut p, mut bdir) = match step(g, start, 4) {
        Some(s) => s,
        None => return c, // pixel isolado
    };
    let first = (p, bdir);
    let cap = (g.w as usize * g.h as usize) * 2 + 8;
    loop {
        c.push(p);
        let (q, nb) = match step(g, p, bdir) {
            Some(s) => s,
            None => break,
        };
        p = q;
        bdir = nb;
        if (p, bdir) == first || c.len() >= cap {
            break;
        }
    }
    if c.last() != c.first() {
        c.push(c[0]);
    }
    c
}

/// Ramer-Douglas-Peucker iterativo (pilha explicita: contorno de 1080p passa de 10 000
/// pontos e a versao recursiva estoura a pilha). Empate de distancia fica no menor indice.
fn rdp(pts: &[(i32, i32)], eps: f32) -> Vec<(i32, i32)> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let e2 = eps as f64 * eps as f64;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let (ax, ay) = (pts[a].0 as f64, pts[a].1 as f64);
        let (dx, dy) = (pts[b].0 as f64 - ax, pts[b].1 as f64 - ay);
        let len2 = dx * dx + dy * dy;
        let (mut best, mut bd) = (a, -1.0f64);
        for (i, q) in pts.iter().enumerate().take(b).skip(a + 1) {
            let (qx, qy) = (q.0 as f64 - ax, q.1 as f64 - ay);
            // caminho fechado: a == b, entao a "reta" degenera e a distancia e ao ponto
            let d = if len2 == 0.0 {
                qx * qx + qy * qy
            } else {
                let cr = qx * dy - qy * dx;
                cr * cr / len2
            };
            if d > bd {
                bd = d;
                best = i;
            }
        }
        if bd > e2 {
            keep[best] = true;
            stack.push((a, best));
            stack.push((best, b));
        }
    }
    pts.iter().zip(keep).filter(|(_, k)| *k).map(|(p, _)| *p).collect()
}

/// Corte proporcional: cada caminho fica com `len * max / total` pontos, por amostragem
/// uniforme que preserva as duas pontas (e portanto o fecho do contorno).
// ponytail: o minimo de 2 pontos por caminho pode passar de `max` quando ha mais de
// max/2 caminhos ; MAX_PATHS segura o pior caso, e quem quiser exatidao filtra por
// comprimento antes (o `Limite: manter os N mais longos` do design).
fn decimate(paths: &mut [Vec<(i32, i32)>], max: usize) {
    let total: usize = paths.iter().map(|p| p.len()).sum();
    if max == 0 || total <= max {
        return;
    }
    for p in paths.iter_mut() {
        let keep = ((p.len() as u64 * max as u64) / total as u64).max(2) as usize;
        if keep >= p.len() {
            continue;
        }
        let last = p.len() - 1;
        let src = std::mem::take(p);
        *p = (0..keep).map(|i| src[i * last / (keep - 1)]).collect();
    }
}

/// Ordena os caminhos por vizinho mais proximo a partir do canto superior esquerdo,
/// para encurtar os saltos apagados.
// ponytail: guloso O(n^2) sem inverter caminho ; MAX_PATHS=2000 e o teto do custo
fn order(paths: &mut [Vec<(i32, i32)>]) {
    let mut cur = (0i64, 0i64);
    for i in 0..paths.len() {
        // `rotate_right` guarda a ordem original entre os nao escolhidos, entao o empate
        // continua ficando com o menor indice de entrada
        let best = (i..paths.len())
            .min_by_key(|&j| {
                let s = paths[j][0];
                (s.0 as i64 - cur.0).pow(2) + (s.1 as i64 - cur.1).pow(2)
            })
            .expect("a faixa comeca em i e nunca e vazia");
        paths[i..=best].rotate_right(1);
        let e = paths[i][paths[i].len() - 1];
        cur = (e.0 as i64, e.1 as i64);
    }
}

/// Caminhos vetorizados em coordenadas ILDA (-32767..32767, Y para cima), ANTES do
/// blanking, do `optimize` e da `safety`. Um caminho por objeto, fechado.
///
/// A imagem entra inteira e centrada, com a proporcao preservada (o lado maior ocupa a
/// faixa toda). Buffer menor que `w * h * 4` devolve vazio.
pub fn paths(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Vec<Point>> {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return Vec::new();
    }
    let m = mask(rgba, w, h, o);
    let g = Grid { m: &m, w: w as i32, h: h as i32 };
    let mut seen = vec![false; w * h];
    let mut raw: Vec<Vec<(i32, i32)>> = Vec::new();
    'fora: for y in 0..g.h {
        for x in 0..g.w {
            let i = (y * g.w + x) as usize;
            // candidato = pixel aceso, ainda nao tracado, com o vizinho da esquerda apagado
            if !m[i] || seen[i] || (x > 0 && m[i - 1]) {
                continue;
            }
            let p = (x, y);
            let c = contour(&g, p);
            for &(cx, cy) in &c {
                seen[(cy * g.w + cx) as usize] = true;
            }
            // O contorno EXTERNO de uma componente e o unico cujo pixel de cima-a-esquerda
            // e o proprio candidato: o raster chega nele primeiro. Saindo da borda de um
            // buraco, o ciclo tem minimo menor que `p` — e retraco da mesma componente e
            // cai fora. Substitui o flood fill da componente inteira, que custava ~40 ns
            // por pixel aceso (80 ms num quadro 1080p todo aceso).
            if c.iter().min_by_key(|&&(px, py)| (py, px)) == Some(&p) {
                raw.push(c);
                if raw.len() >= MAX_PATHS {
                    break 'fora;
                }
            }
        }
    }
    for c in raw.iter_mut() {
        *c = rdp(c, o.epsilon);
    }
    decimate(&mut raw, o.max_points);
    order(&mut raw);

    let s = 2.0 * LIM as f64 / (w.max(h) - 1).max(1) as f64;
    let (cx, cy) = ((w - 1) as f64 / 2.0, (h - 1) as f64 / 2.0);
    raw.iter()
        .map(|c| {
            c.iter()
                .map(|&(x, y)| {
                    let (r, gc, b) = o.color.unwrap_or_else(|| {
                        let i = (y as usize * w + x as usize) * 4;
                        (rgba[i], rgba[i + 1], rgba[i + 2])
                    });
                    Point::new((x as f64 - cx) * s, (cy - y as f64) * s, r, gc, b, false)
                })
                .collect()
        })
        .collect()
}

/// Um quadro RGBA vira um frame ILDA pronto para o `Feed`: caminhos, salto apagado entre
/// eles, `optimize` e `safety` padrao.
// ponytail: safety padrao aqui dentro ; zona e limite proprios se aplicam depois, com
// `frame::safety` — o `Feed` reaplica a dele em todo envio de qualquer jeito.
pub fn trace(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Point> {
    let ps = paths(rgba, w, h, o);
    if ps.is_empty() {
        return Vec::new();
    }
    let mut src = Vec::with_capacity(ps.iter().map(|p| p.len() + 1).sum());
    for (i, p) in ps.iter().enumerate() {
        if i > 0 {
            // ponto apagado no inicio do proximo caminho: o `optimize` abre o gap e
            // interpola o salto sozinho
            src.push(Point { blank: true, ..p[0] });
        }
        src.extend_from_slice(p);
    }
    let mut out = Vec::with_capacity(src.len() * 2);
    optimize_into(&src, &mut out, DWELL, BLANK_GAP, MAX_STEP, ANGLE);
    Safety::default().apply(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdp_reduz_reta_e_guarda_canto() {
        let reta: Vec<(i32, i32)> = (0..20).map(|i| (i, 0)).collect();
        assert_eq!(rdp(&reta, 1.0), vec![(0, 0), (19, 0)]);
        let l = [(0, 0), (5, 0), (10, 0), (10, 5), (10, 10)];
        assert_eq!(rdp(&l, 1.0), vec![(0, 0), (10, 0), (10, 10)]);
    }

    #[test]
    fn decimate_corta_proporcional() {
        let mut p = vec![(0..100).map(|i| (i, 0)).collect::<Vec<_>>()];
        decimate(&mut p, 10);
        assert_eq!(p[0].len(), 10);
        assert_eq!(p[0][0], (0, 0));
        assert_eq!(p[0][9], (99, 0));
    }
}
