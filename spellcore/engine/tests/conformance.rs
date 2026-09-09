//! Conformidade byte a byte com o Python: shows/medgrupo_r0.spell tem que reproduzir
//! tests/conformance/medgrupo_u1.bin nos 2577 frames do universo 1.
//! Formato do .bin: u32 LE nframes, depois nframes * 512 bytes.

use engine::{show, Timeline, Universes};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../"))
}

#[test]
fn medgrupo_universo_1_byte_a_byte() {
    let bin = std::fs::read(root().join("tests/conformance/medgrupo_u1.bin"))
        .expect("tests/conformance/medgrupo_u1.bin ausente: rode gen.py");
    let n = u32::from_le_bytes(bin[0..4].try_into().unwrap()) as usize;
    assert_eq!(
        bin.len(),
        4 + n * 512,
        "tamanho do .bin nao bate com nframes"
    );

    let sh = show::load(&root().join("shows/medgrupo_r0.spell")).unwrap();
    let fps = sh.fps as f64;
    let mut tl = Timeline::new(&sh).unwrap();
    assert_eq!(tl.tracks.len(), 219, "tracks do show assado");
    assert_eq!(
        tl.tracks.iter().map(|t| t.keys.len()).sum::<usize>(),
        67161,
        "keyframes do show assado"
    );

    let mut uni = Universes::new();
    uni.get_or_create(1);
    for i in 0..n {
        tl.apply(&mut uni, i as f64 / fps);
        let want = &bin[4 + i * 512..4 + (i + 1) * 512];
        let got = &uni.get(1).unwrap().data[..];
        if got != want {
            let ch = (0..512).find(|&c| got[c] != want[c]).unwrap();
            panic!(
                "frame {} difere no canal {} (1-based {}): esperado {}, veio {}",
                i,
                ch,
                ch + 1,
                want[ch],
                got[ch]
            );
        }
    }
    assert_eq!(n, 2577, "nframes do fixture");
}
