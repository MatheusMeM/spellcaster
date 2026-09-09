//! Timeline: keyframes com curvas, tracks tipadas, avaliacao por busca binaria.
//! Formulas e semantica identicas a `spellcaster/timeline/model.py`.

use crate::show::Show;
use crate::universe::Universes;

/// Controles padrao do bezier (mesmo `BEZ` do Python).
pub const BEZ: (f64, f64) = (0.42, 0.58);

/// Curva do segmento que CHEGA no keyframe ("chega em 255 com inout").
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Curve {
    Linear,
    Hold,
    In,
    Out,
    InOut,
    Bezier,
}

impl Curve {
    // O contrato do README fixa `from_str` (nao o trait FromStr, que devolve Result).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Curve {
        match s {
            "hold" => Curve::Hold,
            "in" => Curve::In,
            "out" => Curve::Out,
            "inout" => Curve::InOut,
            "bezier" => Curve::Bezier,
            _ => Curve::Linear, // desconhecido -> linear (README)
        }
    }

    pub fn ease(self, u: f64, c: (f64, f64)) -> f64 {
        match self {
            Curve::Linear => u,
            Curve::Hold => 0.0,
            Curve::In => u * u,
            Curve::Out => u * (2.0 - u),
            Curve::InOut => u * u * (3.0 - 2.0 * u),
            // ponytail: bezier cubico com os dois controles no eixo do valor e x=u (nao resolve
            // x(u) por Newton) ; trocar por cubic-bezier CSS quando a GUI arrastar as alcas no tempo.
            Curve::Bezier => {
                3.0 * (1.0 - u).powi(2) * u * c.0 + 3.0 * (1.0 - u) * u * u * c.1 + u * u * u
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    List(Vec<f64>),
    Text(String),
}

impl Value {
    fn push_to(&self, out: &mut Vec<f64>) -> bool {
        match self {
            Value::Num(v) => {
                out.push(*v);
                true
            }
            Value::List(v) => {
                out.extend_from_slice(v);
                true
            }
            // ponytail: texto nao vira DMX ; a R1 resolve texto em tracks OSC, que usam value().
            Value::Text(_) => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Keyframe {
    pub t: f64,
    pub value: Value,
    pub curve: Curve,
    pub c: (f64, f64),
}

/// Lista ordenada de Keyframe. Avaliacao por `partition_point` (= `bisect_right` do Python).
pub struct Keys {
    keys: Vec<Keyframe>,
    times: Vec<f64>,
}

impl Keys {
    pub fn new(mut keys: Vec<Keyframe>) -> Keys {
        keys.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
        let times = keys.iter().map(|k| k.t).collect();
        Keys { keys, times }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn keys(&self) -> &[Keyframe] {
        &self.keys
    }

    /// Indice do segmento: `bisect_right(times, t)`.
    #[inline]
    fn seek(&self, t: f64) -> usize {
        self.times.partition_point(|x| *x <= t)
    }

    /// Valor em degrau (texto, cue, media): o keyframe que vale em t. Sem alocacao.
    pub fn value(&self, t: f64) -> Option<&Value> {
        if self.keys.is_empty() {
            return None;
        }
        let i = self.seek(t);
        if i == 0 {
            return Some(&self.keys[0].value);
        }
        if i == self.keys.len() {
            return Some(&self.keys[i - 1].value);
        }
        let (k, n) = (&self.keys[i - 1], &self.keys[i]);
        // dt <= 0 (keyframe duplicado): o Python devolve o valor do keyframe seguinte.
        Some(if n.t - k.t <= 0.0 { &n.value } else { &k.value })
    }

    /// Valor interpolado em `out` (limpo antes). false = vazio ou nao numerico.
    pub fn eval(&self, t: f64, out: &mut Vec<f64>) -> bool {
        out.clear();
        if self.keys.is_empty() {
            return false;
        }
        let i = self.seek(t);
        if i == 0 {
            return self.keys[0].value.push_to(out);
        }
        let k = &self.keys[i - 1];
        if i == self.keys.len() {
            return k.value.push_to(out);
        }
        let n = &self.keys[i];
        let dt = n.t - k.t;
        if dt <= 0.0 {
            return n.value.push_to(out);
        }
        let e = n.curve.ease((t - k.t) / dt, n.c);
        match (&k.value, &n.value) {
            (Value::Num(a), Value::Num(b)) => {
                out.push(a + (b - a) * e);
                true
            }
            // zip do Python: para no menor dos dois (lista curta trunca a saida).
            (Value::List(a), Value::List(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    out.push(x + (y - x) * e);
                }
                true
            }
            // tipos misturados: o `lerp` do Python segura o valor anterior.
            _ => k.value.push_to(out),
        }
    }

    /// Keyframes com t0 < t <= t1 (disparo por borda).
    pub fn crossed(&self, t0: f64, t1: f64) -> &[Keyframe] {
        if t1 <= t0 {
            return &[];
        }
        &self.keys[self.seek(t0)..self.seek(t1)]
    }
}

pub struct Track {
    pub kind: String,
    pub universe: u16,
    pub address: u16,
    pub keys: Keys,
    /// Endereco de texto dos tracks `osc` e `media` ("/spell/dimmer").
    pub text_address: String,
    /// `media`: numero do clipe (canal 2 do Capture).
    pub clip: u16,
    /// `media`: reprodutor Capture (ausente ou "capture") — os outros viram track OSC.
    pub capture: bool,
    /// `osc`: valor padrao quando o track nao tem keys (o `args` do Python).
    pub args: Vec<f64>,
    buf: Vec<f64>,  // saida reaproveitada: zero alocacao por frame
    last: Vec<f64>, // ultimo valor numerico enviado (tracks osc/media)
    text: String,   // ultimo valor de texto enviado
    sent: bool,     // false = nunca enviado (o sentinela MISS do Python)
}

/// O que `Track::changed` achou de novo neste frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    /// Valor numerico novo em `nums()`.
    Nums,
    /// Valor de texto novo em `text()`.
    Text,
    /// Nada mudou desde o ultimo envio.
    Same,
}

impl Track {
    pub fn new(kind: String, universe: u16, address: u16, keys: Keys) -> Track {
        Track {
            kind,
            universe,
            address,
            keys,
            text_address: String::new(),
            clip: 0,
            capture: true,
            args: Vec::new(),
            buf: Vec::new(), // o clear()+push do primeiro frame aloca uma vez
            last: Vec::new(),
            text: String::new(),
            sent: false,
        }
    }

    fn parse(spec: &serde_json::Value) -> Result<Track, String> {
        let kind = spec
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "track sem \"type\"".to_string())?
            .to_string();
        let universe = spec.get("universe").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
        // OSC usa address de texto; o numerico fica em 0 e o resolvedor DMX o descarta.
        let adr = spec.get("address");
        let address = adr.and_then(|v| v.as_f64()).unwrap_or(1.0) as u16;
        let raw = spec.get("keys").and_then(|v| v.as_array());
        let mut ks = Vec::with_capacity(raw.map(|r| r.len()).unwrap_or(0));
        for k in raw.into_iter().flatten() {
            ks.push(parse_key(k)?);
        }
        let mut tr = Track::new(kind, universe, address, Keys::new(ks));
        tr.text_address = adr.and_then(|v| v.as_str()).unwrap_or("").to_string();
        tr.clip = spec.get("clip").and_then(|v| v.as_f64()).unwrap_or(0.0) as u16;
        tr.capture = spec
            .get("player")
            .and_then(|v| v.as_str())
            .unwrap_or("capture")
            == "capture";
        tr.args = match spec.get("args") {
            Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| x.as_f64()).collect(),
            Some(serde_json::Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0)],
            _ => Vec::new(),
        };
        Ok(tr)
    }

    /// Ultimo valor numerico avaliado (valido depois de `changed() == Side::Nums`).
    pub fn nums(&self) -> &[f64] {
        &self.buf
    }

    /// Ultimo valor de texto avaliado (valido depois de `changed() == Side::Text`).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Avalia um track de efeito colateral (`osc`, `media` nao-Capture) e diz se o valor MUDOU
    /// desde o ultimo envio — o `v != tr.last` do `_side` do Python. Sem alocacao por frame.
    pub fn changed(&mut self, t: f64) -> Side {
        if self.keys.is_empty() {
            // sem keys: o "args" do spec e' o default do value() do Python, enviado uma vez
            if self.sent || self.args.is_empty() {
                return Side::Same;
            }
            self.buf.clear();
            self.buf.extend_from_slice(&self.args);
            self.sent = true;
            return Side::Nums;
        }
        if self.keys.eval(t, &mut self.buf) {
            if self.sent && self.buf == self.last {
                return Side::Same;
            }
            self.last.clear();
            self.last.extend_from_slice(&self.buf);
            self.sent = true;
            return Side::Nums;
        }
        match self.keys.value(t) {
            Some(Value::Text(s)) => {
                if self.sent && self.text == *s {
                    return Side::Same;
                }
                self.text.clear();
                self.text.push_str(s);
                self.sent = true;
                Side::Text
            }
            _ => Side::Same,
        }
    }

    /// Track `media` do Capture: ch1 = 10 play / 20 replay / 28 stop, ch2 = clipe.
    /// None = track sem keyframe (o `if v is not None` do `_media` do Python).
    pub fn media_frame(&self, t: f64) -> Option<[f64; 2]> {
        let ch1 = match self.keys.value(t)? {
            Value::Text(s) => match s.as_str() {
                "play" => 10.0,
                "replay" => 20.0,
                "stop" => 28.0,
                _ => 0.0,
            },
            _ => 0.0,
        };
        Some([ch1, self.clip as f64])
    }
}

// ponytail: keyframe so' na forma de lista [t, valor, curva?, [c0, c1]?], que e' a unica que o
// .spell v1 grava ; aceitar objeto {"t":..,"v":..} quando a GUI passar a escrever assim.
fn parse_key(k: &serde_json::Value) -> Result<Keyframe, String> {
    let a = k
        .as_array()
        .ok_or_else(|| format!("keyframe nao e' lista: {}", k))?;
    let t = a
        .first()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("keyframe sem tempo: {}", k))?;
    let value = match a.get(1) {
        Some(serde_json::Value::Number(n)) => Value::Num(n.as_f64().unwrap_or(0.0)),
        Some(serde_json::Value::String(s)) => Value::Text(s.clone()),
        Some(serde_json::Value::Array(v)) => {
            Value::List(v.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect())
        }
        Some(serde_json::Value::Bool(b)) => Value::Num(if *b { 1.0 } else { 0.0 }),
        _ => Value::Num(0.0),
    };
    let curve = a
        .get(2)
        .and_then(|v| v.as_str())
        .map(Curve::from_str)
        .unwrap_or(Curve::Linear);
    let c = match a.get(3).and_then(|v| v.as_array()) {
        Some(v) if v.len() >= 2 => (
            v[0].as_f64().unwrap_or(BEZ.0),
            v[1].as_f64().unwrap_or(BEZ.1),
        ),
        _ => BEZ,
    };
    Ok(Keyframe { t, value, curve, c })
}

/// Tipos que a R1 resolve (o resto e' ignorado com aviso; `fixture` e' reservado).
fn resolvido(kind: &str) -> bool {
    matches!(kind, "dmx" | "artnet" | "osc" | "media" | "cue" | "fx")
}

/// Tracks de um show, ja separados por destino. `apply()` escreve os tracks DMX nos Universes;
/// as outras listas sao indices em `tracks`, consumidos pelo Player (o `Timeline` do Python).
pub struct Timeline {
    pub fps: u32,
    pub duration: Option<f64>,
    pub tracks: Vec<Track>,
    /// "dmx" e "artnet" — o que `apply()` percorre.
    pub dmx: Vec<usize>,
    /// "osc" — envia por OSC quando o valor muda.
    pub osc: Vec<usize>,
    /// "media" — Capture escreve DMX; os outros reprodutores viram OSC.
    pub media: Vec<usize>,
    /// "cue" — `crossed(prev, t)` dispara a cue.
    pub cue: Vec<usize>,
}

impl Timeline {
    pub fn new(show: &Show) -> Result<Timeline, String> {
        let mut tracks = Vec::with_capacity(show.tracks.len());
        for s in &show.tracks {
            tracks.push(Track::parse(s)?);
        }
        let idx = |k: &[&str]| -> Vec<usize> {
            tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| k.contains(&t.kind.as_str()))
                .map(|(i, _)| i)
                .collect()
        };
        Ok(Timeline {
            fps: show.fps,
            duration: show.duration,
            dmx: idx(&["dmx", "artnet"]),
            osc: idx(&["osc"]),
            media: idx(&["media"]),
            cue: idx(&["cue"]),
            tracks,
        })
    }

    /// Escreve os tracks "dmx" e "artnet" nos Universes no instante t. Zero alocacao por frame.
    pub fn apply(&mut self, universes: &mut Universes, t: f64) {
        let Timeline { tracks, dmx, .. } = self;
        for &i in dmx.iter() {
            let tr = &mut tracks[i];
            if tr.keys.eval(t, &mut tr.buf) {
                universes
                    .get_or_create(tr.universe)
                    .set(tr.address, &tr.buf);
            }
        }
    }

    /// Tipos de track presentes que a R1 nao resolve (para o aviso da CLI e do Player):
    /// `fixture` (reservado), `pyfx` e `laser` (Python), alem de tipo desconhecido.
    pub fn ignored(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .tracks
            .iter()
            .map(|t| t.kind.as_str())
            .filter(|k| !resolvido(k))
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kf(t: f64, v: f64, c: &str) -> Keyframe {
        Keyframe {
            t,
            value: Value::Num(v),
            curve: Curve::from_str(c),
            c: BEZ,
        }
    }

    #[test]
    fn curvas_valores_na_mao() {
        let us = [0.0, 0.25, 0.5, 0.75, 1.0];
        // linear: u
        let lin = [0.0, 0.25, 0.5, 0.75, 1.0];
        // hold: 0
        let hold = [0.0, 0.0, 0.0, 0.0, 0.0];
        // in: u*u
        let ein = [0.0, 0.0625, 0.25, 0.5625, 1.0];
        // out: u*(2-u)
        let eout = [0.0, 0.4375, 0.75, 0.9375, 1.0];
        // inout: u*u*(3-2u)
        let einout = [0.0, 0.15625, 0.5, 0.84375, 1.0];
        // bezier c=(0.42,0.58): 3(1-u)^2 u c0 + 3(1-u)u^2 c1 + u^3
        let ebez = [
            0.0,
            3.0 * 0.5625 * 0.25 * 0.42 + 3.0 * 0.75 * 0.0625 * 0.58 + 0.015625,
            3.0 * 0.25 * 0.5 * 0.42 + 3.0 * 0.5 * 0.25 * 0.58 + 0.125,
            3.0 * 0.0625 * 0.75 * 0.42 + 3.0 * 0.25 * 0.5625 * 0.58 + 0.421875,
            1.0,
        ];
        let cases: [(Curve, [f64; 5]); 6] = [
            (Curve::Linear, lin),
            (Curve::Hold, hold),
            (Curve::In, ein),
            (Curve::Out, eout),
            (Curve::InOut, einout),
            (Curve::Bezier, ebez),
        ];
        for (curve, want) in cases {
            for (i, u) in us.iter().enumerate() {
                let got = curve.ease(*u, BEZ);
                assert!(
                    (got - want[i]).abs() < 1e-12,
                    "curva {:?} em u={} deu {} e nao {}",
                    curve,
                    u,
                    got,
                    want[i]
                );
            }
        }
        assert_eq!(Curve::from_str("nao_existe"), Curve::Linear);
    }

    #[test]
    fn curva_e_a_do_keyframe_de_chegada() {
        // segmento 0..1 chega com "in": em t=0.5 vale 0.25 * 100
        let k = Keys::new(vec![kf(0.0, 0.0, "linear"), kf(1.0, 100.0, "in")]);
        let mut out = Vec::new();
        assert!(k.eval(0.5, &mut out));
        assert!((out[0] - 25.0).abs() < 1e-12, "{:?}", out);
        // invertendo as curvas o resultado muda: a curva do primeiro keyframe nao conta
        let k2 = Keys::new(vec![kf(0.0, 0.0, "in"), kf(1.0, 100.0, "linear")]);
        assert!(k2.eval(0.5, &mut out));
        assert!((out[0] - 50.0).abs() < 1e-12, "{:?}", out);
    }

    #[test]
    fn bordas_e_keyframe_duplicado() {
        let mut out = Vec::new();
        let vazio = Keys::new(vec![]);
        assert!(!vazio.eval(0.0, &mut out));
        assert!(vazio.value(0.0).is_none());

        let k = Keys::new(vec![kf(1.0, 10.0, "linear"), kf(2.0, 20.0, "linear")]);
        // antes do primeiro: valor do primeiro
        assert!(k.eval(0.0, &mut out));
        assert_eq!(out, vec![10.0]);
        // exatamente no primeiro
        assert!(k.eval(1.0, &mut out));
        assert_eq!(out, vec![10.0]);
        // depois do ultimo: valor do ultimo
        assert!(k.eval(9.0, &mut out));
        assert_eq!(out, vec![20.0]);
        assert!(k.eval(2.0, &mut out));
        assert_eq!(out, vec![20.0]);
        assert_eq!(k.value(0.0), Some(&Value::Num(10.0)));
        assert_eq!(k.value(1.5), Some(&Value::Num(10.0)));
        assert_eq!(k.value(9.0), Some(&Value::Num(20.0)));

        // duplicado (dt <= 0): vale o keyframe seguinte
        let d = Keys::new(vec![
            kf(1.0, 10.0, "linear"),
            kf(1.0, 200.0, "linear"),
            kf(3.0, 0.0, "linear"),
        ]);
        assert!(d.eval(0.5, &mut out));
        assert_eq!(out, vec![10.0]);
        assert_eq!(d.value(1.0), Some(&Value::Num(200.0)));
    }

    #[test]
    fn listas_e_texto() {
        let mut out = Vec::new();
        let k = Keys::new(vec![
            Keyframe {
                t: 0.0,
                value: Value::List(vec![0.0, 100.0]),
                curve: Curve::Linear,
                c: BEZ,
            },
            Keyframe {
                t: 1.0,
                value: Value::List(vec![100.0, 0.0]),
                curve: Curve::Linear,
                c: BEZ,
            },
        ]);
        assert!(k.eval(0.5, &mut out));
        assert_eq!(out, vec![50.0, 50.0]);

        let txt = Keys::new(vec![
            Keyframe {
                t: 0.0,
                value: Value::Text("a".into()),
                curve: Curve::Linear,
                c: BEZ,
            },
            Keyframe {
                t: 1.0,
                value: Value::Text("b".into()),
                curve: Curve::Linear,
                c: BEZ,
            },
        ]);
        assert!(!txt.eval(0.5, &mut out), "texto nao vira DMX");
        assert_eq!(txt.value(0.5), Some(&Value::Text("a".into())));
        assert_eq!(txt.value(1.0), Some(&Value::Text("b".into())));
    }

    /// Os tipos de track da R1: listas separadas, `fx` com script, `fixture` reservado.
    #[test]
    fn tipos_de_track_da_r1() {
        let sh: crate::show::Show = serde_json::from_str(
            r#"{"version":1,"fps":30,"tracks":[
                 {"type":"dmx","universe":1,"address":1,"keys":[[0,10]]},
                 {"type":"artnet","universe":2,"address":1,"keys":[[0,20]]},
                 {"type":"osc","address":"/spell/dim","keys":[[0,0.5]]},
                 {"type":"media","universe":1,"address":30,"clip":4,"keys":[[0,"play"],[1,"stop"]]},
                 {"type":"media","player":"resolume","address":"/clip/","keys":[[0,"play"]]},
                 {"type":"cue","keys":[[1,"GO"],[2,0]]},
                 {"type":"fx","script":"medgrupo.rhai","universe":3},
                 {"type":"fixture","universe":9},
                 {"type":"pyfx","file":"medgrupo.py"}]}"#,
        )
        .unwrap();
        let mut tl = Timeline::new(&sh).unwrap();
        assert_eq!(tl.tracks.len(), 9);
        assert_eq!(tl.dmx, vec![0, 1]);
        assert_eq!(tl.osc, vec![2]);
        assert_eq!(tl.media, vec![3, 4]);
        assert_eq!(tl.cue, vec![5]);
        assert_eq!(
            tl.ignored(),
            vec!["fixture", "pyfx"],
            "reservado e Python: aviso"
        );
        assert_eq!(tl.tracks[6].universe, 3);
        assert_eq!(tl.tracks[2].text_address, "/spell/dim");
        assert!(tl.tracks[3].capture && !tl.tracks[4].capture);

        // apply so' escreve dmx/artnet
        let mut uni = Universes::new();
        tl.apply(&mut uni, 0.0);
        assert_eq!(uni.numbers(), vec![1, 2]);
        assert_eq!(uni.get(1).unwrap().data[0], 10);
        assert_eq!(uni.get(1).unwrap().data[29], 0, "media nao entra no apply");

        // media Capture: ch1 = 10 play / 28 stop, ch2 = clipe
        assert_eq!(tl.tracks[3].media_frame(0.0), Some([10.0, 4.0]));
        assert_eq!(tl.tracks[3].media_frame(1.5), Some([28.0, 4.0]));
        assert_eq!(tl.tracks[6].media_frame(0.0), None, "track sem keys");
    }

    #[test]
    fn changed_so_avisa_na_mudanca() {
        let mut tr = Track::new(
            "osc".into(),
            1,
            0,
            Keys::new(vec![kf(0.0, 0.0, "linear"), kf(1.0, 100.0, "linear")]),
        );
        assert_eq!(
            tr.changed(0.0),
            Side::Nums,
            "primeira avaliacao sempre envia"
        );
        assert_eq!(tr.nums(), &[0.0]);
        assert_eq!(tr.changed(0.0), Side::Same);
        assert_eq!(tr.changed(0.5), Side::Nums);
        assert_eq!(
            tr.nums(),
            &[50.0],
            "valor interpolado, como o value() do Python"
        );
        assert_eq!(tr.changed(2.0), Side::Nums);
        assert_eq!(
            tr.changed(3.0),
            Side::Same,
            "depois do ultimo keyframe segura o valor"
        );

        let mut txt = Track::new(
            "media".into(),
            1,
            0,
            Keys::new(vec![
                Keyframe {
                    t: 0.0,
                    value: Value::Text("play".into()),
                    curve: Curve::Linear,
                    c: BEZ,
                },
                Keyframe {
                    t: 1.0,
                    value: Value::Text("stop".into()),
                    curve: Curve::Linear,
                    c: BEZ,
                },
            ]),
        );
        assert_eq!(txt.changed(0.0), Side::Text);
        assert_eq!(txt.text(), "play");
        assert_eq!(txt.changed(0.5), Side::Same);
        assert_eq!(txt.changed(1.0), Side::Text);
        assert_eq!(txt.text(), "stop");

        // sem keys, o "args" do spec vai uma vez so'
        let mut arg = Track::new("osc".into(), 1, 0, Keys::new(vec![]));
        arg.args = vec![1.0, 2.0];
        assert_eq!(arg.changed(0.0), Side::Nums);
        assert_eq!(arg.nums(), &[1.0, 2.0]);
        assert_eq!(arg.changed(1.0), Side::Same);
    }

    #[test]
    fn crossed_por_borda() {
        let k = Keys::new(vec![
            kf(0.0, 0.0, "linear"),
            kf(1.0, 1.0, "linear"),
            kf(2.0, 2.0, "linear"),
        ]);
        assert_eq!(k.crossed(0.0, 1.0).len(), 1);
        assert_eq!(k.crossed(0.0, 2.0).len(), 2);
        assert_eq!(k.crossed(1.0, 1.0).len(), 0);
        assert_eq!(k.crossed(2.0, 5.0).len(), 0);
    }
}
