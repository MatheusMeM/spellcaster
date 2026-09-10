// MED GRUPO RJ show - the R1 "fx" track: a port of shows/medgrupo.py (the FINAL look(t) function,
// already with the PRE = 2.0 pre-roll and the VIDEO channel). It reproduces
// tests/conformance/medgrupo_u1.bin byte for byte across the 2577 frames of universe 1.
//
// Host API (crate `script`): set(addr, values) and st(i) / st(i, v).
// Persistent state = the pan hysteresis of the 8 moving lights (Python's _LAST, which over there is a
// dict per address). Here it is by unit INDEX u in 0..7 (0..3 back bar, 4..7 front bar), 3 slots
// each:  st(3u) = phi of the previous frame, st(3u+1) = m, st(3u+2) = 1 if already initialized.
// Python's default (`_LAST.get(a, (phi, m))`) is the frame's own value.
//
// Porting rules (traps that were measured, not assumed):
//  - every numeric VALUE literal is a float: Rhai's `int()` only exists for floats, and the
//    truncation towards zero + 0..255 clamp in `set` is already the one from Python's `Universe.set`.
//  - `/` between two integers in Rhai is INTEGER division; where Python does true division the
//    denominator here is a float. Where Python does `//` (i // 5) both sides stay integers.
//  - float `%`: Python's is floored, Rhai/Rust's is truncated. Here every operand is >= 0 -> equal.
//  - CPython 3.13's math.hypot is NOT sqrt(x*x+z*z) in the last bit (812 of this show's 5166 pairs
//    diverge), but the divergence never changes an output byte: sqrt is what is used here.
//  - sin, exp, atan2 and degrees from CPython and from Rust match bit for bit on this machine.

fn fmin(a, b) { if b < a { b } else { a } }
fn fmax(a, b) { if b > a { b } else { a } }

fn beat(t, f, k) { exp(-k * ((t * f) % 1.0)) }

fn ease(a, b, u) {
    let x = fmax(0.0, fmin(1.0, u));
    x = x * x * (3.0 - 2.0 * x);
    a + (b - a) * x
}

fn sin01(t, f, ph) { 0.5 + 0.5 * sin(2.0 * PI() * f * t + ph) }

fn rgb(r, g, b) { [int(r), int(g), int(b)] }

// Brisas BX-940 / LED par: R,G,B
fn par(v, c) { rgb(v * c[0] / 255.0, v * c[1] / 255.0, v * c[2] / 255.0) }

// BX-402 footlight wash: R,G,B,W
fn wash(v, c) {
    let m = fmin(fmin(c[0], c[1]), c[2]);
    [int(v * c[0] / 255.0), int(v * c[1] / 255.0), int(v * c[2] / 255.0), int(v * m / 255.0)]
}

// WLED footlight bar 6 ch: R,G,B,W,UV,A
fn bar(v, r, g, b) { [int(v * r), int(v * g), int(v * b), int(v * fmin(fmin(r, g), b)), 0, 0] }
fn barc(v, c) { bar(v, c[0] / 255.0, c[1] / 255.0, c[2] / 255.0) }

fn lerp3(c1, c2, u) {
    let x = fmax(0.0, fmin(1.0, u));
    [c1[0] + (c2[0] - c1[0]) * x, c1[1] + (c2[1] - c1[1]) * x, c1[2] + (c2[2] - c1[2]) * x]
}

