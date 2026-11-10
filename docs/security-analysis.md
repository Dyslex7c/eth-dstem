Sources: [S14] `crates/sim/src/stem/config.rs:3-13`, `crates/sim/src/stem/failsafe.rs:14-27`, `crates/sim/src/stem/fallback.rs:5-21`, `crates/sim/src/stem/digraph.rs:10-19,28-33`, `crates/sim/src/harness.rs:355-373`; [S15] `docs/benchmarks-and-results.md:7-19`; `docs/eth68s-stem-spec.md:13-20,30-54`.

# Security Analysis — eth/68s Stem Phase

**Threat model.** The adversary controls a fraction of nodes in a 200-node network, colludes across them, records first and later transaction-arrival times, and may accept a stem forward then withhold it. Honest nodes follow eth/68s. The simulator measures passive colluding Sybils; an actively dropping stem peer is a design extension, so blackhole resistance below is algorithmic, not yet a measured result.

## 1. Sybil disconnection / blackhole
- **Mitigation algorithm:** every stem forward arms `deadline = decision_time + T`, with `T = 8,000 ms`. If diffusion is not observed by that deadline, the forwarder itself broadcasts via eth fluff. On a detected peer drop/timeout, remove that peer from the local stem set, forward to the first surviving stem peer, and fluff immediately if none survives.
- **Status:** the timer path is implemented in the harness; `handle_stem_forward_failure` is implemented and unit-tested but not yet wired into the simulation loop. At 10% Sybils, 13–15/50 simulated trials needed fail-safe rescue. Residual risk: worst-case added delay is approximately `T` per rescued transaction, and failure-detection time is `unknown`.

## 2. Anonymity-graph reconstruction
- **Attack:** repeated transactions and arrival observations can reveal a node's persistent stem neighbours, letting colluding peers reconstruct useful parts of the stem graph and improve first-spy or timing inference.
- **Mitigation algorithm:** at each epoch `e`, rank every outbound peer by a local pseudorandom function `PRF(local_secret, e || node_id || peer_id)`, select the top `min(stem_degree, peer_count)` peers as the new stem set, replace the old set atomically, and choose uniformly among that set per transaction. Do not expose the set or epoch state on the wire.
- **Status:** current code uses a static pseudorandom rank over `node_id || peer_id` and has no epoch input, so reshuffling is **specified but not implemented or measured**. Epoch duration and reconstruction resistance are `unknown`; this remains a required mitigation, not a demonstrated defence.

## 3. MEV inclusion delay
- **Quantified gossip cost at 10% Sybils:** deanonymization falls from `0.680` to `0.238`, while added time to first Sybil observation averages `1,824 ms` with P95 `8,152 ms`. These are simulator propagation observations, not builder-receipt or block-inclusion measurements.
- **Mitigation algorithm:** give each locally submitted transaction a latency budget `B_tx`; use `effective_deadline = submission_time + min(T, B_tx)`, and fluff when that deadline expires without diffusion. A latency-sensitive transaction may set `B_tx = 0`, opting out of stem privacy for immediate fluff. This per-transaction budget is a proposed control, not implemented in the current global `StemConfig`.
- **MEV-Boost / private bundles:** actual builder, relay, and bundle-inclusion effects are `unknown` because no MEV-Boost builder, relay, slot schedule, or private-bundle path is modelled. The quantified delay applies only when delivery depends on public EL gossip; no numerical bundle-delay claim is made.

**Residual position.** Fail-safe fluff bounds silent loss but creates the dominant latency tail. Epoch reshuffling and deadline-aware opt-out are concrete next implementations; until they are implemented and benchmarked, their security and MEV effects remain `unknown`.
