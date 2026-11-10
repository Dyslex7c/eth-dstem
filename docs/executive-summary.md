Sources: [S14] `crates/sim/src/harness.rs`, `crates/sim/src/stem/*`; [S15] `docs/benchmarks-and-results.md:7-19`; [S16] `docs/eth68s-stem-spec.md`, `docs/security-analysis.md`, `docs/eth68-propagation.md`.

# Executive Summary — Origin-Private Transaction Propagation on Ethereum EL

**Research question: can Ethereum EL achieve meaningful P2P origin privacy without breaking block inclusion latency?**

**Answer: partly — not yet as specified.** A Dandelion++-style stem phase substantially reduces origin leakage in our model, but adds seconds of latency in a meaningful tail of cases. It is not safe for latency-sensitive transactions without an explicit opt-out or a shorter fail-safe budget.

**Measured (simulated, not mainnet):**
- Baseline eth/68-style propagation deanonymizes **68%** of transactions at **10% Sybils**; Dandelion++ cuts this to **~24%** (≈65% relative gain).
- At 5% Sybils the gain is larger (**40% → 7.5%**); at 25% Sybils the baseline is **98%** deanonymization vs **~34%** with stem.
- Cost at 10% Sybils: added mean time to first adversary observation **~1.8 s**, **P95 ~8.1 s**, dominated by fail-safe rescue of dead-ended stem paths.
- Timing-based inference also weakens: mean shortest-path origin error rises from **1.92** to **~2.7 hops**.

**Protocol feasibility.** `eth/68s` is a **separate capability** (`["eth68s",1]`) advertised in RLPx Hello, not eth Status, so legacy peers never see stem messages and no upgrade is forced. The wire change is a one-entry `StemPooledTransactionHashes` plus ordinary eth body fetch, with per-tx stem→fluff transitions, fail-safe timers, and suppression rules.

**Trade-offs.** Privacy improves substantially against passive colluding first-spy adversaries. Liveness requires the fail-safe, which creates the dominant latency tail; without correct per-tx phase state, coverage stayed below 95%. MEV-Boost and private-bundle impact is `unknown`, not modelled. Graph-reconstruction resistance still needs epoch-based reshuffling; the stem graph is static in current code.

**Recommendations:**
- **Geth:** prototype `eth/68s` behind an opt-in flag; expose per-tx latency budgets so MEV-sensitive traffic can bypass stem; instrument stem dead-end and rescue rates before any default-on discussion.
- **Nethermind:** adopt the same capability/wire spec and validate interop against Geth. Its broadcast paths differ from Geth's, so first locate where local-origin txs enter relay logic and apply suppression only to stem-held txs, never to ordinary pool gossip.
- **Besu:** implement the minimal negotiated capability and single-entry announcement first; prioritize fail-safe correctness and per-tx fluff state over optimization, then reproduce the paired Sybil/latency tests before considering adoption.

**Decision:** adopt only as an **opt-in, measured rollout** with transaction-class-aware bypass. The anonymity benefit is real in simulation; the latency and MEV costs are neither low nor well-enough understood for unconditional production deployment.
