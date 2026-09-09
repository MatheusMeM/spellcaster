//! Entrada DMX do show: `"inputs": [{"type":"sacn","universe":1}, {"type":"artnet","universe":2}]`.
//!
//! O Player abre as entradas junto com as saidas e guarda o ultimo frame recebido de cada
//! universo. Quem le e' o comando `input_get`, o monitor do `serve` (topico 2) e a gravacao
//! (`rec.rs`). Nao ha merge com a saida: a entrada e' um buffer paralelo, o passthrough HTP
//! entra quando o dono pedir (ROADMAP, "entrada Art-Net/sACN com merge").

use crate::show::Show;
use protocols::artnet::ArtNetIn;
use protocols::sacn::SacnIn;

/// Uma entrada declarada: universo + de onde ele vem.
struct Fonte {
    universe: u16,
    artnet: bool,
}

/// As entradas abertas de um show. `get` devolve o ultimo frame recebido do universo.
// ponytail: um SacnIn e um ArtNetIn por show (os dois ja' escutam todos os universos que
// chegam no socket) ; separar por interface quando duas mesas mandarem o mesmo universo.
#[derive(Default)]
pub struct Inputs {
    fontes: Vec<Fonte>,
    sacn: Option<SacnIn>,
    artnet: Option<ArtNetIn>,
}

impl Inputs {
    /// Abre o que `show.inputs` declara. Tipo desconhecido vira aviso, nao erro (igual as
    /// saidas). Sem `inputs`, devolve o vazio e nao abre socket nenhum.
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
                _ => eprintln!("aviso: entrada \"{}\" ignorada", tipo),
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
            Some(SacnIn::new(&us).map_err(|e| format!("entrada sacn: {}", e))?)
        };
        let artnet = if fontes.iter().any(|f| f.artnet) {
            Some(ArtNetIn::new().map_err(|e| format!("entrada artnet: {}", e))?)
        } else {
            None
        };
        Ok(Inputs {
            fontes,
            sacn,
            artnet,
        })
    }

    /// Ultimo frame recebido no universo, ou `None` (universo nao declarado ou nada chegou).
    pub fn get(&self, universe: u16) -> Option<[u8; 512]> {
        let f = self.fontes.iter().find(|f| f.universe == universe)?;
        if f.artnet {
            self.artnet.as_ref()?.get(universe)
        } else {
            self.sacn.as_ref()?.get(universe)
        }
    }

    /// Universos declarados, na ordem do show.
    pub fn universes(&self) -> Vec<u16> {
        self.fontes.iter().map(|f| f.universe).collect()
    }

    /// (universo, frame) de cada entrada que ja' recebeu algo — o monitor do `serve`.
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
    fn sem_inputs_nao_abre_socket() {
        let sh: Show = serde_json::from_str(r#"{"version":1}"#).unwrap();
        let i = Inputs::open(&sh).unwrap();
        assert!(i.universes().is_empty());
        assert_eq!(i.get(1), None);
        assert!(i.frames().is_empty());
    }

    /// Tipo desconhecido nao derruba o show; o universo dele nao existe para o `get`.
    #[test]
    fn tipo_desconhecido_e_ignorado() {
        let sh: Show =
            serde_json::from_str(r#"{"version":1,"inputs":[{"type":"midi","universe":9}]}"#)
                .unwrap();
        let i = Inputs::open(&sh).unwrap();
        assert!(i.universes().is_empty());
        assert_eq!(i.get(9), None);
    }
}
