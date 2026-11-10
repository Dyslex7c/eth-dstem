Sources: [S14] `sim/src/harness.rs`, `sim/src/adversary/timing.rs:22-58`, `sim/src/bin/run_sybil_sweep.rs`, `sim/src/bin/run_timing_sweep.rs`; [S15] 200-node graph, 50 paired trials per Sybil fraction, `MODEL_SEED`.

# Benchmarks & Results (simulated — not a mainnet measurement)

**Method.** Each fraction re-seeds the 200-node ring+random graph, marks `pct`% as Sybil, and runs both protocols on the *same* graph, origin, and tx hash. Deanonymization = top-1 `FirstSpyEstimator` precision. Dandelion++ coin uses `thread_rng`, so that column is a mean over 8 independent runs; baseline is deterministic. P95 penalty = 95th percentile of (dandelion − baseline) time-to-first-Sybil-observation. **Provenance:** the committed `run_sybil_sweep` prints mean penalty only, so the table (P95 column + multi-run means) came from an uncommitted scratch copy that mirrors its trial loop; its baseline column reproduces the binary exactly (0.400/0.680/0.880/0.880/0.980). TODO(verify): add `--runs` and P95 output to `run_sybil_sweep`.

| Sybil Fraction | eth/68 Baseline Deanonymization | Dandelion++ Deanonymization | Anonymity Gain (%) | P95 Latency Penalty (ms) |
| --- | --- | --- | --- | --- |
| 5% | 0.400 | 0.075 | 81.2 | 8249 |
| 10% | 0.680 | 0.238 | 65.1 | 8152 |
| 15% | 0.880 | 0.295 | 66.5 | 8135 |
| 20% | 0.880 | 0.368 | 58.2 | 8104 |
| 25% | 0.980 | 0.338 | 65.6 | 8098 |

**Key evidence (10% Sybils).** Deanonymization falls from **0.680 to 0.238** (65% relative gain, i.e. fewer than 1 in 4 traced vs 2 in 3), at a **P95 penalty of ~8.1 s**. The benchmark's motivating "38% → <12% with ~320 ms" figures do **not** hold: real baseline precision at 10% is 0.680, and the penalty is ~26x larger.

**Why the penalty is ~8 s.** It is dominated by `fail_safe_timeout_ms = 8000`: 13–15 of 50 Dandelion trials at 10% Sybils dead-end in the stem (nodes relay only on first receipt) and are rescued by fail-safe fluff at t≈8 s. Mean penalty per fraction: 3266 / 1824 / 1629 / 1454 / 1190 ms for 5→25%.

**Timing adversary** (`run_timing_sweep`, maximum-likelihood shortest-path arrival; mean error distance in hops, 25 scored trials at 10% Sybils): baseline **1.92** (deterministic) vs Dandelion++ **2.60–2.88** (mean 2.71 over 6 runs) — stem routing pushes the ML origin estimate ~0.8 hops further from the truth.

**Caveats.** No client fetch timers, pool admission, or NAT asymmetry; single seed for graphs; `n=50` per fraction (binomial SE ≈ 0.07 on precision). Real-ethereum leakage and latency remain `unknown`.
