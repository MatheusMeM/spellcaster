//! DMX input of the show: `"inputs": [{"type":"sacn","universe":1}, {"type":"artnet","universe":2}]`.
//!
//! The Player opens the inputs together with the outputs and keeps the last frame received on
//! each universe. The readers are the `input_get` command, the `serve` monitor (topic 2) and the
//! recording (`rec.rs`). There is no merge with the output: the input is a parallel buffer, HTP
//! passthrough lands when the owner asks for it (ROADMAP, "Art-Net/sACN input with merge").

use crate::show::Show;
use protocols::artnet::ArtNetIn;
use protocols::sacn::SacnIn;

/// One declared input: universe + where it comes from.
struct Fonte {
    universe: u16,
    artnet: bool,
}

/// The open inputs of a show. `get` returns the last frame received on the universe.
// ponytail: one SacnIn and one ArtNetIn per show (both already listen to every universe that
// reaches the socket) ; split them per interface when two consoles send the same universe.
#[derive(Default)]
pub struct Inputs {
    fontes: Vec<Fonte>,
    sacn: Option<SacnIn>,
    artnet: Option<ArtNetIn>,
}

impl Inputs {
    /// Opens what `show.inputs` declares. An unknown type becomes a warning, not an error (same
    /// as the outputs). With no `inputs`, it returns the empty set and opens no socket at all.
    pub fn open(sh: &Show) -> Result<Inputs, String> {
        let mut fontes = Vec::new();
        for v in sh
            .extra
            .get("inputs")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            let tipo = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let universe = v.get("universe").and_then(|u| u.as_u64()).unwrap_or(1) as u16;
            match tipo {
                "sacn" => fontes.push(Fonte {
                    universe,
                    artnet: false,
                }),
                "artnet" => fontes.push(Fonte {
                    universe,
                    artnet: true,
                }),
                _ => eprintln!("warning: input \"{}\" ignored", tipo),
            }
        }
        let us: Vec<u16> = fontes
            .iter()
            .filter(|f| !f.artnet)
            .map(|f| f.universe)
            .collect();
        let sacn = if us.is_empty() {
            None
        } else {
            Some(SacnIn::new(&us).map_err(|e| format!("sacn input: {}", e))?)
        };
        let artnet = if fontes.iter().any(|f| f.artnet) {
            Some(ArtNetIn::new().map_err(|e| format!("artnet input: {}", e))?)
        } else {
            None
        };
        Ok(Inputs {
            fontes,
            sacn,
            artnet,
        })
    }

    /// Last frame received on the universe, or `None` (universe not declared or nothing arrived).
    pub fn get(&self, universe: u16) -> Option<[u8; 512]> {
        let f = self.fontes.iter().find(|f| f.universe == universe)?;
        if f.artnet {
            self.artnet.as_ref()?.get(universe)
        } else {
            self.sacn.as_ref()?.get(universe)
        }
    }

    /// Declared universes, in show order.
    pub fn universes(&self) -> Vec<u16> {
        self.fontes.iter().map(|f| f.universe).collect()
    }

    /// (universe, frame) for each input that has already received something — the `serve` monitor.
    pub fn frames(&self) -> Vec<(u16, [u8; 512])> {
        self.fontes
            .iter()
            .filter_map(|f| self.get(f.universe).map(|d| (f.universe, d)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_inputs_opens_no_socket() {
        let sh: Show = serde_json::from_str(r#"{"version":1}"#).unwrap();
        let i = Inputs::open(&sh).unwrap();
        assert!(i.universes().is_empty());
        assert_eq!(i.get(1), None);
        assert!(i.frames().is_empty());
    }

    /// An unknown type does not take the show down; its universe does not exist for `get`.
    #[test]
    fn unknown_type_is_ignored() {
        let sh: Show =
            serde_json::from_str(r#"{"version":1,"inputs":[{"type":"midi","universe":9}]}"#)
                .unwrap();
        let i = Inputs::open(&sh).unwrap();
        assert!(i.universes().is_empty());
        assert_eq!(i.get(9), None);
    }
}
