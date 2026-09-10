//! MIDI input. `midir` is RtMidi in Rust: winmm on Windows, ALSA on Linux, CoreMIDI on mac -
//! it is what makes "any keyboard or surface" work without a vendor driver.
//!
//! INPUT only and channel messages only (note on/off, CC, program change, pitch bend). MIDI
//! output, MTC/clock, MIDI Show Control and surface LED feedback are out of scope
//! (design/DECISOES.md).
//!
//! Textual key of the event (the same one the graph node `in.midi` uses): `"<status>/<data1>"` -
//! `144/60` = note on channel 1 note 60, `176/1` = CC 1 of channel 1. The channel is already
//! in the status, so a controller on another channel is another key, with no extra field.

#[cfg(not(target_env = "musl"))]
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::Receiver;

/// Queue between the driver callback thread and the consumer (the frame). When full, the NEW
/// event is dropped: the driver never blocks.
// ponytail: 256 events is depth to spare for one pump per frame (16 ms) ; it would only grow
// if someone sent large SysEx - which `Ignore::All` already discards first.
#[cfg(not(target_env = "musl"))]
const DEPTH: usize = 256;

/// Names of the input ports, in driver order. A machine with no MIDI (or no service) returns
/// an empty list: looking for a port is never an error.
#[cfg(target_env = "musl")]
pub fn ports() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_env = "musl"))]
pub fn ports() -> Vec<String> {
    let Ok(mi) = MidiInput::new("spellcaster") else {
        return Vec::new();
    };
    mi.ports()
        .iter()
        .map(|p| mi.port_name(p).unwrap_or_default())
        .collect()
}

/// One open MIDI input port. Closes on `Drop`.
pub struct MidiIn {
    name: String,
    rx: Receiver<(u8, u8, u8)>,
    /// The live connection: dropping it closes the port.
    #[cfg(not(target_env = "musl"))]
    _conn: MidiInputConnection<()>,
}

impl MidiIn {
    /// `port` = index as text ("0"), part of the name (case-insensitive) or empty = the first
    /// port.
    #[cfg(not(target_env = "musl"))]
    pub fn open(port: &str) -> Result<MidiIn, String> {
        let mut mi = MidiInput::new("spellcaster").map_err(|e| e.to_string())?;
        mi.ignore(Ignore::All); // sysex, clock and active sensing do not become events
        let ps = mi.ports();
        if ps.is_empty() {
            return Err("no MIDI input port".into());
        }
        let nomes: Vec<String> = ps
            .iter()
            .map(|p| mi.port_name(p).unwrap_or_default())
            .collect();
        let i = escolhe(&nomes, port)?;
        let name = nomes[i].clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(DEPTH);
        let conn = mi
            .connect(
                &ps[i],
                "spellcaster-in",
                move |_t, m, _| {
                    if let Some(e) = evento(m) {
                        let _ = tx.try_send(e);
                    }
                },
                (),
            )
            .map_err(|e| format!("{}: {}", name, e))?;
        Ok(MidiIn {
            name,
            rx,
            _conn: conn,
        })
    }

    #[cfg(target_env = "musl")]
    pub fn open(_port: &str) -> Result<MidiIn, String> {
        Err("MIDI unavailable in this binary (static, no ALSA)".into())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Next event from the queue, without blocking.
    pub fn try_recv(&self) -> Option<(u8, u8, u8)> {
        self.rx.try_recv().ok()
    }
}

/// Index of the requested port: empty = the first, a number = index, anything else = part of
/// the name.
#[cfg_attr(target_env = "musl", allow(dead_code))]
fn escolhe(nomes: &[String], port: &str) -> Result<usize, String> {
    let p = port.trim();
    if p.is_empty() {
        return Ok(0);
    }
    if let Ok(i) = p.parse::<usize>() {
        return if i < nomes.len() {
            Ok(i)
        } else {
            Err(format!("port {} does not exist ({} ports)", i, nomes.len()))
        };
    }
    let alvo = p.to_lowercase();
    nomes
        .iter()
        .position(|n| n.to_lowercase().contains(&alvo))
        .ok_or_else(|| format!("no MIDI port matches \"{}\"", port))
}

/// Raw message -> `(status, data1, data2)`, or `None` if it is not a channel message.
///
/// Note on with velocity 0 is the note off half the keyboards send: it becomes status 0x8n,
/// otherwise releasing the key would fire the same command as pressing it.
pub fn evento(m: &[u8]) -> Option<(u8, u8, u8)> {
    let &status = m.first()?;
    if !(0x80..0xF0).contains(&status) {
        return None; // system common / realtime: no channel, does not become a key
    }
    let d1 = m.get(1).copied().unwrap_or(0) & 0x7F;
    let d2 = m.get(2).copied().unwrap_or(0) & 0x7F;
    if status & 0xF0 == 0x90 && d2 == 0 {
        return Some((0x80 | (status & 0x0F), d1, 0));
    }
    Some((status, d1, d2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_event() {
        assert_eq!(evento(&[144, 60, 100]), Some((144, 60, 100)));
        assert_eq!(evento(&[176, 1, 64]), Some((176, 1, 64)));
        // program change and channel pressure have two bytes: data2 = 0
        assert_eq!(evento(&[192, 5]), Some((192, 5, 0)));
        // note on with velocity 0 = note off (otherwise releasing the key refires the command)
        assert_eq!(evento(&[145, 60, 0]), Some((129, 60, 0)));
        assert_eq!(evento(&[129, 60, 0]), Some((129, 60, 0)));
        // no channel: clock, start, sysex, and a stray data byte
        for m in [
            &[248u8][..],
            &[250][..],
            &[240, 1, 2][..],
            &[60, 1][..],
            &[][..],
        ] {
            assert_eq!(evento(m), None, "{:?}", m);
        }
    }

    #[test]
    fn chooses_by_index_and_by_name() {
        let n = vec!["MPK mini 3 0".to_string(), "nanoKONTROL2".to_string()];
        assert_eq!(escolhe(&n, ""), Ok(0));
        assert_eq!(escolhe(&n, "1"), Ok(1));
        assert_eq!(escolhe(&n, "nanokontrol"), Ok(1));
        assert_eq!(escolhe(&n, " MPK "), Ok(0));
        assert!(escolhe(&n, "9").is_err());
        assert!(escolhe(&n, "launchpad").is_err());
    }

    /// A machine with no device at all must not blow up: `ports()` returns an empty list.
    #[test]
    fn ports_never_fails() {
        let p = ports();
        // opening with no port is a text error, never a panic
        if p.is_empty() {
            assert!(MidiIn::open("").is_err());
        }
    }
}
