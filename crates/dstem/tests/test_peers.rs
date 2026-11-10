//! Sources: [S6] geth peer set membership; [S4] per-peer `eth68s` cap negotiation.
use dstem::peers::PeerRegistry;
use std::sync::Arc;
use std::thread;

#[test]
fn fifty_peers_across_four_threads_then_remove_ten() {
    let reg = Arc::new(PeerRegistry::default());
    let handles: Vec<_> = (0..4u64)
        .map(|t| {
            let r = Arc::clone(&reg);
            thread::spawn(move || {
                for i in t * 50 / 4..(t + 1) * 50 / 4 {
                    assert!(r.add_peer([i as u8; 64]), "no duplicates");
                }
            })
        })
        .collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
    assert_eq!(reg.peer_count(), 50, "exactly 50, none lost");
    for i in 0..10u64 {
        assert!(reg.remove_peer(&[i as u8; 64]), "remove hits");
    }
    assert_eq!(reg.peer_count(), 40, "decremented to 40");
    for i in 0..10u64 {
        assert!(!reg.remove_peer(&[i as u8; 64]), "already gone");
    }
    assert_eq!(reg.peer_count(), 40, "no over-removal");
}
