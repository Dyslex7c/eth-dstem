#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StemConfig {
    pub stem_probability: f64,
    pub fail_safe_timeout_ms: u64,
    pub stem_degree: usize,
}

impl Default for StemConfig {
    fn default() -> Self {
        Self {
            stem_probability: 0.90,
            fail_safe_timeout_ms: 8_000,
            stem_degree: 2,
        }
    }
}

impl StemConfig {
    pub fn new(p: f64, timeout: u64, degree: usize) -> Result<Self, &'static str> {
        if !(0.0 < p && p < 1.0) {
            return Err("stem_probability must satisfy 0.0 < p < 1.0");
        }
        if degree < 1 {
            return Err("stem_degree must be at least 1");
        }
        Ok(Self {
            stem_probability: p,
            fail_safe_timeout_ms: timeout,
            stem_degree: degree,
        })
    }
}
