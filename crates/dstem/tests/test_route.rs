//! Sources: [S1] Dandelion++ uniform stem-peer coin; [S4]/[S6] stem-capable set.
use dstem::route::StemRouteTable;
use dstem::types::PeerId;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn stem_selection_is_uniform_within_440_560() {
    let a: PeerId = [0xaa; 64];
    let b: PeerId = [0xbb; 64];
    let tbl = StemRouteTable::default();
    tbl.update_routes(vec![a, b]);
    let mut rng = StdRng::seed_from_u64(0x1d1e_6574);
    let mut n = [0usize; 2];
    for _ in 0..1000 {
        let p = tbl.select_forwarding_peer(&mut rng).expect("non-empty");
        assert!(p == a || p == b, "unexpected peer selected");
        n[usize::from(p == b)] += 1;
    }
    assert!((440..=560).contains(&n[0]) && (440..=560).contains(&n[1]));
    assert_eq!(n[0] + n[1], 1000, "every draw was A or B");
}

#[test]
fn empty_route_table_returns_none() {
    let mut rng = StdRng::seed_from_u64(0x1d1e_6574);
    let t = StemRouteTable::default();
    assert_eq!(t.select_forwarding_peer(&mut rng), None);
}
