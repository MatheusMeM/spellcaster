//! Nada alem do que os dois binarios de bench compartilham.

/// `--nome valor` no argv; `default` se faltar ou nao for numero.
pub fn arg_f64(name: &str, default: f64) -> f64 {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == name)
        .and_then(|i| a.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ausente_cai_no_default() {
        assert_eq!(super::arg_f64("--nao-existe", 7.5), 7.5);
    }
}