// Flash Butrym 10W RGB, 10 ch (patch 13): 1 on/off, 2 strobe, 3 size, 4 pos h, 5 pos v,
// 6 pattern, 7 colour, 8 scan speed, 9 lib, 10 auto/sound. The profile has no rotation channel.
fn laser(size, posh, posv, fig, color, on) {
    if !on { return [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; }
    [255, 255, int(size), int(posh), int(posv), int(fig), int(color), 0, 0, 0]
}

// FOS Scorpio BSW 17 ch, a group of 4 as one unit. `fundo` picks the GEO (back bar vs front bar);
// `only` = the single address to write, -1 = all of them. `only` exists because Python's mv_solo
// calls mv() on the whole group (the hysteresis of all 4 advances) and only returns ONE unit.
// 16-bit pan/tilt; the pan turn is chosen frame by frame by continuity.
fn mv(fundo, only, dim, dz, dx, spread, color, gobo, grot, gobo2, prism, frost,
      zoom, strobe, wave, t, wph, wf, focus) {
    let grp = if fundo { [300, 320, 340, 360] } else { [380, 404, 424, 444] };
    let base = if fundo { 0 } else { 4 };
    let lado = if fundo { 1.0 } else { -1.0 };
    let zmin = if fundo { -1.2 } else { -0.15 };
    let n = grp.len();
    let half = (n - 1) / 2.0;
    let den = fmax(1.0, half);
    for (a, i) in grp {
        let side = (i - half) / den;
        let x = dx + spread * side;
        let z = fmax(zmin, dz + wave * sin(2.0 * PI() * wf * t + i * wph));
        let m = fmin(1.2, sqrt(x * x + z * z));
        let phi = if m > 0.02 { to_degrees(atan(z, lado * x)) } else { 0.0 };
        let s = base + i;
        let lp = phi;
        let lm = m;
        if st(3 * s + 2) != 0.0 { lp = st(3 * s); lm = st(3 * s + 1); }
        // the pan turn closest to the previous frame; order -360, 0, +360 (a tie = the first one)
        let bv = phi;
        let bd = 0.0;
        let have = false;
        let c = phi - 360.0;
        if abs(c) <= 265.0 { bv = c; bd = abs(c - lp); have = true; }
        c = phi;
        if abs(c) <= 265.0 {
            let d = abs(c - lp);
            if !have || d < bd { bv = c; bd = d; have = true; }
        }
        c = phi + 360.0;
        if abs(c) <= 265.0 {
            let d = abs(c - lp);
            if !have || d < bd { bv = c; bd = d; }
        }
        phi = lp + fmax(-2.0, fmin(2.0, bv - lp));      // SLEW_PAN = 2 units per frame
        m = lm + fmax(-0.03, fmin(0.03, m - lm));       // SLEW_TILT = 0.03
        st(3 * s, phi);
        st(3 * s + 1, m);
        st(3 * s + 2, 1.0);
        if only < 0 || only == a {
            let pn = 128.0 + phi * (255.0 / 540.0);
            let tl = 128.0 + 38.0 * m;
            let p16 = to_int(fmax(0.0, fmin(255.0, pn)) * 257.0);
            let t16 = to_int(fmax(0.0, fmin(255.0, tl)) * 257.0);
            set(a, [p16 >> 8, p16 & 255, t16 >> 8, t16 & 255, color, gobo, grot, gobo2,
                    strobe, dim, focus, 0, fmax(80.0, fmin(255.0, zoom)), prism, frost, 0, 0]);
        }
    }
}

fn mv8(only, dim, dz, dx, spread, color, gobo, grot, gobo2, prism, frost,
       zoom, strobe, wave, t, wph, wf, focus) {
    mv(true, only, dim, dz, dx, spread, color, gobo, grot, gobo2, prism, frost,
       zoom, strobe, wave, t, wph, wf, focus);
    mv(false, only, dim, dz, dx, spread, color, gobo, grot, gobo2, prism, frost,
       zoom, strobe, wave, t, wph, wf, focus);
}

// epilogue: one unit alone, straight down, gobo turning (AMARELO 48, G_RAIOS 120)
fn mv_solo(a) {
    mv(a < 380, a, 255.0, 0.0, 0.0, 0.0, 48, 120, 140, 0, 0, 0, 200.0, 255,
       0.0, 0.0, 0.8, 0.6, 128);
}

fn escala(v, g) {
    let o = [];
    for x in v { o.push(int(x * g)); }
    o
}

// Python's ORDEM: bar pars, stage pars, fresnels, the 8 moving lights, WLED bar, wash, laser.
fn ordem(n, g) {
    let CA = [255.0, 215.0, 0.0];
    let CL = [150.0, 255.0, 0.0];
    if n < 4 { set([1, 4, 7, 10][n], escala(par(255.0, CA), g)); }
    else if n < 10 { set([187, 190, 193, 196, 199, 202][n - 4], escala(par(255.0, CL), g)); }
    else if n < 16 { set([29, 34, 39, 44, 49, 54][n - 10], escala([255], g)); }
    else if n < 24 { mv_solo([300, 320, 340, 360, 380, 404, 424, 444][n - 16]); }
    else if n < 32 { set([205, 211, 217, 223, 229, 235, 241, 247][n - 24], escala(barc(255.0, CA), g)); }
    else if n < 42 {
        set([260, 264, 268, 272, 276, 280, 284, 288, 292, 296][n - 32],
            escala(wash(255.0, CL), g));
    } else {
        set(13, escala(laser(200.0, 128.0, 128.0, 0.0, 107.0, true), g));
    }
}

// 5E dramaturgy over the video (46.8 s) + didactic epilogue. See the look() docstring in the .py.
fn luz(t) {
    let PARS = [1, 4, 7, 10, 187, 190, 193, 196, 199, 202];
    let FRESNEL = [29, 34, 39, 44, 49, 54];
    let BAR = [205, 211, 217, 223, 229, 235, 241, 247];
    let WASH = [260, 264, 268, 272, 276, 280, 284, 288, 292, 296];
    let CA = [255.0, 215.0, 0.0];       // C_AMARELO (yellow)
    let CL = [150.0, 255.0, 0.0];       // C_LIMA (lime)
    let CN = [0.0, 25.0, 110.0];        // C_NAVY
    let CB = [255.0, 255.0, 255.0];     // C_BRANCO (white)
    // colour wheel: BRANCO 0, VERMELHO 16, VERDE 32, AMARELO 48, AZUL 64 (white, red, green, yellow, blue)
    // rotating gobo: G_PONTOS 104, G_ANEL 88, G_RAIOS 120 ; shutter: ABERTO 255, STROBE 48

    if t < 10.5 {
        // EXCITEMENT - countdown 10..1 up to 10.5 s
        let b = beat(t, 1.0, 4.0);
        let k = to_int(t);
        for a in FRESNEL { set(a, if k < 2 { int(120.0 * b) } else { 30 }); }
        for (a, i) in BAR {
            set(a, barc(if k >= 2 && k % 2 == i % 2 { 255.0 * b } else { 0.0 }, CA));
        }
        for (a, i) in WASH {
            set(a, wash(if k >= 3 && (i / 5) == k % 2 { 180.0 * b } else { 0.0 }, CL));
        }
        for a in PARS { set(a, par(if k >= 4 { 40.0 + 160.0 * b } else { 0.0 }, CN)); }
        mv(true, -1, ease(0.0, 220.0, (t - 5.0) / 1.0) * (0.6 + 0.4 * b), -0.3, 0.0,
           ease(0.0, 0.6, (t - 5.0) / 4.0), 48, 104, 135, 0, 0, 0, 230.0, 255,
           0.0, 0.0, 0.8, 0.6, 128);
        mv(false, -1, ease(0.0, 220.0, (t - 7.0) / 1.0) * (0.6 + 0.4 * b), 0.3, 0.0,
           ease(0.0, 0.6, (t - 7.0) / 3.0), 32, 88, 120, 0, 0, 0, 230.0, 255,
           0.0, 0.0, 0.8, 0.6, 128);
        set(13, laser(if k >= 1 { 100.0 + 80.0 * b } else { 0.0 }, 128.0, 128.0, 0.0, 107.0, k >= 1));
        if t >= 10.1 {
            // breath: 0.4 s of blackout before the video's yellow flash (only the moving lights carry on)
            for a in FRESNEL { set(a, 0); }
            for a in BAR { set(a, [0, 0, 0, 0, 0, 0]); }
            for a in WASH { set(a, [0, 0, 0, 0]); }
            for a in PARS { set(a, [0, 0, 0]); }
            set(13, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            mv(true, -1, 0.0, -0.3, 0.0, 0.6, 48, 104, 135, 0, 0, 0, 230.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
            mv(false, -1, 0.0, 0.3, 0.0, 0.6, 32, 88, 120, 0, 0, 0, 230.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
        }
    } else if t < 12.0 {
        // ENTRY - the video's yellow flash (10.5-11.3) and the '0' (11.5-12)
        let u = (t - 10.5) / 1.5;
        let stb = if u < 0.5 { 48 } else { 255 };
        let c = if u < 0.55 { CA } else { CB };
        for a in PARS { set(a, par(255.0, c)); }
        for a in FRESNEL { set(a, 255); }
        for a in BAR { set(a, barc(255.0, c)); }
        for a in WASH { set(a, wash(255.0, c)); }
        mv8(-1, 255.0, ease(-0.3, -0.9, u), 0.0, ease(0.6, 0.2, u),
            if u < 0.55 { 48 } else { 0 }, 0, 0, 0, 255, 0, 255.0, stb,
            0.0, 0.0, 0.8, 0.6, 128);
        set(13, laser(ease(80.0, 255.0, u), 128.0, 128.0, 0.0, 255.0, true));
    } else if t < 33.12 {
        // ENGAGEMENT - blocks anchored to the video cuts (CUES)
        let CUES = [12.0, 14.8, 17.72, 19.72, 21.36, 23.36, 26.84, 29.5];
        let seg = -1;
        for (cv, ci) in CUES { if cv <= t { seg = ci; } }   // bisect_right(CUES, t) - 1
        let t0 = CUES[seg];
        let t1 = if seg + 1 < 8 { CUES[seg + 1] } else { 33.12 };
        let us = (t - t0) / (t1 - t0);
        let u = (t - 12.0) / (33.12 - 12.0);
        let m3 = seg % 3;
        let cp = if m3 == 0 { CA } else if m3 == 1 { CL } else { CN };
        let cw = if m3 == 0 { CL } else if m3 == 1 { CA } else { CN };
        let cs = if m3 == 0 { 48 } else if m3 == 1 { 32 } else { 0 };
        let gb = if m3 == 0 { 104 } else if m3 == 1 { 120 } else { 88 };
        for (a, i) in PARS { set(a, par(255.0 * (0.3 + 0.7 * sin01(t, 0.25, i)), cp)); }
        for (a, i) in FRESNEL {
            set(a, if seg % 2 == 0 { int(80.0 * sin01(t, 0.25, i)) } else { 0 });
        }
        let tb = to_int(t * 2.0) % 8;
        for (a, i) in BAR { set(a, barc(if tb == i { 255.0 } else { 40.0 }, cp)); }
        for (a, i) in WASH { set(a, wash(if tb == i { 255.0 } else { 70.0 }, cw)); }
        if seg == 0 {
            // 12.72 the list comes in: back bar in a breathing fan on the screen, front bar in tight beams
            mv(true, -1, 230.0, -0.5, 0.0, 0.5 + 0.3 * sin(2.0 * PI() * 0.3 * t), cs, gb, 135,
               0, 0, 0, 240.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
            mv(false, -1, 200.0, 0.1, 0.0, 0.2, cs, gb, 120, 0, 0, 0, 200.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
        } else if seg == 1 {
            // 14.8 Rio: front bar sweeps up with prism, back bar in a frost wash on the edges
            mv(false, -1, 240.0, ease(0.1, 0.9, us), 0.0, 0.4, cs, gb, 0, 0, 255, 0, 220.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
            mv(true, -1, 180.0, -0.4, 0.0, 0.6, cs, 0, 0, 0, 0, 255, 255.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
        } else if seg == 2 {
            // 17.72: the 8 in a slow wave from left to right
            mv8(-1, 240.0, 0.3, 0.0, 0.5, cs, gb, 130, 0, 0, 0, 230.0, 255,
                0.4, t, 0.8, 0.4, 128);
        } else if seg == 3 {
            // 19.72: back bar in crossed beams with prism, front bar in an open fan
            mv(true, -1, 240.0, -0.3, 0.0, -0.7, cs, gb, 0, 0, 255, 0, 200.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
            mv(false, -1, 200.0, 0.4, 0.0, 0.8, cs, gb, 120, 0, 0, 0, 255.0, 255,
               0.0, 0.0, 0.8, 0.6, 128);
        } else if seg == 4 {
            // 21.36: the 8 turn in unison from the screen to the audience
            mv8(-1, 250.0, ease(-0.6, 0.8, us), 0.0, 0.5, cs, gb, 140, 0, 0, 0, 230.0, 255,
                0.0, 0.0, 0.8, 0.6, 128);
        } else if seg == 5 {
            // 23.36 the list stops at CACHOEIRO: contemplative pause
            for a in PARS { set(a, par(90.0, CN)); }
            for a in FRESNEL { set(a, 0); }
            for a in BAR { set(a, barc(30.0, CA)); }
            for a in WASH { set(a, wash(50.0, CN)); }
            mv(true, -1, ease(240.0, 140.0, us * 2.0), -0.2, 0.0, 0.6, 48, 0, 0, 0, 0, 255,
               255.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
            mv(false, -1, ease(200.0, 0.0, us * 2.0), 0.3, 0.0, 0.5, cs, 0, 0, 0, 0, 255,
               255.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
        } else if seg == 6 {
            // 26.84: pick-up, the 8 rise together with the fan opening
            mv8(-1, ease(0.0, 250.0, us * 3.0), ease(-0.2, 0.6, us), 0.0, ease(0.2, 0.6, us),
                48, 120, 140, 0, 0, 0, 230.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
        } else {
            // 29.5 "the final destination is approval": they cross and open with prism, up to the logo
            mv8(-1, 255.0, 0.4, 0.0, -0.7 + 1.3 * sin01(us, 1.0, -PI() / 2.0), 48, 120, 140,
                0, 255, 0, 240.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
        }
        set(13, laser(150.0, 170.0, 128.0 + 70.0 * sin(u * 2.0 * PI() * 3.0), 0.0, 107.0, true));
    } else if t < 40.0 {
        // PEAK - the logo enters at 33.12: the 8 as one unit, zoom closed to 13 degrees only here
        let u = (t - 33.12) / (40.0 - 33.12);
        let b = beat(t, 2.0, 4.0);
        for a in PARS { set(a, par(255.0, lerp3(CL, CA, b))); }
        for a in FRESNEL { set(a, int(150.0 + 100.0 * b)); }
        for a in BAR { set(a, barc(255.0 * b, CA)); }
        let tp = to_int(t * 2.0) % 2;
        for (a, i) in WASH {
            set(a, wash(255.0 * (if i % 2 == tp { b } else { 0.3 }), CL));
        }
        mv8(-1, 255.0, ease(-0.8, 0.8, u), 0.0, 0.3 + 0.3 * sin(2.0 * PI() * 0.5 * t),
            if b > 0.5 { 48 } else { 0 }, 120, 140, 0, 255, 0,
            ease(80.0, 255.0, (u - 0.2) / 0.6), if u < 0.15 { 48 } else { 255 },
            0.0, 0.0, 0.8, 0.6, 128);
        set(13, laser(ease(255.0, 140.0, u), 128.0, 128.0, 0.0, 255.0, true));
    } else if t < 46.8 {
        // EXIT - everything closes to the centre and goes out; the fresnels warm, last
        let u = (t - 40.0) / 6.8;
        for a in PARS { set(a, par(ease(150.0, 0.0, u * 1.5), CA)); }
        for a in BAR { set(a, barc(0.0, CA)); }
        for a in WASH { set(a, wash(ease(120.0, 0.0, u * 1.2), CL)); }
        mv8(-1, ease(220.0, 0.0, u * 1.3), ease(0.5, 0.0, u), 0.0, ease(0.5, 0.0, u), 48,
            0, 0, 0, 0, 255, 255.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
        for a in FRESNEL { set(a, int(ease(200.0, 0.0, fmax(0.0, (u - 0.5) * 2.0)))); }
        set(13, laser(ease(140.0, 0.0, u * 1.3), 128.0, 128.0, 0.0, 107.0, u < 0.7));
    } else {
        // didactic EPILOGUE: blackout, one fixture at a time in patch order, all together, fade
        let te = t - 46.8;
        for a in PARS { set(a, [0, 0, 0]); }
        for a in FRESNEL { set(a, 0); }
        for a in BAR { set(a, [0, 0, 0, 0, 0, 0]); }
        for a in WASH { set(a, [0, 0, 0, 0]); }
        mv8(-1, 0.0, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0, 255.0, 255, 0.0, 0.0, 0.8, 0.6, 128);
        set(13, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        if te < 1.0 { return; }
        let T_DID = 1.0 + 43 * 0.7;
        let n = to_int((te - 1.0) / 0.7);
        if n < 43 {
            ordem(n, 1.0);
        } else {
            let u = (te - T_DID) / 4.0;
            let g = if u < 1.0 { 1.0 } else { fmax(0.0, 1.0 - (u - 1.0) * 2.0) };
            for i in 0..43 { ordem(i, g); }
            mv8(-1, 255.0 * g, 0.3, 0.0, 0.5, 48, 120, 140, 0, 255, 0, 230.0, 255,
                0.0, 0.0, 0.8, 0.6, 128);
            if g == 0.0 { set(13, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); }
        }
    }
}

// Video pre-roll: 1 s of stop + 1 s of replay before the light starts.
// ch1 of the media player (8-15 play, 16-23 replay, 24-31 stop), ch2 = clip.
fn look(t) {
    let PRE = 2.0;
    let VIDEO_END = 46.8;
    let DUR_LUZ = VIDEO_END + (1.0 + 43 * 0.7) + 6;
    luz(if t >= PRE { fmax(0.0, t - PRE) } else { DUR_LUZ });
    let c1 = if t < 1.0 { 28 } else if t < PRE { 20 }
             else if t < PRE + VIDEO_END { 10 } else { 28 };
    set(400, [c1, 0]);
}
