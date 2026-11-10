//! Sources: [S4] eth/68 announcement; [S12] RLP; EIP-2718 (type 2 = EIP-1559, type 3 = EIP-4844).
use dstem::wire::{StemAnnouncementPacket, STEM_POOLED_TX_HASHES_MSG_ID};

#[test]
fn encode_two_typed_txs_starts_with_0x0b_and_exact_payload() {
    let pkt = StemAnnouncementPacket {
        types: vec![2, 3],
        sizes: vec![100, 200],
        hashes: vec![[0xaa; 32], [0xbb; 32]],
    };
    let buf = pkt.encode();
    assert_eq!(buf[0], STEM_POOLED_TX_HASHES_MSG_ID, "prefix 0x0b");
    assert_eq!(STEM_POOLED_TX_HASHES_MSG_ID, 0x0b, "id is 0x0b");
    assert_eq!(buf[1], 0xf8, "outer list head");
    assert_eq!(buf[2] as usize, buf.len() - 3, "payload contiguous");
    let items = &buf[3..];
    assert_eq!(items.len(), 3 + 4 + 68, "sum type+size+hash");
    assert_eq!(&items[0..3], &[0x82, 0x02, 0x03], "type bytes");
    assert_eq!(&items[3..7], &[0xc3, 0x64, 0x81, 0xc8], "sizes");
    assert_eq!(&items[7..10], &[0xf8, 66, 0xa0], "hash list hdr");
    assert_eq!(&items[10..42], &[0xaa; 32], "tx2 hash slice");
    assert_eq!(items[42], 0xa0, "second 32B head");
    assert_eq!(&items[43..75], &[0xbb; 32], "tx3 hash slice");
}
