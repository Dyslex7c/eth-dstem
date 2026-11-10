use dstem::types::{PeerId, ProtocolCapability, TxHash};
use std::collections::HashSet;

#[test]
fn tx_hash_and_peer_id_format_and_convert_bytes() {
    let (tx, peer): (TxHash, PeerId) = ([0xab; 32], [0x01; 64]);
    assert_eq!(format!("{tx:?}").matches("171").count(), 32);
    assert_eq!(format!("{peer:x?}").matches('1').count(), 64);
    assert_eq!(tx.as_slice(), [0xab; 32]);
    assert_eq!(TxHash::try_from(peer[..32].to_vec()).unwrap(), [0x01; 32]);
}

#[test]
fn capability_identifies_eth68s_version_one() {
    let cap = ProtocolCapability::new();
    assert_eq!(cap, ProtocolCapability::default());
    assert_eq!(&cap.name[..6], b"eth68s");
    assert_eq!(cap.version, 1);
}

#[test]
fn tx_hash_is_stable_as_a_hash_set_key() {
    let tx: TxHash = [0x2a; 32];
    let mut seen: HashSet<TxHash> = HashSet::new();
    assert!(seen.insert(tx));
    assert!((0..100).all(|_| seen.contains(&tx) && !seen.insert(tx)));
    assert_eq!(seen.len(), 1);
    assert!(seen.remove(&tx) && !seen.contains(&tx));
}
