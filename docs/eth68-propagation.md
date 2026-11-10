Sources:
- [S4] ethereum/devp2p `caps/eth.md` (master), messages `0x08`–`0x0a`, Change Log eth/68 — verified. https://github.com/ethereum/devp2p/blob/master/caps/eth.md
- [S11] ethereum/devp2p `rlpx.md`, Framing, Message Encoding, Message ID-based Multiplexing — verified. https://github.com/ethereum/devp2p/blob/master/rlpx.md
- [S5] EIP-5793, Specification — verified. https://eips.ethereum.org/EIPS/eip-5793
- [S12] ethereum.org RLP, Definition — verified. https://ethereum.org/developers/docs/data-structures-and-encoding/rlp/

# eth/68 transaction propagation

Master `caps/eth.md` is **eth/72**. eth/68 is the changelog shape: do not copy the current `0x08` body, whose `cells: B_16` field arrived in eth/72 [S4]. Spec tokens `B` / `P` / `B_32` have no glossary in the fetched file. `P` is a big-endian integer with no leading zeros; integer 0 is `0x80` [S12].

| Code | eth/68 `msg-data` (an RLP list) |
| --- | --- |
| `NewPooledTransactionHashes` `0x08` | `[types: B, [size₀: P, …], [hash₀: B_32, …]]`. `types` is one packed byte per tx, not a list; all three elements have equal count; `sizeₙ` is the consensus-encoding length (`tx-type\|\|tx-data`, or RLP `legacy-tx`). No `cells` field [S5; S4 eth/68 + size prose]. |
| `GetPooledTransactions` `0x09` | `[request-id: P, [hash₀: B_32, …]]` [S4]. |
| `PooledTransactions` `0x0a` | `[request-id: P, [tx₀, tx₁, …]]`: same order as the request; unavailable hashes may be skipped; empty iff none match; type-3 blob bytes are elided [S4]. |

`request-id` is a requester-chosen 64-bit integer, mirrored in the response. Recommended soft limits: `0x08` 4096 items; `0x09` 256 hashes [S4].

**Frame**, after Hello [S11]: `header-ciphertext(16) || header-mac(16) || frame-ciphertext || frame-mac(16)`. Header plaintext: 3-byte big-endian `frame-size` || `0xc28080` (RLP `[capability-id=0, context-id=0]` [S11][S12]) || 10 zero bytes. `frame-data = rlp(msg-id) || snappy(msg-data)`; `frame-size` is that length; `frame-ciphertext` is `frame-data` zero-padded to 16 under `aes(aes-secret, …)`. MAC update steps: [S11], not repeated. Reject uncompressed `msg-data` above 16 MiB [S11].

**Wire id:** `0x00–0x0f` are p2p; shared caps then take compact ids from `0x10` in alphabetic order [S11]. If no shared name sorts before `eth`, the offset is `0x10`, so these codes are the single RLP bytes `0x18`, `0x19`, `0x1a` [S11][S12]. Offset when an earlier capability is shared: `unknown`.

## Broadcast vs Announce Mechanics in Geth and Nethermind

Pinned sources (verified by fetching raw files; lines are only valid at these commits):
- [S6] go-ethereum `d01e869d59696d915410b0b5bc374ae1a8b9418e` (2026-10-26): `eth/handler.go`, `eth/protocols/eth/peer.go`.
- [S13] Nethermind `1ee91b547c4cedf47943b5842088bf782ba233b3` (2026-10-26): `src/Nethermind/Nethermind.TxPool/{TxBroadcaster,TransactionExtensions,TxPool}.cs`; `…/Nethermind.Network/P2P/{ProtocolHandlers/SyncPeerProtocolHandlerBase,Subprotocols/Eth/V68/Eth68ProtocolHandler}.cs`.

**Geth: sqrt(peers) rule applies.** In `handler.BroadcastTransactions` (`eth/handler.go:553`), blob txs and txs larger than `txMaxBroadcastSize = 4096` bytes (`:65`) are announce-only (`:572-575`). For every other tx, `broadcastChoice.choosePeers` (`:783`) sorts peers by `siphash(key, selfID‖peerID‖txSender)` (`:786-798`) and keeps the top `ceil(sqrt(len(peers)))` (`:802`). Those peers get full bodies via `AsyncSendTransactions`; every other peer not already known to have the tx gets the hash via `AsyncSendPooledTransactionHashes` (`:587-603`; `peer.go:179`, `:214`). Origin-privacy note: the direct set is keyed by node, peer, and sender only. A sender's non-large txs therefore reach the same √n peers while the peer set and key stay fixed (`:753-755`). `BroadcastTransactions` does not tell local txs apart from relayed ones (`:553-604`).

