use crate::event::Event;
use std::collections::BinaryHeap;

#[derive(Clone, Debug, Default)]
pub struct SimulationEngine {
    pub current_time_ms: u64,
    pub queue: BinaryHeap<Event>,
}

impl SimulationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn schedule(&mut self, mut event: Event) {
        event.scheduled_time_ms = event.scheduled_time_ms.max(self.current_time_ms);
        self.queue.push(event);
    }

    pub fn step(&mut self) -> Option<Event> {
        let event = self.queue.pop()?;
        self.current_time_ms = self.current_time_ms.max(event.scheduled_time_ms);
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventType;

    fn event(scheduled_time_ms: u64, from: usize) -> Event {
        Event {
            scheduled_time_ms,
            from,
            to: 0,
            payload: EventType::AnnounceHash { tx_hash: [0; 32] },
        }
    }

    #[test]
    fn new_engine_starts_at_zero_and_empty() {
        let mut engine = SimulationEngine::new();
        assert_eq!(engine.current_time_ms, 0);
        assert!(engine.queue.is_empty());
        assert_eq!(engine.step(), None);
        assert_eq!(engine.current_time_ms, 0);
    }

    #[test]
    fn step_advances_clock_to_each_next_event_time() {
        let mut engine = SimulationEngine::new();
        for (t, from) in [(300, 1), (100, 2), (200, 3)] {
            engine.schedule(event(t, from));
        }

        for (expected_time, expected_from) in [(100, 2), (200, 3), (300, 1)] {
            let popped = engine.step().expect("event queued");
            assert_eq!(popped.scheduled_time_ms, expected_time);
            assert_eq!(popped.from, expected_from);
            assert_eq!(engine.current_time_ms, expected_time);
        }
        assert_eq!(engine.step(), None);
        assert_eq!(engine.current_time_ms, 300);
    }

    #[test]
    fn simultaneous_events_keep_clock_unchanged() {
        let mut engine = SimulationEngine::new();
        engine.schedule(event(50, 1));
        engine.schedule(event(50, 2));

        engine.step().unwrap();
        assert_eq!(engine.current_time_ms, 50);
        engine.step().unwrap();
        assert_eq!(engine.current_time_ms, 50);
    }

    #[test]
    fn events_scheduled_mid_run_are_interleaved_by_time() {
        let mut engine = SimulationEngine::new();
        engine.schedule(event(10, 1));
        engine.schedule(event(40, 2));

        engine.step().unwrap();
        engine.schedule(event(engine.current_time_ms + 5, 3));

        let next = engine.step().unwrap();
        assert_eq!((next.scheduled_time_ms, next.from), (15, 3));
        assert_eq!(engine.current_time_ms, 15);
        assert_eq!(engine.step().unwrap().scheduled_time_ms, 40);
        assert_eq!(engine.current_time_ms, 40);
    }

    #[test]
    fn past_events_are_clamped_so_time_never_moves_backwards() {
        let mut engine = SimulationEngine::new();
        engine.schedule(event(100, 1));
        engine.step().unwrap();

        engine.schedule(event(20, 2));
        let popped = engine.step().unwrap();
        assert_eq!(popped.scheduled_time_ms, 100);
        assert_eq!(engine.current_time_ms, 100);
    }

    #[test]
    fn clock_stays_monotonic_if_queue_is_pushed_directly() {
        let mut engine = SimulationEngine::new();
        engine.schedule(event(100, 1));
        engine.step().unwrap();

        engine.queue.push(event(20, 2));
        assert_eq!(engine.step().unwrap().scheduled_time_ms, 20);
        assert_eq!(engine.current_time_ms, 100);
    }

    #[test]
    fn extreme_timestamps_do_not_overflow() {
        let mut engine = SimulationEngine::new();
        engine.schedule(event(u64::MAX, 1));
        engine.schedule(event(0, 2));

        assert_eq!(engine.step().unwrap().scheduled_time_ms, 0);
        assert_eq!(engine.current_time_ms, 0);
        assert_eq!(engine.step().unwrap().scheduled_time_ms, u64::MAX);
        assert_eq!(engine.current_time_ms, u64::MAX);
    }
}
