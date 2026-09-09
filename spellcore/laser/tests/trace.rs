// FOSFORO: bitmap RGBA -> contorno -> pontos ILDA. As entradas 64x64 saem do proprio
// teste (`quadrado`, `circulo`). Saida ASCII pura (console cp1252).

use std::time::Instant;

use laser::frame::{Point, LIM};
use laser::trace::{paths, trace, Opts};

/// Tela `w x h` preta opaca com `dentro(x, y)` em branco.
fn tela(w: usize, h: usize, dentro: impl Fn(usize, usize) -> bool) -> Vec<u8> {
    let mut v = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let c = if dentro(x, y) { 255 } else { 0 };
            v.extend_from_slice(&[c, c, c, 255]);
        }
    }
    v
}

/// Quadrado cheio de 32x32 centrado em 64x64: cantos em (16,16) e (47,47).
fn quadrado() -> Vec<u8> {
    tela(64, 64, |x, y| (16..=47).contains(&x) && (16..=47).contains(&y))
}

/// Disco cheio de raio 20 centrado em 64x64.
fn circulo() -> Vec<u8> {
    tela(64, 64, |x, y| (x as f64 - 31.5).powi(2) + (y as f64 - 31.5).powi(2) <= 400.0)
}

/// Corridas de pontos acesos (um caminho desenhado = uma corrida).
fn corridas(p: &[Point]) -> usize {
    p.windows(2).filter(|w| !w[0].lit() && w[1].lit()).count()
        + usize::from(p.first().is_some_and(|q| q.lit()))
}

#[test]
fn quadrado_da_um_caminho_de_quatro_vertices() {
    let ps = paths(&quadrado(), 64, 64, &Opts::default());
    assert_eq!(ps.len(), 1, "quadrado tem que dar um caminho so");
    // 4 cantos + o ponto de fecho (igual ao primeiro); o aceite e 4 +-1
    let n = ps[0].len();
    assert!((4..=6).contains(&n), "vertices depois do RDP: {n}");
    assert_eq!(ps[0][0], ps[0][n - 1], "caminho tem que sair fechado");
    assert!(ps[0].iter().all(|p| p.x.abs() as i32 <= LIM && p.y.abs() as i32 <= LIM));
    // o quadrado cheio ocupa metade da imagem: a bbox tem que ficar perto de metade da faixa
    let xs: Vec<i32> = ps[0].iter().map(|p| p.x as i32).collect();
    let larg = xs.iter().max().unwrap() - xs.iter().min().unwrap();
    assert!((30000..=35000).contains(&larg), "largura em unidades ILDA: {larg}");
}

#[test]
fn circulo_da_um_caminho_com_pontos_de_sobra() {
    let ps = paths(&circulo(), 64, 64, &Opts::default());
    assert_eq!(ps.len(), 1);
    let n = ps[0].len();
    assert!((8..=80).contains(&n), "pontos do circulo: {n}");
    // circulo nao pode virar poligono de 4 lados nem guardar o contorno cru (129 pixels)
    assert!(ps[0].iter().all(|p| p.x.abs() as i32 <= LIM && p.y.abs() as i32 <= LIM));
}

#[test]
fn dois_objetos_dois_caminhos_com_blanking() {
    let img = tela(64, 64, |x, y| {
        (4..=24).contains(&x) && (4..=24).contains(&y)
            || (40..=60).contains(&x) && (40..=60).contains(&y)
    });
    let ps = paths(&img, 64, 64, &Opts::default());
    assert_eq!(ps.len(), 2, "dois objetos separados = dois caminhos");
    let pts = trace(&img, 64, 64, &Opts::default());
    assert_eq!(corridas(&pts), 2, "duas corridas acesas");
    assert!(pts.iter().any(|p| p.blank), "sem ponto apagado entre os caminhos");
}

