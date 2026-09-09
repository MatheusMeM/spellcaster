//! Buffers DMX: um Universe por numero, 512 canais, endereco 1-based.
//! Igual a `spellcaster/core/universe.py`.

pub struct Universe {
    pub number: u16,
    pub data: [u8; 512],
}

impl Universe {
    pub fn new(number: u16) -> Universe {
        Universe {
            number,
            data: [0u8; 512],
        }
    }

    /// Escreve values a partir de addr (1-based); clamp 0..255; ignora o que passar de 512.
    /// `v as u8` no Rust trunca para zero e satura — mesmo resultado de `max(0, min(255, int(v)))`.
    pub fn set(&mut self, addr: u16, values: &[f64]) {
        // ponytail: addr 0 e' descartado ; o Python indexa com -1 e escreve lixo, nao vale copiar.
        if addr == 0 {
            return;
        }
        let i = (addr - 1) as usize;
        if i >= 512 {
            return;
        }
        let n = values.len().min(512 - i);
        for (d, v) in self.data[i..i + n].iter_mut().zip(values) {
            *d = *v as u8;
        }
    }

    pub fn set_bytes(&mut self, addr: u16, values: &[u8]) {
        if addr == 0 {
            return;
        }
        let i = (addr - 1) as usize;
        if i >= 512 {
            return;
        }
        let n = values.len().min(512 - i);
        self.data[i..i + n].copy_from_slice(&values[..n]);
    }
}

/// Vec ordenado por numero + busca binaria: zero alocacao depois do primeiro frame.
pub struct Universes {
    v: Vec<Universe>,
}

impl Default for Universes {
    fn default() -> Self {
        Universes::new()
    }
}

impl Universes {
    pub fn new() -> Universes {
        Universes { v: Vec::new() }
    }

    pub fn get_or_create(&mut self, number: u16) -> &mut Universe {
        match self.v.binary_search_by_key(&number, |u| u.number) {
            Ok(i) => &mut self.v[i],
            Err(i) => {
                self.v.insert(i, Universe::new(number));
                &mut self.v[i]
            }
        }
    }

    pub fn get(&self, number: u16) -> Option<&Universe> {
        self.v
            .binary_search_by_key(&number, |u| u.number)
            .ok()
            .map(|i| &self.v[i])
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Universe> {
        self.v.iter()
    }

    pub fn numbers(&self) -> Vec<u16> {
        self.v.iter().map(|u| u.number).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_clamp_base_1_e_transbordo() {
        let mut u = Universe::new(1);
        u.set(1, &[10.0, -5.0, 300.0, 255.9, -0.5]);
        assert_eq!(&u.data[0..5], &[10, 0, 255, 255, 0]);
        // addr 1-based: canal 512 e' o indice 511
        u.set(512, &[7.0, 9.0, 11.0]);
        assert_eq!(u.data[511], 7);
        // transbordo: nada escrito, nada estoura
        u.set(513, &[42.0]);
        u.set(0, &[42.0]);
        assert_eq!(u.data[511], 7);
        u.set(511, &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(&u.data[510..512], &[1, 2]);
    }

    #[test]
    fn set_bytes_igual_ao_set() {
        let mut a = Universe::new(1);
        let mut b = Universe::new(1);
        a.set(5, &[1.0, 2.0, 3.0]);
        b.set_bytes(5, &[1, 2, 3]);
        assert_eq!(a.data, b.data);
    }

    #[test]
    fn universes_ordenado_e_estavel() {
        let mut us = Universes::new();
        us.get_or_create(5).set(1, &[1.0]);
        us.get_or_create(1).set(1, &[2.0]);
        us.get_or_create(3);
        assert_eq!(us.numbers(), vec![1, 3, 5]);
        assert_eq!(us.get(5).unwrap().data[0], 1);
        assert_eq!(us.get(1).unwrap().data[0], 2);
        assert!(us.get(9).is_none());
        assert_eq!(us.len(), 3);
    }
}
