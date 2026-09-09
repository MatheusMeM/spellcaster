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
    buf: Vec<f64>, // saida reaproveitada: zero alocacao por frame
}

impl Track {
    pub fn new(kind: String, universe: u16, address: u16, keys: Keys) -> Track {
        let n = keys
            .keys()
            .first()
            .map(|k| match &k.value {
                Value::List(v) => v.len(),
                _ => 1,
            })
            .unwrap_or(1);
        Track {
            kind,
            universe,
            address,
            keys,
            buf: Vec::with_capacity(n),
        }
    }

    fn parse(spec: &serde_json::Value) -> Result<Track, String> {
        let kind = spec
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "track sem \"type\"".to_string())?
            .to_string();
        let universe = spec.get("universe").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
        // OSC usa address de texto: vira 0 e o track e' ignorado na R0 (igual ao Python).
        let address = spec.get("address").and_then(|v| v.as_f64()).unwrap_or(1.0) as u16;
        let raw = spec.get("keys").and_then(|v| v.as_array());
        let mut ks = Vec::with_capacity(raw.map(|r| r.len()).unwrap_or(0));
        for k in raw.into_iter().flatten() {
            ks.push(parse_key(k)?);
        }
        Ok(Track::new(kind, universe, address, Keys::new(ks)))
    }
}

// ponytail: keyframe so' na forma de lista [t, valor, curva?, [c0, c1]?], que e' a unica que o
// .spell v1 grava ; aceitar objeto {"t":..,"v":..} quando a GUI passar a escrever assim.
fn parse_key(k: &serde_json::Value) -> Result<Keyframe, String> {
    let a = k.as_array().ok_or_else(|| format!("keyframe nao e' lista: {}", k))?;
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

pub struct Timeline {
    pub fps: u32,
    pub duration: Option<f64>,
    pub tracks: Vec<Track>,
}

impl Timeline {
    pub fn new(show: &Show) -> Result<Timeline, String> {
        let mut tracks = Vec::with_capacity(show.tracks.len());
        for s in &show.tracks {
            tracks.push(Track::parse(s)?);
        }
        Ok(Timeline {
            fps: show.fps,
            duration: show.duration,
            tracks,
        })
    }

    /// Escreve os tracks "dmx" e "artnet" nos Universes no instante t. Zero alocacao por frame.
    // ponytail: pyfx / laser / media / osc / cue / fixture sao ignorados na R0 ; a R1 traz `fx`
    // em script e os resolvedores de laser e midia.
    pub fn apply(&mut self, universes: &mut Universes, t: f64) {
        for tr in &mut self.tracks {
            if tr.kind != "dmx" && tr.kind != "artnet" {
                continue;
            }
            if tr.keys.eval(t, &mut tr.buf) {
                universes.get_or_create(tr.universe).set(tr.address, &tr.buf);
            }
        }
    }

    /// Tipos de track presentes que a R0 nao resolve (para o aviso da CLI).
    pub fn ignored(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .tracks
            .iter()
            .map(|t| t.kind.as_str())
            .filter(|k| *k != "dmx" && *k != "artnet")
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
        let d = Keys::new(vec![kf(1.0, 10.0, "linear"), kf(1.0, 200.0, "linear"), kf(3.0, 0.0, "linear")]);
        assert!(d.eval(0.5, &mut out));
        assert_eq!(out, vec![10.0]);
        assert_eq!(d.value(1.0), Some(&Value::Num(200.0)));
    }

    #[test]
    fn listas_e_texto() {
        let mut out = Vec::new();
        let k = Keys::new(vec![
            Keyframe { t: 0.0, value: Value::List(vec![0.0, 100.0]), curve: Curve::Linear, c: BEZ },
            Keyframe { t: 1.0, value: Value::List(vec![100.0, 0.0]), curve: Curve::Linear, c: BEZ },
        ]);
        assert!(k.eval(0.5, &mut out));
        assert_eq!(out, vec![50.0, 50.0]);

        let txt = Keys::new(vec![
            Keyframe { t: 0.0, value: Value::Text("a".into()), curve: Curve::Linear, c: BEZ },
            Keyframe { t: 1.0, value: Value::Text("b".into()), curve: Curve::Linear, c: BEZ },
        ]);
        assert!(!txt.eval(0.5, &mut out), "texto nao vira DMX");
        assert_eq!(txt.value(0.5), Some(&Value::Text("a".into())));
        assert_eq!(txt.value(1.0), Some(&Value::Text("b".into())));
    }

    #[test]
    fn crossed_por_borda() {
        let k = Keys::new(vec![kf(0.0, 0.0, "linear"), kf(1.0, 1.0, "linear"), kf(2.0, 2.0, "linear")]);
        assert_eq!(k.crossed(0.0, 1.0).len(), 1);
        assert_eq!(k.crossed(0.0, 2.0).len(), 2);
        assert_eq!(k.crossed(1.0, 1.0).len(), 0);
        assert_eq!(k.crossed(2.0, 5.0).len(), 0);
    }
}
