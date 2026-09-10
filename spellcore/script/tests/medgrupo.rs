//! Conformance of the fx track: shows/medgrupo.rhai must reproduce
//! tests/conformance/medgrupo_u1.bin byte by byte across the 2577 frames of universe 1.
//! The frames are evaluated IN ORDER: the pan hysteresis of the moving heads accumulates.

use engine::hook::FrameHook;
use engine::Universes;
use script::Fx;
use std::path::Path;

#[test]
fn medgrupo_rhai_reproduces_the_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let bin = std::fs::read(root.join("tests/conformance/medgrupo_u1.bin")).expect("fixture");
    let n = u32::from_le_bytes(bin[0..4].try_into().unwrap()) as usize;
    assert_eq!(n, 2577, "the fixture changed size");
    assert_eq!(bin.len(), 4 + n * 512);

    let mut fx = Fx::new(&root.join("shows/medgrupo.rhai"), 1).expect("compiles the .rhai");
    let mut uni = Universes::new();
    let mut frames_ruins = 0usize;
    let mut relato = String::new();
    for i in 0..n {
        fx.frame(i as f64 / 30.0, &mut uni);
        let esperado = &bin[4 + i * 512..4 + (i + 1) * 512];
        let obtido = &uni.get(1).expect("universe 1").data[..];
        if obtido != esperado {
            frames_ruins += 1;
            if frames_ruins <= 5 {
                for c in 0..512 {
                    if obtido[c] != esperado[c] && relato.len() < 2000 {
                        relato.push_str(&format!(
                            "frame {} channel {} got {} expected {}\n",
                            i,
                            c + 1,
                            obtido[c],
                            esperado[c]
                        ));
                    }
                }
            }
        }
    }
    assert_eq!(
        frames_ruins, 0,
        "{} of {} frames diverge:\n{}",
        frames_ruins, n, relato
    );
}
