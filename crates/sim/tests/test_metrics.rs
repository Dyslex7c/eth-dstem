use eth_tx_sim::metrics::SimulationMetrics;

#[test]
fn accuracy_and_mean_latency_match_recorded_samples() {
    let mut metrics = SimulationMetrics::default();
    for _ in 0..25 {
        metrics.record_result(true, 150, 0);
    }
    for _ in 0..75 {
        metrics.record_result(false, 280, 0);
    }

    assert_eq!(metrics.total_trials, 100);
    assert_eq!(metrics.deanonymized_trials, 25);
    assert_eq!(metrics.propagation_latencies_ms.len(), 100);
    assert_eq!(metrics.top1_accuracy(), 0.25);

    let sum: u64 = metrics.propagation_latencies_ms.iter().sum();
    let mean = sum as f64 / metrics.propagation_latencies_ms.len() as f64;
    assert_eq!(mean, 247.5);
}
