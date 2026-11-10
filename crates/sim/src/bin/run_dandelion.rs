//! Sources: [S14] local model — `crates/sim/src/harness.rs` (graph, hop latency, and two-hop hash
//! fetch identical to `crates/sim/src/bin/run_baseline.rs:23-92`); [S15] 100 trials, 200 nodes, 10%
//! Sybils. `avg_latency_ms` is the mean simulated time to the first Sybil observation, not
//! end-to-end propagation. Graphs, origins, and tx hashes are seeded and identical for both
//! rows; the Dandelion++ coin comes from `thread_rng` (`crates/sim/src/stem/lifecycle.rs:21`), so
//! that row varies between runs. Real-network leakage is `unknown`.

use eth_tx_sim::harness::{run_experiment, Protocol, SYBIL_PCT};
use eth_tx_sim::metrics::SimulationMetrics;

fn summarize(name: &str, metrics: &SimulationMetrics) {
    let trials = metrics.total_trials.max(1) as f64;
    let latency_sum: u64 = metrics.propagation_latencies_ms.iter().sum();
    let avg_latency_ms = latency_sum as f64 / trials;
    let top1_precision = metrics.top1_accuracy();
    println!("{name},{SYBIL_PCT},{top1_precision:.6},{avg_latency_ms:.3}");
}

fn main() {
    println!("protocol,sybil_pct,top1_precision,avg_latency_ms");
    summarize("eth68_flood", &run_experiment(Protocol::Eth68Flood));
    summarize("dandelion_pp", &run_experiment(Protocol::DandelionStem));
}
