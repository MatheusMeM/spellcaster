// Vetoriza um quadro RGBA cru em um frame .ild, para conferir o FOSFORO no olho antes de
// existir NDI. Saida ASCII pura (console cp1252).
//
//   trace in.rgba 1920x1080 out.ild [--threshold 128] [--epsilon 1.5]
//                                   [--max-points 4000] [--invert] [--sampled]
//
// O .rgba e o buffer cru, 4 bytes por pixel, linha a linha de cima para baixo. Do lado do
// dono: qualquer screenshot convertido, por exemplo
//   ffmpeg -i tela.png -pix_fmt rgba -f rawvideo tela.rgba

use std::path::Path;
use std::time::Instant;

use laser::frame::Frame;
use laser::ild;
use laser::trace::{trace, Opts};

fn main() -> Result<(), String> {
    let arg: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| arg.iter().any(|a| a == name);
    let opt = |name: &str| arg.iter().position(|x| x == name).and_then(|i| arg.get(i + 1).cloned());
    let pos: Vec<&String> = arg.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.len() < 3 {
        return Err("uso: trace in.rgba WxH out.ild [--threshold N] [--epsilon F] \
                    [--max-points N] [--invert] [--sampled]"
            .into());
    }
    let (w, h) = pos[1]
        .split_once(['x', 'X'])
        .and_then(|(a, b)| Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?)))
        .ok_or_else(|| format!("tamanho invalido: {} (esperado WxH)", pos[1]))?;

    let mut o = Opts::default();
    if let Some(v) = opt("--threshold") {
        o.threshold = v.parse().map_err(|_| format!("--threshold invalido: {v}"))?;
    }
    if let Some(v) = opt("--epsilon") {
        o.epsilon = v.parse().map_err(|_| format!("--epsilon invalido: {v}"))?;
    }
    if let Some(v) = opt("--max-points") {
        o.max_points = v.parse().map_err(|_| format!("--max-points invalido: {v}"))?;
    }
    o.invert = flag("--invert");
    if flag("--sampled") {
        o.color = None;
    }

    let data = std::fs::read(pos[0]).map_err(|e| format!("{}: {e}", pos[0]))?;
    if data.len() < w * h * 4 {
        return Err(format!("{}: {} bytes, {}x{} pede {}", pos[0], data.len(), w, h, w * h * 4));
    }

    let t0 = Instant::now();
    let pts = trace(&data, w, h, &o);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;

    // caminhos = corridas de pontos acesos; sai de graca, sem uma segunda vetorizacao
    let n = pts.windows(2).filter(|w| !w[0].lit() && w[1].lit()).count()
        + usize::from(pts.first().is_some_and(|p| p.lit()));
    let total = pts.len();
    ild::write(Path::new(&pos[2]), &[Frame::new(pts, "trace")], 5, "trace", "spell", None)?;
    println!("{}: {n} caminhos, {total} pontos, {ms:.2} ms", pos[2]);
    Ok(())
}
