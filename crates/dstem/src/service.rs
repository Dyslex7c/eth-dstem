//! Sources: [S1] stem coin + fail-safe; [S14] crates/sim/src/stem/config.rs:12. `Ok(None)` = fluff.
use crate::{peers::PeerRegistry, route::StemRouteTable, tracker::*, types::*};
use std::{sync::OnceLock, time::Instant};
pub const FAIL_SAFE_TIMEOUT_MS: u64 = 8_000;
pub type StemOutcome = Result<Option<PeerId>, &'static str>;
#[derive(Debug, Default)]
pub struct StemService {
    pub registry: PeerRegistry,
    pub route_table: StemRouteTable,
    pub tracker: InFlightTracker,
    epoch: OnceLock<Instant>,
}
impl StemService {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn now_ms(&self) -> u64 {
        self.epoch.get_or_init(Instant::now).elapsed().as_millis() as u64
    }
    pub fn handle_inbound_stem(&self, tx: TxHash, stem_prob: f64) -> StemOutcome {
        match StemRouteTable::select_forwarding_peer(&self.route_table, &mut rand::thread_rng()) {
            _ if !(0.0..=1.0).contains(&stem_prob) => Err("stem_prob must be within [0, 1]"),
            Some(next) if rand::random::<f64>() < stem_prob => {
                self.tracker.track(InFlightRecord {
                    tx_hash: tx,
                    forwarded_to: next,
                    deadline_ms: self.now_ms() + FAIL_SAFE_TIMEOUT_MS,
                });
                Ok(Some(next))
            }
            _ => Ok(None),
        }
    }
}
