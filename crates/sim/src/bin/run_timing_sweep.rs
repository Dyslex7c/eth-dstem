//! Sources: [S14] `crates/sim/src/harness.rs` model + `crates/sim/src/adversary/timing.rs:22-58`
//! (shortest-path arrival ML); [S15] 200 nodes, 25 paired trials per protocol. Error
//! distance = BFS hops from the ML estimate to the true origin. Trials with no Sybil
//! observation carry no estimate, so they are excluded from the mean.
use eth_tx_sim::harness::{draw_origin, hop_distance, random_topology, MODEL_SEED};
use eth_tx_sim::harness::{run_trial_observed, timing_estimate, Protocol::*};
use eth_tx_sim::stem::{config::StemConfig, digraph::build_stem_digraph};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn main() {
    let (config, trials) = (StemConfig::default(), 25);
    let mut rng = StdRng::seed_from_u64(MODEL_SEED);
    let (mut error, mut scored) = ([0.0f64; 2], [0usize; 2]);
    for _ in 0..trials {
        let mut topo = random_topology(&mut rng);
        let routes = build_stem_digraph(&topo, config.stem_degree);
        let origin = draw_origin(&mut topo, &mut rng);
        let tx_hash: [u8; 32] = rng.gen();
        for (i, protocol) in [Eth68Flood, DandelionStem].into_iter().enumerate() {
            let (_, _, seen) =
                run_trial_observed(&topo, &routes, protocol, &config, origin, tx_hash);
            let estimate = timing_estimate(&topo, &seen);
            error[i] += estimate.map_or(0, |node| hop_distance(&topo, node, origin)) as f64;
            scored[i] += usize::from(estimate.is_some());
        }
    }
    eprintln!("scored_trials,{},{}", scored[0], scored[1]);
    for (name, i) in [("baseline", 0), ("dandelion", 1)] {
        let mean = error[i] / scored[i].max(1) as f64;
        println!("mean_error_distance_{name},{mean:.4}");
    }
}
