use eth_tx_sim::stem::config::StemConfig;

#[test]
fn default_carries_the_documented_parameters() {
    let config = StemConfig::default();
    assert_eq!(config.stem_probability, 0.90);
    assert_eq!(config.fail_safe_timeout_ms, 8_000);
    assert_eq!(config.stem_degree, 2);
}

#[test]
fn constructor_rejects_invalid_parameters() {
    let prob_err = "stem_probability must satisfy 0.0 < p < 1.0";
    let degree_err = "stem_degree must be at least 1";
    for prob in [0.0, 1.0, -0.1, f64::NAN] {
        assert_eq!(StemConfig::new(prob, 8_000, 2).unwrap_err(), prob_err);
    }
    assert_eq!(StemConfig::new(0.9, 8_000, 0).unwrap_err(), degree_err);
}

#[test]
fn constructor_accepts_valid_custom_parameters() {
    let config = StemConfig::new(0.5, 250, 1).unwrap();
    assert_eq!(config.stem_probability, 0.5);
    assert_eq!(config.fail_safe_timeout_ms, 250);
    assert_eq!(config.stem_degree, 1);
    assert!(StemConfig::new(0.999, u64::MAX, usize::MAX).is_ok());
}
