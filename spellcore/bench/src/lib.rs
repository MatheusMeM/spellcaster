//! Nada alem do que os dois binarios de bench compartilham.

/// `--nome valor` no argv; `default` se faltar ou nao for numero.
pub fn arg_f64(name: &str, default: f64) -> f64 {
    let a: Vec<String> = std::env::args().collect();
    for i in 0..a.len() {
        if a[i] == name {
            if let Some(x) = a.get(i + 1).and_then(|v| v.parse().ok()) {
                return x;
            }
        }
    }
    default
}

#[cfg(test)]
mod tests {
    #[test]
    fn ausente_cai_no_default() {
        assert_eq!(super::arg_f64("--nao-existe", 7.5), 7.5);
    }
}
