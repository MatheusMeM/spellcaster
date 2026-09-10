//! Entrada MIDI. `midir` e' a RtMidi em Rust: winmm no Windows, ALSA no Linux, CoreMIDI no mac —
//! e' o que faz "qualquer teclado ou surface" funcionar sem driver do fabricante.
//!
//! So' ENTRADA e so' mensagem de canal (note on/off, CC, program change, pitch bend). Saida MIDI,
//! MTC/clock, MIDI Show Control e feedback de LED de superficie estao fora (design/DECISOES.md).
//!
//! Chave textual do evento (a mesma do no' `in.midi` do graph): `"<status>/<data1>"` —
//! `144/60` = note on canal 1 nota 60, `176/1` = CC 1 do canal 1. O canal ja' esta' no status,
//! entao um controlador em outro canal e' outra chave, sem campo a mais.

#[cfg(not(target_env = "musl"))]
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::Receiver;

/// Fila entre a thread de callback do driver e quem consome (o frame). Cheia, o evento NOVO cai:
/// o driver nunca bloqueia.
// ponytail: 256 eventos e' fundo de sobra para um pump por frame (16 ms) ; so' subiria se alguem
// mandasse SysEx grande — que o `Ignore::All` ja' descarta antes.
const DEPTH: usize = 256;

/// Nomes das portas de entrada, na ordem do driver. Maquina sem MIDI (ou sem servico) devolve
/// lista vazia: procurar porta nunca e' erro.
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

/// Uma porta MIDI de entrada aberta. Fecha no `Drop`.
pub struct MidiIn {
    name: String,
    rx: Receiver<(u8, u8, u8)>,
    /// A conexao viva: solta-la fecha a porta.
    #[cfg(not(target_env = "musl"))]
    _conn: MidiInputConnection<()>,
}

#[cfg(target_env = "musl")]
impl MidiIn {
    pub fn open(_port: &str) -> Result<MidiIn, String> {
        Err("MIDI indisponivel neste binario (estatico, sem ALSA)".into())
    }
}

#[cfg(not(target_env = "musl"))]
impl MidiIn {
    /// `port` = indice em texto ("0"), trecho do nome (sem diferenca de caixa) ou vazio = a
    /// primeira porta.
    pub fn open(port: &str) -> Result<MidiIn, String> {
        let mut mi = MidiInput::new("spellcaster").map_err(|e| e.to_string())?;
        mi.ignore(Ignore::All); // sysex, clock e active sensing nao viram evento
        let ps = mi.ports();
        if ps.is_empty() {
            return Err("nenhuma porta MIDI de entrada".into());
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

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Proximo evento da fila, sem bloquear.
    pub fn try_recv(&self) -> Option<(u8, u8, u8)> {
        self.rx.try_recv().ok()
    }
}

/// Indice da porta pedida: vazio = a primeira, numero = indice, resto = trecho do nome.
fn escolhe(nomes: &[String], port: &str) -> Result<usize, String> {
    let p = port.trim();
    if p.is_empty() {
        return Ok(0);
    }
    if let Ok(i) = p.parse::<usize>() {
        return if i < nomes.len() {
            Ok(i)
        } else {
            Err(format!("porta {} nao existe ({} portas)", i, nomes.len()))
        };
    }
    let alvo = p.to_lowercase();
    nomes
        .iter()
        .position(|n| n.to_lowercase().contains(&alvo))
        .ok_or_else(|| format!("nenhuma porta MIDI casa com \"{}\"", port))
}

/// Mensagem crua -> `(status, data1, data2)`, ou `None` se nao for mensagem de canal.
///
/// Note on com velocidade 0 e' o note off que meio teclado manda: vira status 0x8n, senao
/// soltar a tecla dispararia o mesmo comando que aperta-la.
pub fn evento(m: &[u8]) -> Option<(u8, u8, u8)> {
    let &status = m.first()?;
    if !(0x80..0xF0).contains(&status) {
        return None; // system common / realtime: nao tem canal, nao vira chave
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
    fn evento_de_canal() {
        assert_eq!(evento(&[144, 60, 100]), Some((144, 60, 100)));
        assert_eq!(evento(&[176, 1, 64]), Some((176, 1, 64)));
        // program change e channel pressure tem dois bytes: data2 = 0
        assert_eq!(evento(&[192, 5]), Some((192, 5, 0)));
        // note on com velocidade 0 = note off (senao soltar a tecla redispara o comando)
        assert_eq!(evento(&[145, 60, 0]), Some((129, 60, 0)));
        assert_eq!(evento(&[129, 60, 0]), Some((129, 60, 0)));
        // sem canal: clock, start, sysex, e byte de dado solto
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
    fn escolhe_por_indice_e_por_nome() {
        let n = vec!["MPK mini 3 0".to_string(), "nanoKONTROL2".to_string()];
        assert_eq!(escolhe(&n, ""), Ok(0));
        assert_eq!(escolhe(&n, "1"), Ok(1));
        assert_eq!(escolhe(&n, "nanokontrol"), Ok(1));
        assert_eq!(escolhe(&n, " MPK "), Ok(0));
        assert!(escolhe(&n, "9").is_err());
        assert!(escolhe(&n, "launchpad").is_err());
    }

    /// Maquina sem dispositivo nenhum nao pode explodir: `ports()` devolve lista vazia.
    #[test]
    fn ports_nunca_falha() {
        let p = ports();
        // abrir sem porta e' erro de texto, nunca panico
        if p.is_empty() {
            assert!(MidiIn::open("").is_err());
        }
    }
}
