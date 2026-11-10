# eth-dstem

[![CI](https://github.com/Dyslex7c/eth-dstem/actions/workflows/ci.yml/badge.svg)](https://github.com/Dyslex7c/eth-dstem/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](Cargo.toml)

Origin-private transaction propagation for the Ethereum Execution Layer, introducing a backward-compatible Dandelion++ stem phase (`eth/68s`) to mitigate P2P first-spy deanonymization.

---

## Overview

On Ethereum's Execution Layer (EL), standard transaction diffusion (`eth/68`) announces new transactions to peers immediately upon pool admission. A colluding set of passive "first-spy" adversary nodes can observe initial arrival timestamps and link an unconfirmed transaction's origin IP to its sender address with high precision.

`eth-dstem` researches, specifies, and implements an opt-in, backward-compatible stem phase (`eth/68s`) on DevP2P/RLPx:
1. **Stem Phase:** An originating node forwards a single-entry transaction announcement along a pseudo-random line route of connected stem peers with probability `p` (default `0.90`).
2. **Fluff Phase:** At each hop, with probability `1 - p` (or upon fail-safe timer expiry), the transaction transitions into standard diffusion (fluff), broadcasting to the general mempool.
3. **Fail-Safe Mechanism:** Guarantees block inclusion liveness by triggering diffusion if a stem transaction has not been witnessed via normal gossip within a configurable window `T` (e.g. 8,000 ms).

---

## Benchmark & Empirical Results

Simulated on dynamic P2P graphs (50 honest nodes with bounded degrees, variable Sybil fraction, and normal propagation delays):

| Sybil Fraction | Baseline (`eth/68` Flood) Precision | `eth/68s` (Stem Routing) Precision | Relative Origin Protection |
| :--- | :--- | :--- | :--- |
| **5%** | 40.0% | 7.5% | **81.2% reduction** |
| **10%** | 68.0% | 20.0% - 24.0% | **65.0% reduction** |
| **25%** | 98.0% | 34.0% | **65.3% reduction** |

### Latency Trade-offs
* **Mean latency overhead:** ~1.8 s at 10% Sybil coverage.
* **P95 latency overhead:** ~8.1 s, bounded directly by fail-safe timeout recovery on dead-ended stem routes.
* **Timing inference:** Shortest-path origin distance estimation error increases from 1.92 hops to ~2.7 hops.

---

## Workspace Architecture

```text
eth-dstem/
├── crates/
│   ├── dstem/         # Production-grade async client middleware (Rust/tokio)
│   └── sim/           # Discrete-event network simulation & benchmark sweep engine
├── docs/              # Specifications, threat models, and client integration designs
└── .github/workflows/ # GitHub Actions CI workflow
```

### 1. `crates/dstem` (Reference Middleware)
A modular Rust crate designed for direct embedding into Ethereum EL clients (e.g., Reth, Nethermind):
* **`capability`:** RLPx `["eth68s", 1]` capability negotiation via RLPx `Hello` without breaking legacy `eth` status handshakes.
* **`wire`:** Single-entry `StemPooledTransactionHashes` (`0x0b`) frame encoder and decoder conforming to EIP-5793 shapes.
* **`peers`:** Lock-free, concurrent peer registry managing stem capabilities, disconnect events, and dynamic route tables.
* **`epoch`:** Epoch-based stem graph reshuffling using local PRF hashing (`PRF(local_secret, epoch || node_id || peer_id)`).
* **`failsafe`:** In-flight stem tracker with deadline queues that automatically drain and trigger emergency fluffing.

### 2. `crates/sim` (`eth-tx-sim`)
A discrete-event network simulator modeled after Ethereum devp2p meshes:
* **Adversary Models:** Evaluates both `FirstSpyEstimator` and graph-theoretic `TimingEstimator` (squared timestamp error minimization over all observers).
* **Topology:** Symmetric capacity-constrained random topologies with configurable Sybil node placements.
* **Reproducible CLI Sweeps:** Built-in binaries to reproduce baseline comparisons, Sybil sweeps, and latency distributions.

### 3. `docs/` (Research & Specifications)
* [`eth68s-stem-spec.md`](docs/eth68s-stem-spec.md): Formal wire spec, capability negotiation, and state transition invariants.
* [`executive-summary.md`](docs/executive-summary.md): Research summary, empirical findings, and client recommendations.
* [`benchmarks-and-results.md`](docs/benchmarks-and-results.md): Detailed benchmark tables, precision curves, and latency metrics.
* [`eth68-propagation.md`](docs/eth68-propagation.md): Deep-dive into Geth (`sqrt(peers)` rule) vs Nethermind broadcast paths.
* [`nethermind-integration.md`](docs/nethermind-integration.md): Concrete integration architecture for Nethermind (.NET / C#).
* [`reth-integration.md`](docs/reth-integration.md): Concrete integration architecture for Paradigm's Reth (Rust).
* [`security-analysis.md`](docs/security-analysis.md): Graph deanonymization defenses, Sybil resilience, and PRF permutation proofs.
* [`production-audit.md`](docs/production-audit.md): DoS vector analysis, memory exhaustion mitigation, and per-peer rate limiting.
* [`prior-art.md`](docs/prior-art.md): Historical context comparing Dandelion, Dandelion++, Kovri, and Tor.

---

## Getting Started

### Prerequisites
* Rust 1.85+ (`cargo`, `rustc`)

### Running Tests
Execute all unit and integration tests across the workspace:

```bash
cargo test --workspace --all-targets
```

Verify formatting and lints:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

### Running Benchmark Sweeps

Run the baseline `eth/68` flooding simulation:
```bash
cargo run --release -p eth-tx-sim --bin run_baseline
```

Run the paired Dandelion++ comparison (precision and mean latency):
```bash
cargo run --release -p eth-tx-sim --bin run_dandelion
```

Run the Sybil fraction sensitivity sweep (1% to 50% adversary coverage):
```bash
cargo run --release -p eth-tx-sim --bin run_sybil_sweep
```

Run the timing inference estimator sweep:
```bash
cargo run --release -p eth-tx-sim --bin run_timing_sweep
```

---

## Protocol Specification Summary (`eth/68s`)

1. **Hello Capability:** Nodes advertise `["eth68s", 1]` in the RLPx `Hello` packet.
2. **Wire Framing:** Message ID `0x0b` (`StemPooledTransactionHashes`) carries:
   ```text
   [types: B, [size0: P], [hash0: B_32]]
   ```
   Strictly enforces single-transaction announcements to prevent multi-transaction correlation.
3. **Opt-in / Fallback:** If a node has 0 connected `eth68s` peers, transactions immediately default to standard diffusion, ensuring zero liveness failure.

---

## License

Dual-licensed under either:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