**Nethermind: no sqrt(peers) split found in this code path.**
- `TxPool.AddCore` (`TxPool.cs:1933`) calls `TxBroadcaster.Broadcast(tx, isPersistent)` (`:1978`, `:2019`). It is persistent only when the tx was submitted with `TxHandlingOptions.PersistentBroadcast` (`:1722-1724`). Which callers set that flag: `unknown` (TODO(verify)).
- Persistent path: `StartBroadcast` → `NotifyPeersAboutLocalTx` (`TxBroadcaster.cs:121-135`, `:445-462`) calls `SendNewTransaction` on **all** peers. `Eth68ProtocolHandler.SendNewTransactionCore` (`Eth68ProtocolHandler.cs:383-395`) sends the full body if `CanBeBroadcast` (non-blob and ≤ 4 KiB, `TransactionExtensions.cs:20`, `:139`), otherwise a single-entry `0x08` announcement.
- Non-persistent (relayed) path: `BroadcastOnce` queues the tx (`TxBroadcaster.cs:141-148`). `TimerOnElapsed` → `NotifyAllPeers(_txsToSend, false)` (`:363-388`, `:392-401`) → `SendNewTransactions(…, sendFullTx: false)` (`SyncPeerProtocolHandlerBase.cs:237-252`) then sends hashes only to every peer (`Eth68ProtocolHandler.cs:397-424`).
- On each new head, `BroadcastPersistentTxs` (`TxBroadcaster.cs:235-277`) re-sends to all peers about `PeerNotificationThreshold`% (default 5, `ITxPoolConfig.cs:11-12`) of persistent txs (`:284-285`). Full bodies go out when `CanBeBroadcast`; the rest, which carry blobs or exceed 4 KiB, go as hashes.

Implication, a project inference rather than a client claim: Nethermind's full-body push of persistent txs to all peers, against hash-only relay, may give a first-spy observer a distinguishable origin signal. Its measured magnitude is `unknown`.

## Gossip Timers, Queue Limits, and Discv4/v5 Interaction

Sources: [S6] geth `d01e869d59696d915410b0b5bc374ae1a8b9418e`; [S13] Nethermind `1ee91b547c4cedf47943b5842088bf782ba233b3` (both 2026-10-26); lines valid only at these commits.

**Announce batching.** Geth has **no periodic announce timer**: `Peer.announceTransactions` (`eth/protocols/eth/broadcast.go:101`) drains its queue whenever the writer is free, packing hashes until `maxTxPacketSize = 100 KiB` (`:28`); per-peer queues hold ≤ `maxQueuedTxs = maxQueuedTxAnns = 4096` (`peer.go:44`, `:48`); one announcement carries ≤ `maxTransactionAnnouncements = 5000` hashes (`protocol.go:55`). Nethermind ticks `TxBroadcaster._timer` every **1000 ms** (`TxBroadcaster.cs:98-101`), rate-limits persistent rebroadcasts to ≥1 s apart (`:70`), and sends ~`PeerNotificationThreshold = 5`% of persistent txs per new head (`ITxPoolConfig.cs:11-12`, `TxBroadcaster.cs:284-285`).

**Fetch timeouts.** Geth `TxFetcher`: wait `txArriveTimeout = 500 ms` after an announce (+`txGatherSlack = 100 ms`) before batching `GetPooledTransactions`, then fail an in-flight request at `txFetchTimeout = 5 s` (`eth/fetcher/tx_fetcher.go:71`, `:75`, `:89`); serving is capped at `maxPooledTxServe = 4096` lookups and `softResponseLimit = 2 MiB` (`eth/protocols/eth/handler.go:36-37`, `:61-65`). Nethermind: 0x08 triggers `RequestPooledTransactions` (`Eth65ProtocolHandler.cs:119`); eth/66 requests expire at `Timeouts.Cleanup = 15 s` (`= Eth 10 s × 1.5`, `Nethermind.Synchronization/Timeouts.cs`; `Nethermind.Network/Timeouts.cs`; `MessageDictionary.cs:34-39`). The 2 s constant (`DefaultFetchHeaderTimeout`) is for header sync, not tx fetch.

**Peer limits (actual defaults, not 25/25).** Geth: `MaxPeers` default **50** (`node/defaults.go:74`); outbound dial cap = `50 / defaultDialRatio(3)` → **16** outbound (`p2p/server.go:50`, `:530-533`), rest accepts inbound. Nethermind: `MaxActivePeers` default **50** (`INetworkConfig.cs:76`) with **+10** margin for outbound attempts (`PeerManager.cs:122`, `:990`), ≤20 new outbound connects/s and 2000 ms connect timeout (`INetworkConfig.cs:128`, `:131`).

**Tx-pool admission gates.** Geth legacy-pool defaults: `PriceLimit = 1` wei, `PriceBump = 10`%, `AccountSlots = 16`, `AccountQueue = 64`, `GlobalQueue = 1024`, `Lifetime = 3 h` (`core/txpool/txpool.go:162-170`); rejected txs never relay. Nethermind: broadcast requires `MaxFeePerGas ≥ 70%` of base fee (`MinBaseFeeThreshold`, `ITxPoolConfig.cs:14-15`, `TxBroadcaster.cs:126-127`); pool cap `Size = 2048` (`ITxPoolConfig.cs:17-18`); gossip policy accepts incoming txs only in `WaitingForBlock` unless `AcceptTxWhenNotSynced` (default **false**) (`SyncedTxGossipPolicy.cs:34-35`, `ITxPoolConfig.cs:114-116`) and gates blob/proof-version mismatches (`SpecDrivenTxGossipPolicy.cs:12-13`).

**discv4/v5.** Geth: discv4 on by default; `--discovery.v5` only applies when explicitly set (`cmd/utils/flags.go:943-944`, `:1522-1523`) → discv5 default **off**. Nethermind: `DiscoveryVersion` default **V4** (`IDiscoveryConfig.cs:142-143`), discv4 loop `DiscoveryInterval = 30000 ms` (`:97-98`), `BondWaitTime = 500 ms` (`:77-78`). In both sources discovery only resolves enodes/ENRs for RLPx dials; no coupling between discovery timers and announce/fetch timers was found in these files — coupling elsewhere: `unknown`.
