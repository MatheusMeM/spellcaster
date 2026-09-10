// Pixel mapping conformance: known values in a 2x2 frame and the universe limits.
// Pure ASCII output (cp1252 console).

use pixelmap::{Fixture, Frame, Mapper, Order, PixelMap, Sampling};

/// 2x2 RGB: (0,0)=10,20,30  (1,0)=40,50,60  (0,1)=70,80,90  (1,1)=100,110,120
const PX: [u8; 12] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];

fn fx(universe: u16, channel: u16, order: Order, u: f32, v: f32) -> Fixture {
    Fixture {
        universe,
        channel,
        order,
        u,
        v,
    }
}

fn map(fixtures: Vec<Fixture>) -> PixelMap {
    PixelMap {
        name: "t".into(),
        source: "media/1".into(),
        fixtures,
    }
}

#[test]
fn four_fixtures_in_a_2x2_frame() {
    let mut m = Mapper::new(&map(vec![
        fx(1, 1, Order::Rgb, 0.25, 0.25),  // (0,0)
        fx(1, 10, Order::Grb, 0.75, 0.25), // (1,0), swapped G<->R
        fx(2, 1, Order::Rgbw, 0.25, 0.75), // (0,1), white extracted
        fx(2, 500, Order::Rgb, 1.5, -0.3), // clamp -> (1,0)
    ]));
    assert_eq!((m.universes(), m.pixels()), (2, 4));
    m.render(&Frame::rgb(2, 2, &PX).unwrap());

    let u1 = m.universe(1).unwrap();
    assert_eq!(&u1[0..3], &[10, 20, 30]);
    assert_eq!(&u1[9..12], &[50, 40, 60]);
    assert_eq!(&u1[3..9], &[0; 6], "an uncovered channel stays zero");

    let u2 = m.universe(2).unwrap();
    assert_eq!(
        &u2[0..4],
        &[0, 10, 20, 70],
        "rgbw: w = min(r,g,b), subtracted from the three"
    );
    assert_eq!(
        &u2[499..502],
        &[40, 50, 60],
        "u/v outside 0..1 clamp to the edge"
    );

    // it is the exact order protocols::Output::send receives
    let saida: Vec<u16> = m.frames().map(|(n, _)| n).collect();
    assert_eq!(saida, vec![1, 2]);
}

#[test]
fn universe_limits() {
    let mut m = Mapper::new(&map(vec![
        fx(3, 510, Order::Rgbw, 0.0, 0.0), // 510..513: the W does not fit
        fx(3, 513, Order::Rgb, 0.0, 0.0),  // first channel out of range: dropped
        fx(0, 1, Order::Rgb, 0.0, 0.0),    // universe 0 does not exist in sACN
        fx(3, 0, Order::Rgb, 0.0, 0.0),    // the channel is 1-based
    ]));
    assert_eq!((m.universes(), m.pixels()), (1, 1));
    m.render(&Frame::rgb(2, 2, &PX).unwrap());

    let u3 = m.universe(3).unwrap();
    assert_eq!(
        &u3[509..512],
        &[0, 10, 20],
        "rgbw truncated to the 3 channels that fit"
    );
    assert_eq!(&u3[0..509], &[0u8; 509][..]);
}

#[test]
fn bilinear_and_invalid_frame() {
    let f = Frame::rgb(2, 2, &PX).unwrap();
    assert_eq!(f.sample(0.5, 0.5, Sampling::Nearest), (100, 110, 120));
    assert_eq!(f.sample(0.5, 0.5, Sampling::Bilinear), (55, 65, 75));
    assert_eq!(f.sample(-9.0, 9.0, Sampling::Bilinear), (70, 80, 90));

    // RGBA: the fourth byte is ignored
    let rgba = [1u8, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255];
    assert_eq!(
        Frame::rgba(2, 2, &rgba)
            .unwrap()
            .sample(0.9, 0.9, Sampling::Nearest),
        (10, 11, 12)
    );

    assert!(
        Frame::rgb(2, 2, &PX[..11]).is_err(),
        "a short buffer is an error, not a panic"
    );
    assert!(Frame::rgb(0, 2, &PX).is_err());
    assert!(Frame::new(2, 2, 2, &PX).is_err());
}

#[test]
fn pixelmaps_block_of_the_spell() {
    let src = r#"{"pixelmaps":[{"name":"wall","source":"media/1",
                   "fixtures":[{"universe":10,"channel":1,"order":"grb","u":0.0,"v":0.0},
                               {"universe":10,"channel":4,"u":1.0,"v":1.0}]}]}"#;
    let v: serde_json::Value = serde_json::from_str(src).unwrap();
    let maps: Vec<PixelMap> = serde_json::from_value(v["pixelmaps"].clone()).unwrap();
    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].source, "media/1");
    assert_eq!(maps[0].fixtures[0].order, Order::Grb);
    assert_eq!(
        maps[0].fixtures[1].order,
        Order::Rgb,
        "without \"order\" the default is rgb"
    );

    let mut m = Mapper::new(&maps[0]);
    m.render(&Frame::rgb(2, 2, &PX).unwrap());
    assert_eq!(&m.universe(10).unwrap()[0..6], &[20, 10, 30, 100, 110, 120]);

    // round trip through serde
    let txt = serde_json::to_string(&maps).unwrap();
    let volta: Vec<PixelMap> = serde_json::from_str(&txt).unwrap();
    assert_eq!(volta[0].fixtures.len(), 2);
    assert_eq!(volta[0].name, "wall");
}
