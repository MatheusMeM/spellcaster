//! Cue list: manual GO, `wait` before it starts, automatic `follow` when it ends, linear fade
//! between snapshots. Port of `spellcaster/timeline/cues.py`.
//!
//! Snapshot = {"universe/address": [values]} in the .spell; becomes `((universe, address), values)`.
//! The current state is written into the Universes by `update()`, with no per-frame allocation:
//! the value `Vec`s are reused (only the first touch on each key allocates).

use crate::universe::Universes;
use serde_json::Value;

type Key = (u16, u16);
type Snap = Vec<(Key, Vec<f64>)>;

/// `"1/100"` -> `(1, 100)`; `"100"` -> `(1, 100)`.
// ponytail: a malformed key returns None and the Cue drops it with a warning (Python raises
// ValueError) ; make it a load error once the .spell has schema validation.
pub fn key(s: &str) -> Option<Key> {
    match s.split_once('/') {
        Some((u, a)) => Some((u.trim().parse().ok()?, a.trim().parse().ok()?)),
        None => Some((1, s.trim().parse().ok()?)),
    }
}

pub struct Cue {
    pub name: String,
    pub fade: f64,
    /// Delay between the trigger and the start of the fade.
    pub wait: f64,
    /// When it ends, it fires the next one.
    pub follow: bool,
    pub values: Vec<(Key, Vec<f64>)>,
}

impl Cue {
    pub fn new(spec: &Value) -> Cue {
        let num = |k: &str| spec.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let mut values = Vec::new();
        // ponytail: key order is the serde_json Map order (alphabetical), not the file order
        // ; it only changes the result if two keys of the SAME cue write to the same channel
        // ; turn on the serde_json "preserve_order" feature if some show comes to depend on it.
        if let Some(m) = spec.get("values").and_then(|v| v.as_object()) {
            for (k, v) in m {
                match key(k) {
                    Some(kk) => values.push((
                        kk,
                        match v {
                            Value::Array(a) => {
                                a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()
                            }
                            other => vec![other.as_f64().unwrap_or(0.0)],
                        },
                    )),
                    None => eprintln!("warning: cue with invalid key: {}", k),
                }
            }
        }
        Cue {
            name: spec
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            fade: num("fade"),
            wait: num("wait"),
            follow: spec
                .get("follow")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            values,
        }
    }
}

pub struct CueList {
    cues: Vec<Cue>,
    state: Snap,
    from: Snap,
    index: i32,
    cur: Option<usize>,
    t0: f64,
    /// (index, trigger instant) — becomes the current cue once the `wait` is over.
    pending: Option<(usize, f64)>,
}

impl CueList {
    pub fn new(specs: &[Value]) -> CueList {
        CueList {
            cues: specs.iter().map(Cue::new).collect(),
            state: Vec::new(),
            from: Vec::new(),
            index: -1,
            cur: None,
            t0: 0.0,
            pending: None,
        }
    }

