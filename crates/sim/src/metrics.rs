#[derive(Clone, Debug, Default)]
pub struct SimulationMetrics {
    pub total_trials: usize,
    pub deanonymized_trials: usize,
    pub propagation_latencies_ms: Vec<u64>,
    pub stem_hop_counts: Vec<usize>,
}

impl SimulationMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_result(&mut self, deanonymized: bool, latency_ms: u64, hops: usize) {
        self.total_trials += 1;
        if deanonymized {
            self.deanonymized_trials += 1;
        }
        self.propagation_latencies_ms.push(latency_ms);
        self.stem_hop_counts.push(hops);
    }

    pub fn top1_accuracy(&self) -> f64 {
        if self.total_trials == 0 {
            0.0
        } else {
            self.deanonymized_trials as f64 / self.total_trials as f64
        }
    }
}
