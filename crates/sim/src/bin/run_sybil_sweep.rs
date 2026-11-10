//! Sources: [S14] `sim/src/harness.rs` model; [S15] 200 nodes, 50 paired trials per fraction.
//! Latency = mean ms to first Sybil observation (0 if none); stem coin uses `thread_rng`.
use eth_tx_sim::harness::{random_topology, run_trial, Protocol::*, MODEL_SEED, NETWORK_SIZE};
use eth_tx_sim::node::NodeType::{Honest, Sybil, Target};
use eth_tx_sim::stem::{config::StemConfig, digraph::build_stem_digraph};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

fn main() {
    let (config, trials) = (StemConfig::default(), 50);
    println!("sybil_pct,baseline_precision,dandelion_precision,latency_penalty_ms");
    for pct in [5, 10, 15, 20, 25] {
        let (mut rng, sybils) = (StdRng::seed_from_u64(MODEL_SEED), NETWORK_SIZE * pct / 100);
        let (mut hits, mut ms) = ([0usize; 2], [0u64; 2]);
        for _ in 0..trials {
            let mut topo = random_topology(&mut rng);
            let mut ids: Vec<usize> = (0..NETWORK_SIZE).collect();
            ids.shuffle(&mut rng);
            for (rank, &id) in ids.iter().enumerate() {
                topo.nodes[id].node_type = if rank < sybils { Sybil } else { Honest };
            }
            let routes = build_stem_digraph(&topo, config.stem_degree);
            let origin = ids[rng.gen_range(sybils..NETWORK_SIZE)];
            topo.nodes[origin].node_type = Target;
            let tx_hash: [u8; 32] = rng.gen();
            for (i, protocol) in [Eth68Flood, DandelionStem].into_iter().enumerate() {
                let (hit, first_ms) = run_trial(&topo, &routes, protocol, &config, origin, tx_hash);
                (hits[i], ms[i]) = (hits[i] + usize::from(hit), ms[i] + first_ms);
            }
        }
        let [base, dand] = hits.map(|h| h as f64 / trials as f64);
        let penalty = (ms[1] as f64 - ms[0] as f64) / trials as f64;
        println!("{pct},{base:.6},{dand:.6},{penalty:.3}");
    }
}
