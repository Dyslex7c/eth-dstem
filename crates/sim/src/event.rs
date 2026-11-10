use crate::node::NodeId;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventType {
    AnnounceHash { tx_hash: [u8; 32] },
    FetchRequest { tx_hash: [u8; 32] },
    DeliverTx { tx_hash: [u8; 32] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub scheduled_time_ms: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub payload: EventType,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .scheduled_time_ms
            .cmp(&self.scheduled_time_ms)
            .then_with(|| self.from.cmp(&other.from))
            .then_with(|| self.to.cmp(&other.to))
            .then_with(|| self.payload.cmp(&other.payload))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BinaryHeap;

    fn event(scheduled_time_ms: u64) -> Event {
        Event {
            scheduled_time_ms,
            from: 0,
            to: 1,
            payload: EventType::AnnounceHash { tx_hash: [0; 32] },
        }
    }

    #[test]
    fn heap_pops_lowest_timestamp_first() {
        let mut queue = BinaryHeap::new();
        for timestamp in [50, u64::MAX, 10, 0, 50, 1] {
            queue.push(event(timestamp));
        }

        let mut timestamps = Vec::new();
        while let Some(event) = queue.pop() {
            timestamps.push(event.scheduled_time_ms);
        }
        assert_eq!(timestamps, vec![0, 1, 10, 50, 50, u64::MAX]);
        assert!(queue.is_empty());
    }

    #[test]
    fn timestamp_takes_priority_over_other_fields() {
        let mut earlier = event(1);
        earlier.from = usize::MAX;
        earlier.to = usize::MAX;
        earlier.payload = EventType::DeliverTx { tx_hash: [255; 32] };
        let later = event(2);

        assert!(earlier > later);
        assert_eq!(earlier.partial_cmp(&later), Some(Ordering::Greater));
    }

    #[test]
    fn equal_timestamps_preserve_distinct_events_and_consistent_ordering() {
        let events = [
            event(10),
            Event {
                from: 2,
                ..event(10)
            },
            Event { to: 2, ..event(10) },
            Event {
                payload: EventType::AnnounceHash { tx_hash: [1; 32] },
                ..event(10)
            },
            Event {
                payload: EventType::FetchRequest { tx_hash: [0; 32] },
                ..event(10)
            },
            Event {
                payload: EventType::DeliverTx { tx_hash: [0; 32] },
                ..event(10)
            },
        ];
        for a in &events {
            for b in &events {
                assert_eq!(a.cmp(b) == Ordering::Equal, a == b);
                assert_eq!(a.partial_cmp(b), Some(a.cmp(b)));
                assert_eq!(a.cmp(b), b.cmp(a).reverse());
            }
        }

        let mut forward = BinaryHeap::new();
        let mut reverse = BinaryHeap::new();
        for event in &events {
            forward.push(event.clone());
        }
        for event in events.iter().rev() {
            reverse.push(event.clone());
        }
        assert_eq!(forward.len(), events.len());
        while let Some(event) = forward.pop() {
            assert_eq!(reverse.pop(), Some(event));
        }
        assert!(reverse.is_empty());
    }
}
