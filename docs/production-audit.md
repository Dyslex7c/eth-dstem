Sources: [S14] `crates/dstem/` in this worktree (`src/*.rs`, `Cargo.toml:6-10`, 11 tests in `tests/`); audit harness run 2026-10-28 (release build, macOS, deleted after use); rules `docs/eth68s-stem-spec.md:8-23,42-52`; host contracts `docs/reth-integration.md`, `docs/nethermind-integration.md`; host peer caps `docs/eth68-propagation.md:47`.
# dstem production audit — 2026-10-28
**Verdict: NOT production-ready.** Every item below was audited. Memory safety passes. DoS bounds **fail**: there is no in-flight cap and no per-peer quota, and duplicates re-arm the timer. Interop fallback passes, but only under host contracts `dstem` cannot enforce. Seven blockers, B1–B7.
**Method.** `rg 'unwrap|expect|panic!|unreachable!|todo!|unsafe'` over `src/` → 0 hits. Clippy restriction lints (`unwrap_used, expect_used, panic, indexing_slicing, arithmetic_side_effects, cast_possible_truncation`) → 10 hits, each proved unreachable below. DoS figures come from a scratch harness.

| Area | Check | Status | Evidence |
| --- | --- | --- | --- |
| Memory | 0 `unwrap`/`expect`/`panic!` in runtime paths | PASS | 0 hits in `src/` (uses are confined to `tests/`); no `unsafe` block. |
| Memory | Panic-capable index/slice | PASS | `wire.rs:28` `body[0]` is short-circuited by `len == 1`; `wire.rs:23` slice start `leading_zeros/8 ≤ 4` on a 4-byte array; `wire.rs:31` `be[zeros..]` has `zeros ≤ 7` since `len ≥ 56`. |
| Memory | Overflow / truncation | PASS | `wire.rs:30-31` header peaks at `0xc0+55+8 = 0xff`; `epoch.rs:21` `i < k ≤ len` keeps `gen_range` non-empty and `j < len`; `service.rs:18,27` overflow only after ~5.8e8 years. Release builds wrap silently, so these proofs are load-bearing. |
| Memory | Atomic `RwLock`s | PASS | `parking_lot::RwLock` (never poisons) guards every shared set (`peers.rs:8`, `route.rs:14`, `tracker.rs:19`). Each mutation completes under one guard; scan+evict (`failsafe.rs:10`) and the drop purge (`disconnect.rs:17-19`) are single critical sections. |
| Memory | Deadlock freedom | PASS | The only nested acquisition is `disconnect.rs:17-19`, fixed order registry → routes → flights. `service.rs:21` drops the route guard before `tracker.write()`. New code must keep this order. |
| Memory | Bounded queues | N/A | `dstem` owns no channel or task. `tokio` is declared (`Cargo.toml:7`) but unused in `src/` (B7). Queue bounds belong to the host (Reth `MemoryBoundedReceiver`). |
| Memory | TOCTOU windows | OPEN (B6) | `epoch.rs:18→25` snapshots peers then writes routes, so a peer dropped in between re-enters routes until the next reshuffle. `service.rs:21→24` can track against a just-dropped peer. Both are liveness-safe: fail-safe expiry fluffs within `T`. |
| DoS | Max in-flight limit | **FAIL (B1)** | `tracker.rs:25` inserts unconditionally into an unbounded `HashMap`. Harness: 1,000,000 distinct hashes from one source → 1,000,000 retained, **~524 MB peak RSS** (136 B payload per entry + table + sweep output). Steady state `≈ r·T`: at `T = 8 s`, 125k stem msgs/s holds 1M entries. Fix: `max_in_flight`, fluff on overflow (fail-open keeps liveness). |
| DoS | Sybil memory exhaustion / per-peer quota | **FAIL (B2)** | No per-source accounting; `handle_inbound_stem` is never told the sender. A few Sybils can fill any global cap and push honest txs into fluff. Fix: per-peer in-flight quota + token bucket keyed by inbound `PeerId`. |
| DoS | Duplicate `0x0b` | **FAIL (B3)** | Spec: ignore duplicates, timer unchanged (`spec:19,42`). Harness: re-track moved the deadline 1000→9000 and the peer 1→2, so resends postpone fluff. Fix: `entry().or_insert` + terminal FLUFFED set. |
| DoS | Stale hold after fluff | **FAIL (B4)** | `Ok(None)` leaves the old record (harness: `hold_still_present=true`); a later sweep fluffs again, breaking single-exit (`spec:52`). Fix: remove on fluff. |
| DoS | Sweep cost | WARN | `collect_expired_records` is O(n) under the write lock: **93 ms at 1M entries**, stalling all relays. Bounded once B1 lands; a deadline heap removes it. |
| DoS | Peer/route growth | PASS (contract) | `peers.rs:14` `contains` is O(n), acceptable only because hosts cap peers at 50 by default. Routes are replaced wholesale, sized by `degree`. |
| Interop | Fallback to eth/68 with no eth68s peer | PASS | Empty routes → `Ok(None)`, never `Err`, nothing tracked: harness 10,000/10,000. Host relays via plain eth/68. |
| Interop | Only negotiated peers become stem hops | CONTRACT | No capability check in `dstem`; the host adds a peer only after both Hellos list `["eth68s", 1]` (`spec:8-9`). Reth: `OnNotSupported::KeepAlive`; Nethermind: `HasAgreedCapability`. |
| Interop | Legacy peers untouched | PASS (design) | `0x0b` is capability-local; unshared caps are ignored (`spec:5,23`). Live-client test: `unknown`. |
| Interop | Wire validity | OPEN (B5) | `wire.rs` encodes only — no decoder; it does not enforce equal-length vectors, a single entry, or blob exclusion (`spec:14-15`). Inbound validation is unimplemented. |

**Blockers:** B1 in-flight cap · B2 per-peer quota + rate limit · B3 ignore duplicates · B4 clear hold on fluff · B5 decoder + inbound validation · B6 reshuffle/track TOCTOU · B7 drop unused `tokio`.
**Not yet audited:** fuzzing of encode/decode, `cargo audit`/`cargo deny`, Miri, loom model-checking of lock order, live Geth/Reth/Nethermind interop.
