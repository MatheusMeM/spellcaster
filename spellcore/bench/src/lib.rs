//! Nothing beyond what the two bench binaries share.

/// `--name value` in argv; `default` if it is missing or not a number.
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
    fn a_missing_one_falls_back_to_the_default() {
        assert_eq!(super::arg_f64("--does-not-exist", 7.5), 7.5);
    }
}
