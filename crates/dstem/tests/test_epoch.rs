//! Sources: [S1] Dandelion++ per-epoch stem-graph reshuffle (degree = 2).
use dstem::{epoch::reshuffle_stem_routes, peers::PeerRegistry, route::StemRouteTable};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::BTreeSet;

#[test]
fn reshuffle_selects_two_registry_peers_and_varies_across_epochs() {
    let reg = PeerRegistry::default();
    let pool: BTreeSet<_> = (0..20u8).map(|i| [i; 64]).collect();
    for p in &pool {
        assert!(reg.add_peer(*p));
    }
    assert_eq!(reg.peer_count(), 20);
    let tbl = StemRouteTable::default();
    let mut rng = StdRng::seed_from_u64(0x6574_6836_3873);
    let mut epochs = BTreeSet::new();
    for _ in 0..5 {
        reshuffle_stem_routes(&mut rng, &reg, &tbl, 2);
        let set: BTreeSet<_> = (0..64)
            .map(|_| tbl.select_forwarding_peer(&mut rng).unwrap())
            .collect();
        assert_eq!(set.len(), 2);
        assert!(set.is_subset(&pool));
        epochs.insert(set);
    }
    assert!(epochs.len() > 1);
}
