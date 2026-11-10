Sources: [S14] local model — `sim/src/bin/run_baseline.rs`, `sim/src/gossip.rs:18-60`, `sim/src/adversary/first_spy.rs:14-27`; [S15] experiment inputs below (user-specified, not client defaults).

# Baseline: first-spy deanonymization of unmitigated eth/68 gossip (simulated)

**Setup** (every value is a model input, not a measured Ethereum parameter): 200 nodes total, Sybils included; connected random graph (ring + random links), peer cap 10 (measured mean origin degree 9.79–9.95); fixed 10 ms per hop; `ceil(sqrt(peers))` full-body push, hash announce to the rest, immediate fetch on announce [S14]. Sybils are passive and colluding. The origin is a uniformly random honest node. 100 trials per row, seed `0x0065746800363800`. Success means `FirstSpyEstimator` (earliest Sybil observation → its delivering peer) returns the true origin. A trial with no Sybil observation counts as a miss.

| Sybil % | Sybils | Correct / 100 | Top-1 precision | 95% CI (Wilson) | Mean time to first spy detection (min–max) |
| --- | --- | --- | --- | --- | --- |
| 5% | 10 | 44 | **0.44** | 0.347–0.538 | 16.3 ms (10–30) |
| 10% | 20 | 68 | **0.68** | 0.583–0.763 | 13.2 ms (10–20) |
| 20% | 40 | 89 | **0.89** | 0.814–0.937 | 11.1 ms (10–20) |

Every trial had at least one Sybil observation. Detection times are simulated, so 10 ms = 1 hop: mean first detection is 1.1–1.6 hops after broadcast.

**Mechanism.** Per row, correct trials exactly equal the trials where the origin had a Sybil as a direct peer (44/68/89 of 100). This follows from the model. Only the origin's sends arrive at t = 10 ms, and every direct peer receives a `DeliverTx` or `AnnounceHash` then, so an adjacent Sybil always attributes the tx to the origin. With no adjacent Sybil, the first observation must come from a relay. In this model, precision is therefore P(origin has ≥ 1 Sybil neighbor). The hash/body split gives no protection, because an announcement reveals the sender as fast as a body does.

**Provenance.** The 10% precision is printed by the committed binary: `cargo run --manifest-path sim/Cargo.toml --bin run_baseline` → `100,10,0.680000`. The 5%/20% rows, detection times, CIs, and adjacency counts came from an uncommitted scratch copy of that binary. Its only changes: `SYBIL_PCT` is a parameter and it returns the first-observation time. It reproduced `0.680000` at 10% exactly. TODO: add a `--sybil-pct` sweep and timing column to `run_baseline` so every row is reproducible from the repo.

**Limits.** No client fetch timers (geth `txArriveTimeout = 500 ms`, see `docs/eth68-propagation.md`), latency jitter, pool admission, NAT/inbound asymmetry, or adaptive adversary are modelled. Single seed; n = 100 per row.

## Conclusion — core empirical baseline

In this simulator, a 10% passive Sybil adversary using the first-spy estimator deanonymized **68% of transactions (68/100; 95% CI 58.3–76.3%)** on unmitigated eth/68-style propagation. Each of these was identified within one hop (≤ 20 ms simulated). A stem phase must therefore stop the origin's direct peers from being first to observe the transaction. The real-mainnet deanonymization rate remains `unknown`. This is a model result, not a network measurement.
