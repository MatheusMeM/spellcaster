//! Byte-for-byte conformance with Python: shows/medgrupo_r0.spell has to reproduce
//! tests/conformance/medgrupo_u1.bin over the 2577 frames of universe 1.
//! Format of the .bin: u32 LE nframes, then nframes * 512 bytes.

use engine::{show, Timeline, Universes};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../"))
}

#[test]
fn medgrupo_universe_1_byte_for_byte() {
    let bin = std::fs::read(root().join("tests/conformance/medgrupo_u1.bin"))
        .expect("tests/conformance/medgrupo_u1.bin missing: run gen.py");
    let n = u32::from_le_bytes(bin[0..4].try_into().unwrap()) as usize;
    assert_eq!(
        bin.len(),
        4 + n * 512,
        "the .bin size does not match nframes"
    );

    let sh = show::load(&root().join("shows/medgrupo_r0.spell")).unwrap();
    let fps = sh.fps as f64;
    let mut tl = Timeline::new(&sh).unwrap();
    assert_eq!(tl.tracks.len(), 219, "tracks of the baked show");
    assert_eq!(
        tl.tracks.iter().map(|t| t.keys.len()).sum::<usize>(),
        67161,
        "keyframes of the baked show"
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
                "frame {} differs on channel {} (1-based {}): expected {}, got {}",
                i,
                ch,
                ch + 1,
                want[ch],
                got[ch]
            );
        }
    }
    assert_eq!(n, 2577, "nframes of the fixture");
}
