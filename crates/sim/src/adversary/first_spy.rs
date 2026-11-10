use crate::node::NodeId;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FirstSpyEstimator {
    observations: BTreeMap<NodeId, (u64, NodeId)>,
}

impl FirstSpyEstimator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_observation(&mut self, sybil_id: NodeId, peer_id: NodeId, timestamp_ms: u64) {
        let observation = (timestamp_ms, peer_id);
        self.observations
            .entry(sybil_id)
            .and_modify(|first| *first = (*first).min(observation))
            .or_insert(observation);
    }

    pub fn estimate_origin(&self) -> Option<NodeId> {
        self.observations
            .iter()
            .map(|(&sybil_id, &(timestamp_ms, peer_id))| (timestamp_ms, sybil_id, peer_id))
            .min()
            .map(|(_, _, peer_id)| peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_observations_has_no_estimate() {
        assert_eq!(FirstSpyEstimator::new().estimate_origin(), None);
        assert_eq!(FirstSpyEstimator::default().estimate_origin(), None);
    }

    #[test]
    fn returns_delivering_peer_not_sybil_observer() {
        let mut estimator = FirstSpyEstimator::new();
        estimator.record_observation(20, 3, 50);

        assert_eq!(estimator.estimate_origin(), Some(3));
        assert_eq!(estimator.observations[&20], (50, 3));
    }

    #[test]
    fn selects_earliest_timestamp_across_sybils_not_call_order() {
        let mut estimator = FirstSpyEstimator::new();
        estimator.record_observation(20, 4, 50);
        estimator.record_observation(22, 7, 10);
        estimator.record_observation(21, 2, 25);

        assert_eq!(estimator.observations.len(), 3);
        assert_eq!(estimator.estimate_origin(), Some(7));
    }

    #[test]
    fn retains_only_earliest_observation_for_each_sybil() {
        let mut estimator = FirstSpyEstimator::new();
        estimator.record_observation(20, 4, 50);
        estimator.record_observation(20, 7, 10);
        estimator.record_observation(20, 2, 25);
        estimator.record_observation(21, 9, 30);

        assert_eq!(estimator.observations.len(), 2);
        assert_eq!(estimator.observations[&20], (10, 7));
        assert_eq!(estimator.observations[&21], (30, 9));
        assert_eq!(estimator.estimate_origin(), Some(7));
    }

    #[test]
    fn ties_are_deterministic_regardless_of_recording_order() {
        let observations = [(21, 1, 10), (20, 9, 10), (20, 3, 10), (22, 0, 10)];
        let mut forward = FirstSpyEstimator::new();
        let mut reverse = FirstSpyEstimator::new();
        for &(sybil_id, peer_id, timestamp_ms) in &observations {
            forward.record_observation(sybil_id, peer_id, timestamp_ms);
        }
        for &(sybil_id, peer_id, timestamp_ms) in observations.iter().rev() {
            reverse.record_observation(sybil_id, peer_id, timestamp_ms);
        }

        assert_eq!(forward, reverse);
        assert_eq!(forward.estimate_origin(), Some(3));
    }

    #[test]
    fn duplicate_observations_are_idempotent() {
        let mut estimator = FirstSpyEstimator::new();
        estimator.record_observation(20, 4, 50);
        let before = estimator.clone();

        estimator.record_observation(20, 4, 50);

        assert_eq!(estimator, before);
    }

    #[test]
    fn supports_zero_and_maximum_timestamps() {
        let mut estimator = FirstSpyEstimator::new();
        estimator.record_observation(20, 4, u64::MAX);
        assert_eq!(estimator.estimate_origin(), Some(4));

        estimator.record_observation(21, 0, 0);
        assert_eq!(estimator.estimate_origin(), Some(0));
    }
}
