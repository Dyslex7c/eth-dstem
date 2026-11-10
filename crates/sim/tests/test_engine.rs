use eth_tx_sim::engine::SimulationEngine;
use eth_tx_sim::event::{Event, EventType};

#[test]
fn steps_pop_events_in_strict_time_order_and_track_clock() {
    let mut engine = SimulationEngine::new();
    assert_eq!(engine.current_time_ms, 0);

    for (scheduled_time_ms, tx_hash) in [(50, 1), (10, 2), (25, 3)] {
        engine.schedule(Event {
            scheduled_time_ms,
            from: scheduled_time_ms as usize,
            to: 0,
            payload: EventType::AnnounceHash {
                tx_hash: [tx_hash; 32],
            },
        });
    }
    assert_eq!(engine.queue.len(), 3);

    for (expected_time_ms, expected_from, expected_tx_hash) in
        [(10, 10, 2), (25, 25, 3), (50, 50, 1)]
    {
        let event = engine
            .step()
            .unwrap_or_else(|| panic!("expected an event at {expected_time_ms}ms"));

        assert_eq!(event.scheduled_time_ms, expected_time_ms);
        assert_eq!(event.from, expected_from);
        assert_eq!(
            event.payload,
            EventType::AnnounceHash {
                tx_hash: [expected_tx_hash; 32]
            }
        );
        assert_eq!(engine.current_time_ms, expected_time_ms);
    }

    assert!(engine.queue.is_empty());
    assert_eq!(engine.step(), None);
    assert_eq!(engine.current_time_ms, 50);
}
