// FOSFORO: RGBA bitmap -> contour -> ILDA points. The 64x64 inputs come from the test
// itself (`quadrado`, `circulo`). Pure ASCII output (cp1252 console).

use std::time::Instant;

use laser::frame::{Point, LIM};
use laser::trace::{paths, trace, Opts};

/// Opaque black `w x h` canvas with `dentro(x, y)` in white.
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

/// Filled 32x32 square centered in 64x64: corners at (16,16) and (47,47).
fn quadrado() -> Vec<u8> {
    tela(64, 64, |x, y| {
        (16..=47).contains(&x) && (16..=47).contains(&y)
    })
}

/// Filled disc of radius 20 centered in 64x64.
fn circulo() -> Vec<u8> {
    tela(64, 64, |x, y| {
        (x as f64 - 31.5).powi(2) + (y as f64 - 31.5).powi(2) <= 400.0
    })
}

/// Runs of lit points (one drawn path = one run).
fn corridas(p: &[Point]) -> usize {
    p.windows(2).filter(|w| !w[0].lit() && w[1].lit()).count()
        + usize::from(p.first().is_some_and(|q| q.lit()))
}

#[test]
fn square_gives_one_path_of_four_vertices() {
    let ps = paths(&quadrado(), 64, 64, &Opts::default());
    assert_eq!(ps.len(), 1, "the square must give a single path");
    // 4 corners + the closing point (equal to the first); the acceptance is 4 +-1
    let n = ps[0].len();
    assert!((4..=6).contains(&n), "vertices after the RDP: {n}");
    assert_eq!(ps[0][0], ps[0][n - 1], "the path must come out closed");
    assert!(ps[0]
        .iter()
        .all(|p| p.x.abs() as i32 <= LIM && p.y.abs() as i32 <= LIM));
    // the filled square takes half the image: the bbox must land near half the range
    let xs: Vec<i32> = ps[0].iter().map(|p| p.x as i32).collect();
    let larg = xs.iter().max().unwrap() - xs.iter().min().unwrap();
    assert!(
        (30000..=35000).contains(&larg),
        "width in ILDA units: {larg}"
    );
}

#[test]
fn circle_gives_one_path_with_points_to_spare() {
    let ps = paths(&circulo(), 64, 64, &Opts::default());
    assert_eq!(ps.len(), 1);
    let n = ps[0].len();
    assert!((8..=80).contains(&n), "points of the circle: {n}");
    // a circle must not become a 4-sided polygon nor keep the raw contour (129 pixels)
    assert!(ps[0]
        .iter()
        .all(|p| p.x.abs() as i32 <= LIM && p.y.abs() as i32 <= LIM));
}

#[test]
fn two_objects_two_paths_with_blanking() {
    let img = tela(64, 64, |x, y| {
        (4..=24).contains(&x) && (4..=24).contains(&y)
            || (40..=60).contains(&x) && (40..=60).contains(&y)
    });
    let ps = paths(&img, 64, 64, &Opts::default());
    assert_eq!(ps.len(), 2, "two separate objects = two paths");
    let pts = trace(&img, 64, 64, &Opts::default());
    assert_eq!(corridas(&pts), 2, "two lit runs");
    assert!(
        pts.iter().any(|p| p.blank),
        "no blanked point between the paths"
    );
}

/// Without a flood fill of the component, the raster still finds candidates INSIDE an object
/// (a lit pixel whose left neighbor is off). None of them may become a second path: it would
/// be the outer contour traced twice.
#[test]
fn a_hole_does_not_duplicate_the_contour() {
    // 9x3 strip with two pixels off in the middle of the middle row
    let furos = tela(16, 16, |x, y| {
        (3..=11).contains(&x) && (6..=8).contains(&y) && !(y == 7 && (x == 5 || x == 9))
    });
    assert_eq!(paths(&furos, 16, 16, &Opts::default()).len(), 1);
    // ring: the closed hole does not become a path (ponytail documented in the header)
    let anel = tela(32, 32, |x, y| {
        let d = (x as i32 - 16).pow(2) + (y as i32 - 16).pow(2);
        (36..=169).contains(&d)
    });
    assert_eq!(paths(&anel, 32, 32, &Opts::default()).len(), 1);
}

#[test]
fn max_points_respected() {
    let img = circulo();
    let o = Opts {
        max_points: 16,
        epsilon: 0.2,
        ..Opts::default()
    };
    let ps = paths(&img, 64, 64, &o);
    let n: usize = ps.iter().map(|p| p.len()).sum();
    assert!(n <= 16, "points after the cut: {n}");
    assert!(n >= 8, "the cut ate the whole circle: {n}");
    // without the cut the same epsilon gives many more
    let solto: usize = paths(
        &img,
        64,
        64,
        &Opts {
            epsilon: 0.2,
            ..Opts::default()
        },
    )
    .iter()
    .map(|p| p.len())
    .sum();
    assert!(solto > n);
}

#[test]
fn an_empty_image_gives_no_point() {
    let vazia = vec![0u8; 32 * 32 * 4]; // black: zero luma
    assert!(paths(&vazia, 32, 32, &Opts::default()).is_empty());
    assert!(trace(&vazia, 32, 32, &Opts::default()).is_empty());
    // a short buffer does not panic
    assert!(trace(&[0u8; 8], 32, 32, &Opts::default()).is_empty());
}

#[test]
fn invert_and_color() {
    let img = quadrado();
    let o = Opts {
        color: Some((10, 20, 30)),
        ..Opts::default()
    };
    assert!(paths(&img, 64, 64, &o)[0]
        .iter()
        .all(|p| (p.r, p.g, p.b) == (10, 20, 30)));
    // inverted, the figure is the black frame: contour of the image border
    let inv = paths(
        &img,
        64,
        64,
        &Opts {
            invert: true,
            ..Opts::default()
        },
    );
    assert_eq!(inv.len(), 1);
    let xs: Vec<i32> = inv[0].iter().map(|p| p.x as i32).collect();
    assert!(xs.iter().min().unwrap() < &-32000 && xs.iter().max().unwrap() > &32000);
}

#[test]
fn deterministic() {
    let img = circulo();
    let o = Opts::default();
    assert_eq!(trace(&img, 64, 64, &o), trace(&img, 64, 64, &o));
}

#[test]
fn time_of_a_1080p_frame() {
    // 20 scattered discs of radius 60, the realistic worst case of a vectorizable NDI frame
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
    println!("1920x1080, 20 discs: {melhor:.2} ms, {n} points");
    assert_eq!(paths(&img, w, h, &o).len(), 20);
    if !cfg!(debug_assertions) {
        assert!(
            melhor < 8.0,
            "PRD target: < 8 ms per frame in release; got {melhor:.2} ms"
        );
    }
}