    pub fn len(&self) -> usize {
        self.cues.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    /// Index of the last cue fired; -1 = none.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Fires the next cue (or the one at the given index). The fade starts after its `wait`.
    pub fn go(&mut self, t: f64, index: Option<usize>) -> bool {
        let i = match index {
            Some(i) => i as i64,
            None => self.index as i64 + 1,
        };
        if i < 0 || i >= self.cues.len() as i64 {
            return false;
        }
        self.pending = Some((i as usize, t));
        true
    }

    /// Advances the fade and writes the current snapshot into the Universes.
    pub fn update(&mut self, t: f64, uni: &mut Universes) {
        if let Some((i, t0)) = self.pending {
            if t >= t0 + self.cues[i].wait {
                self.pending = None;
                self.index = i as i32;
                self.cur = Some(i);
                self.t0 = t;
                self.from.clone_from(&self.state); // starting point of the fade
            }
        }
        if let Some(i) = self.cur {
            let CueList {
                cues,
                state,
                from,
                t0,
                ..
            } = self;
            let c = &cues[i];
            let u = if c.fade <= 0.0 {
                1.0
            } else {
                ((t - *t0) / c.fade).clamp(0.0, 1.0)
            };
            for (k, v) in &c.values {
                // starting value: whatever the state held, padded with 0.0 (same as Python)
                let a = from
                    .iter()
                    .find(|(kk, _)| kk == k)
                    .map(|(_, x)| x.as_slice())
                    .unwrap_or(&[]);
                let dst = slot(state, *k);
                dst.clear();
                if u >= 1.0 {
                    dst.extend_from_slice(v);
                } else {
                    for (j, y) in v.iter().enumerate() {
                        let x = a.get(j).copied().unwrap_or(0.0);
                        dst.push(x + (y - x) * u);
                    }
                }
            }
            if u >= 1.0 {
                let follow = c.follow;
                self.cur = None;
                if follow {
                    self.go(t, None);
                }
            }
        }
        for ((u, a), v) in &self.state {
            uni.get_or_create(*u).set(*a, v);
        }
    }

    pub fn reset(&mut self) {
        self.state.clear();
        self.from.clear();
        self.index = -1;
        self.cur = None;
        self.pending = None;
    }
}

/// The `snap` entry for the key, appending it at the end if it does not exist yet.
// ponytail: linear search ; a cue has dozens of keys, not thousands — make it a sorted Vec with
// binary search if a show comes to have hundreds of channels per cue.
fn slot(snap: &mut Snap, k: Key) -> &mut Vec<f64> {
    match snap.iter().position(|(kk, _)| *kk == k) {
        Some(i) => &mut snap[i].1,
        None => {
            snap.push((k, Vec::new()));
            &mut snap.last_mut().expect("just pushed").1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_in_both_forms() {
        assert_eq!(key("1/100"), Some((1, 100)));
        assert_eq!(key("100"), Some((1, 100)));
        assert_eq!(key("3/512"), Some((3, 512)));
        assert_eq!(key(" 2 / 7 "), Some((2, 7)));
        assert_eq!(key("x"), None);
        assert_eq!(key("1/x"), None);
    }

    fn ch(uni: &Universes, u: u16, a: u16) -> u8 {
        uni.get(u).expect("universe").data[(a - 1) as usize]
    }

    /// wait + fade + follow with the values worked out by hand.
    #[test]
    fn wait_fade_follow() {
        let specs = vec![
            json!({"name": "a", "wait": 1.0, "fade": 2.0, "follow": true,
                   "values": {"1/1": [100]}}),
            json!({"name": "b", "fade": 0.0, "values": {"1/1": [200], "2/5": [10, 20]}}),
        ];
        let mut cl = CueList::new(&specs);
        let mut uni = Universes::new();
        uni.get_or_create(1);
        assert_eq!(cl.len(), 2);
        assert_eq!(cl.index(), -1);

        // with no GO nothing happens
        cl.update(0.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 0);

        assert!(cl.go(0.0, None));
        // inside the wait: it has not started yet
        cl.update(0.5, &mut uni);
        assert_eq!(cl.index(), -1);
        assert_eq!(ch(&uni, 1, 1), 0);

        // t = 1.0: the wait is over, the fade starts here (u = 0)
        cl.update(1.0, &mut uni);
        assert_eq!(cl.index(), 0);
        assert_eq!(ch(&uni, 1, 1), 0);
        // middle of the 2 s fade
        cl.update(2.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 50);
        cl.update(2.5, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 75);
        // end of the fade: full value and the follow arms the next one
        cl.update(3.0, &mut uni);
        assert_eq!(ch(&uni, 1, 1), 100);
        assert_eq!(
            cl.index(),
            0,
            "the next one only lands on the following update"
        );
        // cue b: fade 0 => u = 1 right away
        cl.update(3.1, &mut uni);
        assert_eq!(cl.index(), 1);
        assert_eq!(ch(&uni, 1, 1), 200);
        assert_eq!(ch(&uni, 2, 5), 10);
        assert_eq!(ch(&uni, 2, 6), 20);
        // end of the list: the follow of the last one has nowhere to go
        cl.update(4.0, &mut uni);
        assert_eq!(cl.index(), 1);

        // GO with an explicit index goes back to cue 0
        assert!(cl.go(4.0, Some(0)));
        cl.update(5.0, &mut uni); // wait 1.0 served: u = 0, starting from 200
        assert_eq!(ch(&uni, 1, 1), 200);
        cl.update(6.0, &mut uni); // halfway from 200 to 100
        assert_eq!(ch(&uni, 1, 1), 150);

        assert!(!cl.go(6.0, Some(9)), "index outside the list");
        cl.reset();
        assert_eq!(cl.index(), -1);
        cl.update(7.0, &mut uni);
        assert_eq!(
            ch(&uni, 1, 1),
            150,
            "reset does not erase what was already written"
        );
    }

    #[test]
    fn zero_fade_and_bare_value() {
        let specs = vec![json!({"values": {"7": 255, "1/2": [1, 2, 3]}})];
        let mut cl = CueList::new(&specs);
        let mut uni = Universes::new();
        cl.go(0.0, None);
        cl.update(0.0, &mut uni);
        assert_eq!(ch(&uni, 1, 7), 255, "a key with no universe is universe 1");
        assert_eq!(ch(&uni, 1, 2), 1);
        assert_eq!(ch(&uni, 1, 4), 3);
        assert_eq!(cl.index(), 0);
    }

    #[test]
    fn empty_list_and_invalid_key() {
        let mut cl = CueList::new(&[]);
        let mut uni = Universes::new();
        assert!(cl.is_empty());
        assert!(!cl.go(0.0, None));
        cl.update(0.0, &mut uni);
        assert!(uni.is_empty());

        let cl = CueList::new(&[json!({"values": {"not/a/key": [1]}})]);
        assert_eq!(cl.len(), 1, "an invalid key does not take the cue down");
    }
}
