Sources: [S4] devp2p `caps/eth.md` (master = eth/72): Status `0x00`, `0x08`–`0x0a`, Transaction Exchange, Change Log eth/68 — verified 2026-10-27. [S5] EIP-5793. [S11] devp2p `rlpx.md`: Capability Messaging, Message ID-based Multiplexing, Hello `0x00`, Disconnect `0x01` — verified 2026-10-27. [S12] RLP. [S6] go-ethereum `d01e869d59696d915410b0b5bc374ae1a8b9418e`: `eth/protocols/eth/protocol.go:33-49`, `eth/protocols/eth/handler.go:301-304`, `p2p/peer.go:43,437-460` — verified. [S1] Dandelion++ (stem/fluff concept; to-verify). [S14] `sim/` model and `docs/benchmarks-and-results.md`.

# eth/68s — stem-phase extension for `eth` (status: draft proposal)

**Deviations from the brief, forced by the specs.** (1) Capabilities are advertised in the RLPx p2p `Hello` (`capabilities = [[name, version], …]`), not in eth `Status`, which has no capability field [S11 Hello; S4 Status]. (2) A capability is an ASCII name (≤ 8 chars) plus a version *number* [S11], so "68s" cannot be an eth version: eth/68s is the capability **`["eth68s", 1]`**. (3) `0x0b` is relative to eth68s, not inside eth: geth rejects unknown eth codes with `errInvalidMsgCode` [S6 handler.go:301-304], and [S4] assigns nothing to eth `0x0b`/`0x0c` only up to eth/72, so a future eth version could claim them.

## 1. Capability negotiation
- Advertise `["eth68s", 1]` in Hello next to `eth`. The eth `Status` is sent unchanged; eth68s defines no Status message [S4].
- eth68s is active on a connection iff both Hellos list it **and** an `eth` version is shared, because it fetches bodies with eth `GetPooledTransactions` `0x09` / `PooledTransactions` `0x0a`, whose shapes are identical from eth/68 to eth/72 [S4]. Without shared eth, eth68s messages MUST NOT be sent.
- Message-ID length: **12** (`0x00`–`0x0b`). `0x00`–`0x0a` are reserved and MUST NOT be sent; receiving one → Disconnect `0x02` (breach of protocol) [S11].
- Wire id: shared caps take compact IDs from `0x10` in alphabetic order [S11; S6 peer.go:437-460], and `eth` < `eth68s` < `snap`. With no other shared name between `eth` and `eth68s`, wire id = `0x10 + len(eth/v) + 0x0b`. Using geth lengths [S6 protocol.go:49]: eth/69, eth/70 = 18 → **`0x2d`**; eth/71 = 20 → `0x2f`; eth/72 = 22 → `0x31` (each a single RLP byte [S12]). eth/68 length: `unknown` (geth at [S6] no longer ships eth/68, `protocol.go:45`).

## 2. `StemPooledTransactionHashes` (eth68s `0x0b`)
`[types: B, [size₀: P], [hash₀: B_32]]`: the eth/68 `0x08` shape without the eth/72 `cells` field. Field semantics follow [S4 `0x08`; S5]; all three elements have equal length.
- MUST carry exactly **one** entry (project design rule: batching would link txs to one origin). Type `0x03` (blob) entries MUST NOT be stemmed; receivers discard them (blob interaction: `unknown`).
- **Sender:** sends to exactly one peer of its current stem set (≤ `stem_degree` eth68s peers, default 2 in [S14]; epoch rotation period `unknown`, TODO(verify) [S1]).
- **Receiver:** fetches the body via eth `0x09` from the sender and validates it like any pooled tx [S4]. While the tx is in stem it MUST NOT relay it via eth `0x02`/`0x08`, MUST omit it from the new-connection pool-sync `0x08` [S4 Transaction Exchange], and MUST skip it when answering other peers' `0x09` (skipping is permitted [S4 `0x0a`]).
- **Relay step, per tx:** with probability `p` (default 0.90 [S14]) send one `0x0b` to one of its own stem peers; otherwise **fluff**, i.e. relay normally via eth, after which the tx is ordinary pool content. Phase is per-tx and one-way: a node that has fluffed the tx, or seen it via eth `0x02`/`0x08`, never stems it again. Without this rule fluff does not flood, as found in [S14] `sim/src/harness.rs`.
- A duplicate `0x0b` for a tx already held is ignored (not forwarded), preventing loops.
- **Fail-safe:** on forwarding, arm a timer `T`; if the tx is not seen via eth `0x02`/`0x08` before `T`, fluff it. `T = 8000 ms` is a model input [S14], not a sourced constant; in simulation it sets the P95 latency penalty (~8.1 s, `docs/benchmarks-and-results.md`). Mainnet value: `unknown`.