/// Sem flood fill da componente, o raster ainda encontra candidatos DENTRO de um objeto
/// (pixel aceso com o vizinho da esquerda apagado). Nenhum deles pode virar um segundo
/// caminho: seria o contorno externo tracado duas vezes.
#[test]
fn buraco_nao_duplica_o_contorno() {
    // faixa de 9x3 com dois pixels apagados no meio da linha do meio
    let furos = tela(16, 16, |x, y| {
        (3..=11).contains(&x) && (6..=8).contains(&y) && !(y == 7 && (x == 5 || x == 9))
    });
    assert_eq!(paths(&furos, 16, 16, &Opts::default()).len(), 1);
    // anel: o buraco fechado nao vira caminho (ponytail documentado no cabecalho)
    let anel = tela(32, 32, |x, y| {
        let d = (x as i32 - 16).pow(2) + (y as i32 - 16).pow(2);
        (36..=169).contains(&d)
    });
    assert_eq!(paths(&anel, 32, 32, &Opts::default()).len(), 1);
}

#[test]
fn max_points_respeitado() {
    let img = circulo();
    let o = Opts { max_points: 16, epsilon: 0.2, ..Opts::default() };
    let ps = paths(&img, 64, 64, &o);
    let n: usize = ps.iter().map(|p| p.len()).sum();
    assert!(n <= 16, "pontos depois do corte: {n}");
    assert!(n >= 8, "corte comeu o circulo inteiro: {n}");
    // sem o corte o mesmo epsilon da bem mais
    let solto: usize = paths(&img, 64, 64, &Opts { epsilon: 0.2, ..Opts::default() })
        .iter()
        .map(|p| p.len())
        .sum();
    assert!(solto > n);
}

#[test]
fn imagem_vazia_nao_da_ponto() {
    let vazia = vec![0u8; 32 * 32 * 4]; // preto: luma zero
    assert!(paths(&vazia, 32, 32, &Opts::default()).is_empty());
    assert!(trace(&vazia, 32, 32, &Opts::default()).is_empty());
    // buffer curto nao entra em panico
    assert!(trace(&[0u8; 8], 32, 32, &Opts::default()).is_empty());
}

#[test]
fn invert_e_cor() {
    let img = quadrado();
    let o = Opts { color: Some((10, 20, 30)), ..Opts::default() };
    assert!(paths(&img, 64, 64, &o)[0].iter().all(|p| (p.r, p.g, p.b) == (10, 20, 30)));
    // invertido, a figura e a moldura preta: contorno da borda da imagem
    let inv = paths(&img, 64, 64, &Opts { invert: true, ..Opts::default() });
    assert_eq!(inv.len(), 1);
    let xs: Vec<i32> = inv[0].iter().map(|p| p.x as i32).collect();
    assert!(xs.iter().min().unwrap() < &-32000 && xs.iter().max().unwrap() > &32000);
}

#[test]
fn deterministico() {
    let img = circulo();
    let o = Opts::default();
    assert_eq!(trace(&img, 64, 64, &o), trace(&img, 64, 64, &o));
}

#[test]
fn tempo_de_um_quadro_1080p() {
    // 20 discos de raio 60 espalhados, o pior caso realista de um quadro NDI vetorizavel
    let (w, h) = (1920usize, 1080usize);
    let img = tela(w, h, |x, y| {
        let (cx, cy) = ((x / 384) * 384 + 192, (y / 270) * 270 + 135);
        let (dx, dy) = (x as i64 - cx as i64, y as i64 - cy as i64);
        dx * dx + dy * dy <= 60 * 60
    });
    let o = Opts::default();
    let mut melhor = f64::MAX;
    let mut n = 0;
    for _ in 0..5 {
        let t = Instant::now();
        let pts = trace(&img, w, h, &o);
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
        n = pts.len();
    }
    println!("1920x1080, 20 discos: {melhor:.2} ms, {n} pontos");
    assert_eq!(paths(&img, w, h, &o).len(), 20);
    if !cfg!(debug_assertions) {
        assert!(melhor < 8.0, "meta do PRD: < 8 ms por quadro em release; deu {melhor:.2} ms");
    }
}
