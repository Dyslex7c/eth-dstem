Sources:
- [S2] Bojja Venkatakrishnan, Fanti & Viswanath, Dandelion, arXiv:1701.04439v1, §§2.3–2.4, 5.3, 6 — verified.
- [S1] Fanti et al., Dandelion++, arXiv:1805.11060v1, §§3–5 — verified.
- [S8] Biryukov, Khovratovich & Pustogarov, arXiv:1405.7418v3, §§2, 4–6 — verified; source registered locally in this artifact.
- [S9] Flashbots Docs, "MEV Protection Overview"; "MEV-Share: Introduction", §§What is MEV-Share?/How does MEV-Share work?; "April 2023 (beta launch)" — verified.
- [S10] Blagoj et al., "Ethereum consensus layer validator anonymity using Dandelion++ and RLN conclusion", ethresear.ch topic 12698, posts 1, 3 and 6 — verified; source registered locally in this artifact.
- [S5] EIP-5793, "eth/68 - Add tx type to tx announcement", Abstract, Motivation, Specification, Backwards Compatibility — verified.

# Prior art: P2P transaction origin inference

Titles follow the linked preprints; dates are first arXiv submissions, sourced from each record's submission history (not conference dates).
Ethereum EL limitations below are project gap analysis against each paper's cited scope, not Ethereum measurements.

| Paper Title | Link | Date | Core Finding | What It Fails to Address for Ethereum EL |
| --- | --- | --- | --- | --- |
| Dandelion: Redesigning the Bitcoin Network for Anonymity | [Paper / full text](https://arxiv.org/abs/1701.04439) | 2017-01-16 | Randomized line-based stem forwarding before diffusion provides near-optimal source anonymity under its honest-but-curious model, measured by precision and recall [S2, §§2.3–2.4, 5.3]. | Bitcoin-focused model and implementation discussion do not establish backward-compatible `eth/68` integration or an Ethereum anonymity/latency tradeoff; both remain `unknown` [S2, §§2, 6]. |
| Dandelion++: Lightweight Cryptocurrency Networking with Formal Anonymity Guarantees | [Paper / full text](https://arxiv.org/abs/1805.11060) | 2018-05-28 | Revises stem routing to mitigate graph-learning, intersection, black-hole and partial-deployment attacks; evaluates interoperability and latency on Bitcoin mainnet [S1, §§4–5]. | Bitcoin interoperability does not establish Ethereum EL interoperability: no `eth/68` capability negotiation, legacy-peer fallback, or Ethereum-specific latency evaluation is supplied; their design/results remain `unknown` [S1, §§4.5, 5.1–5.3]. |
| Deanonymisation of clients in Bitcoin P2P network | [Paper / full text](https://arxiv.org/abs/1405.7418) | 2014-05-28 | Learns clients' entry-peer sets and matches early transaction relays to link transactions to source IPs, including clients behind NAT; experimentally tests the attack on Bitcoin testnet [S8, §§4–6]. | Studies Bitcoin, not Ethereum: its address-propagation and relay assumptions require EL-specific validation. It supplies neither Ethereum origin-inference measurements nor an `eth/68` stem defense; EL attack success remains `unknown` [S8, §§2, 4–6]. |

## Ethereum initiatives and remaining gaps

Dates are cited release, post, or EIP creation dates; limitations are project gap analysis, not measured EL results.

| Initiative | Link | Date | Mechanism | Why It Does Not Solve Public P2P Gossip Privacy |
| --- | --- | --- | --- | --- |
| Flashbots Protect RPC / MEV-Share private order flow | [Protect](https://docs.flashbots.net/flashbots-protect/overview), [MEV-Share](https://docs.flashbots.net/flashbots-mev-share/introduction), [release](https://docs.flashbots.net/flashbots-mev-share/release-notes/2023-03) | 2023-04 (MEV-Share beta) | Protect sends transactions to a private mempool; MEV-Share selectively reveals transaction data to searchers and forwards successful bundles to builders [S9]. | Trades broad public-mempool visibility for disclosure to an RPC/MEV-Share node and selected orderflow actors. It supplies no stem routing or origin concealment for users/relays still submitting through public `eth/68` peers; protecting those propagation paths remains outside its documented mechanism [S9]. |
| CL validator Dandelion++ + RLN proposal | [ethresear.ch proposal and discussion](https://ethresear.ch/t/ethereum-consensus-layer-validator-anonymity-using-dandelion-and-rln-conclusion/12698) | 2022-05-22 | Validator-only private pre-network relays messages along Dandelion++ stems before ordinary GossipSub publication; RLN supplies spam protection [S10, post 1]. | Targets beacon-validator messages, not EL transactions or `eth/68` legacy-peer interoperability. Its authors reject strong CL anonymity under latency constraints; EL anonymity/latency tradeoffs remain `unknown` [S10, posts 1, 3, 6]. |
| EIP-5793 (`eth/68`; gossip standard, not an anonymity proposal) | [EIP-5793](https://eips.ethereum.org/EIPS/eip-5793) | 2022-10-18 | Adds transaction type and size to hash announcements for selective retrieval, throttling, and bandwidth control [S5, Abstract, Motivation, Specification]. | Standardizes announcement metadata, not origin anonymity: it defines no stem phase, source-hiding mechanism, or anonymity metric. A dedicated EL-gossip-anonymity EIP is `unknown`; this row documents only the relevant verified `eth/68` wire change [S5]. |

## Verdict & Research Gap

**Verdict: Unsolved.** No source verified in this file shows any production Ethereum EL client shipping — or implementing — a stem phase for tx gossip: the only verified `eth/68` wire change standardizes announcement metadata, not anonymity [S5]; the only Ethereum stem proposal found is consensus-layer research its authors judged infeasible under CL latency constraints [S10]; Flashbots is off-protocol private order flow, not public-gossip stem routing [S9]; the stem designs themselves are Bitcoin-only [S1][S2]. Production-client deployment status beyond these sources: `unknown` pending a client-by-client audit (go-ethereum/Reth, to-verify) — so treat the EL stem phase as an open problem, not solved work.

The 3 open technical blockers:

1. **backward-compatibility with eth/68 fetch scheduling** — `eth/68` propagation is announce→fetch→serve (`NewPooledTransactionHashes`/`GetPooledTransactions`/`PooledTransactions`) [S5]. A stem hop must either forward the full tx unsolicited to one stem peer (bypassing fetch scheduling) or announce the hash (leaking first-seen timing to that peer and adding a round trip per hop); a legacy peer cannot express stem intent, so the stem silently collapses into fluff at the first non-upgraded hop, shrinking the anonymity set with partial deployment [S1, §4.5]. Backward-compat mechanism: `unknown`.
2. **MEV inclusion latency penalties** — every stem hop delays when the public mempool and block builders first see the tx, risking missed inclusion opportunities versus unstopped gossip and versus private RPC flow that reaches builders directly [S9]. No EL figure for this penalty exists in any verified source: magnitude `unknown`; Bitcoin-mainnet latency results do not transfer [S1, §5].
3. **Sybil discovery attacks** — an adversary running many peers can learn the stem/anonymity graph and a tx's entry peers, then apply first-spy/timing inference: if the origin's stem relay is Sybil, the origin is exposed on hop one [S2, §§2.3–2.4; S8, §§4–5]. Dandelion++ mitigations (4-regular graph, pseudorandom forwarding, fail-safe timers) are designed and evaluated only for Bitcoin [S1, §§4.1–4.4]; their Sybil resistance under Ethereum EL peer discovery is `unknown`.