## 3. Backward-compatibility rules
1. A peer without eth68s never receives `0x0b`: unshared capabilities are ignored [S11]. Its own Hello and the eth Status are unchanged; it ignores the extra `eth68s` entry in ours.
2. Stem peers are drawn only from eth68s-sharing peers. With none, a node originates by plain eth relay: no anonymity gain, but no failure.
3. A legacy peer first sees a stem tx at fluff, as ordinary eth `0x02`/`0x08` traffic; nothing new is required of it.
4. The interop target is any shared eth ≥ 68; geth at [S6] speaks only eth/69–72 (`protocol.go:45`).

**Open:** reserved `0x00`–`0x0a` range vs. a 1-message space (code `0x00`) is a design choice. Production adoption and the tolerable latency budget: `unknown`.

## 4. Stem-to-Fluff State Machine & Fail-Safe Invariants
Per tx, one local variable `phase ∈ {STEM, FLUFFED}` plus one timer; the wire carries no phase flag. Nodes that have never seen the tx hold no state. [S14]

| Current state | Trigger | Action | Next |
| --- | --- | --- | --- |
| unseen tx, locally submitted | coin `u ~ U[0,1)`, `u < p` | one eth68s `0x0b` to one stem peer; arm fail-safe `T` | STEM |
| unseen tx, locally submitted | `u ≥ p` | immediate fluff via eth `0x02`/`0x08` to the ordinary peer set; no timer | FLUFFED |
| STEM, `0x0b` received, body fetched via eth `0x09` | `u < p` | one `0x0b` to one of **this node's** own stem peers; arm `T` | STEM |
| STEM, `0x0b` received | `u ≥ p` | fluff via eth; cancel timer | FLUFFED |
| STEM | `T` expires, tx still unseen via eth | fluff via eth from this node | FLUFFED |
| STEM | tx seen via eth `0x02`/`0x08` (diffusion) or in a block | cancel timer; ordinary pool content | FLUFFED |
| STEM | no stem peers, or chosen peer drops/times out | fluff immediately; no further `0x0b` | FLUFFED |
| STEM | duplicate `0x0b` for a tx already held | ignore; do not forward; timer unchanged | STEM |
| FLUFFED | later `0x0b` or duplicate eth relay | normal eth behaviour; never re-enter stem | FLUFFED |

**Constants.** `p = 0.90`, so fluff probability `1 - p = 0.10`; `T = 8000 ms` (8.0 s); `stem_degree = 2`. All are model inputs [S14 `sim/src/stem/config.rs`], not sourced protocol constants; mainnet values `unknown`. One fresh `u` per relay decision makes pre-fluff path length geometric with mean `1/(1-p) = 10` hops — the model's expectation, not a measured guarantee.
**Timer start.** Armed at the forwarder when the forward decision is made (`t_arm = t_decision`, not on delivery); deadline `= t_arm + T`. Per forwarder per tx, at most one armed timer; re-arming forbidden.

**Invariants.**
1. *Monotone phase.* FLUFFED is absorbing per node per tx: nothing returns to STEM, and a node that fluffed never stems the tx again. Required for coverage — without it fluff recipients re-entered the stem and coverage stayed below 95% [S14].
2. *Liveness.* For every tx in STEM at a node, either an in-flight `0x0b` exists or a live timer is armed, so every path reaches FLUFFED within one `T` of the last forward and the tx cannot silently vanish. Measured rescue rate: 13–15/50 trials at 10% Sybils (`docs/benchmarks-and-results.md`).
3. *Suppression.* While STEM the tx is not relayed, pool-synced, or served to other peers (§2); fluff is the only diffusion entry point.
4. *Single exit, timer cancellation.* Fluff fires at most once per node per tx; the first of {coin failure, timeout, diffusion, no viable peer} wins, clears the local timer, and cancels the rest. Stale deadlines cause spurious late broadcasts [S14].
5. *No origin signal on the wire.* `0x0b` carries only the announcement, so phase, timer, and hop count are unobservable to other peers.
