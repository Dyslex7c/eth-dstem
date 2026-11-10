use crate::node::NodeId;
use crate::topology::NetworkTopology;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimingEstimator {
    observations: BTreeMap<NodeId, u64>,
}

impl TimingEstimator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_observation(&mut self, sybil_id: NodeId, timestamp_ms: u64) {
        self.observations
            .entry(sybil_id)
            .and_modify(|first| *first = (*first).min(timestamp_ms))
            .or_insert(timestamp_ms);
    }

    pub fn score_candidate_origin(
        &self,
        topo: &NetworkTopology,
        candidate: NodeId,
        expected_latency_ms: u64,
    ) -> f64 {
        if candidate >= topo.nodes.len() || self.observations.is_empty() {
            return f64::INFINITY;
        }

        let mut distances = vec![None; topo.nodes.len()];
        let mut queue = VecDeque::from([candidate]);
        distances[candidate] = Some(0usize);

        while let Some(node) = queue.pop_front() {
            let next_distance = distances[node].unwrap() + 1;
            for &neighbor in topo.adjacency.get(&node).into_iter().flatten() {
                if let Some(distance) = distances.get_mut(neighbor) {
                    if distance.is_none() {
                        *distance = Some(next_distance);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        let mut score = 0.0;
        for (&sybil_id, &observed_time_ms) in &self.observations {
            let Some(hops) = distances.get(sybil_id).copied().flatten() else {
                return f64::INFINITY;
            };
            let expected_time_ms = (hops as u128) * u128::from(expected_latency_ms);
            let error = expected_time_ms.abs_diff(u128::from(observed_time_ms)) as f64;
            score += error * error;
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeType;

    fn chain() -> NetworkTopology {
        let mut topo = NetworkTopology::new(4, 3);
        for (a, b) in [(0, 1), (1, 2), (2, 3)] {
            topo.add_symmetric_link(a, b).unwrap();
        }
        topo.nodes[2].node_type = NodeType::Sybil;
        topo.nodes[3].node_type = NodeType::Sybil;
        topo
    }

    #[test]
    fn lower_score_favors_candidate_matching_all_observers() {
        let topo = chain();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(2, 20);
        estimator.record_observation(3, 30);

        assert_eq!(estimator.score_candidate_origin(&topo, 0, 10), 0.0);
        assert_eq!(estimator.score_candidate_origin(&topo, 1, 10), 200.0);
    }

    #[test]
    fn sums_squared_errors_instead_of_absolute_errors() {
        let topo = chain();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(2, 23);
        estimator.record_observation(3, 26);

        assert_eq!(estimator.score_candidate_origin(&topo, 0, 10), 25.0);
    }

    #[test]
    fn uses_shortest_path_through_cycles() {
        let mut topo = chain();
        topo.add_symmetric_link(0, 3).unwrap();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(2, 20);
        estimator.record_observation(3, 10);

        assert_eq!(estimator.score_candidate_origin(&topo, 0, 10), 0.0);
    }

    #[test]
    fn retains_earliest_arrival_regardless_of_recording_order() {
        let topo = chain();
        let observations = [(2, 50), (3, 40), (2, 20), (3, 30), (2, 20)];
        let mut forward = TimingEstimator::new();
        let mut reverse = TimingEstimator::new();
        for &(sybil, timestamp) in &observations {
            forward.record_observation(sybil, timestamp);
        }
        for &(sybil, timestamp) in observations.iter().rev() {
            reverse.record_observation(sybil, timestamp);
        }

        assert_eq!(forward, reverse);
        assert_eq!(forward.observations.len(), 2);
        assert_eq!(forward.score_candidate_origin(&topo, 0, 10), 0.0);
    }

    #[test]
    fn no_observations_or_invalid_candidate_cannot_win() {
        let topo = chain();
        let mut estimator = TimingEstimator::default();
        assert!(estimator.score_candidate_origin(&topo, 0, 10).is_infinite());

        estimator.record_observation(2, 20);
        for candidate in [topo.nodes.len(), usize::MAX] {
            assert!(estimator
                .score_candidate_origin(&topo, candidate, 10)
                .is_infinite());
        }
        let empty = NetworkTopology::new(0, 0);
        assert!(estimator
            .score_candidate_origin(&empty, 0, 10)
            .is_infinite());
    }

    #[test]
    fn disconnected_or_invalid_observers_make_score_infinite() {
        let mut topo = NetworkTopology::new(4, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(1, 10);
        assert_eq!(estimator.score_candidate_origin(&topo, 0, 10), 0.0);

        estimator.record_observation(3, 20);
        assert!(estimator.score_candidate_origin(&topo, 0, 10).is_infinite());
        let mut invalid = TimingEstimator::new();
        invalid.record_observation(usize::MAX, 10);
        assert!(invalid.score_candidate_origin(&topo, 0, 10).is_infinite());
    }

    #[test]
    fn zero_latency_and_zero_hop_distance_are_supported() {
        let topo = chain();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(2, 0);
        assert_eq!(estimator.score_candidate_origin(&topo, 2, 10), 0.0);

        estimator.record_observation(3, 5);
        assert_eq!(estimator.score_candidate_origin(&topo, 0, 0), 25.0);
    }

    #[test]
    fn large_timestamps_do_not_overflow_or_lose_small_errors() {
        let topo = chain();
        let mut estimator = TimingEstimator::new();
        estimator.record_observation(2, u64::MAX - 1);
        assert_eq!(estimator.score_candidate_origin(&topo, 1, u64::MAX), 1.0);

        let expected_error = u64::MAX as f64 + 1.0;
        assert_eq!(
            estimator.score_candidate_origin(&topo, 0, u64::MAX),
            expected_error * expected_error
        );
    }
}
