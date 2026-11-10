//! Sources: [S4] eth/68 `0x08` payload; [S11] `msg-id || snappy(msg-data)`; [S12] RLP rules.
use crate::types::TxHash;
use bytes::BytesMut;
pub const STEM_POOLED_TX_HASHES_MSG_ID: u8 = 0x0b;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StemAnnouncementPacket {
    pub types: Vec<u8>,
    pub sizes: Vec<u32>,
    pub hashes: Vec<TxHash>,
}
impl StemAnnouncementPacket {
    pub fn encode(&self) -> BytesMut {
        let sizes: Vec<u8> = self.sizes.iter().flat_map(|s| enc_u32(*s)).collect();
        let hashes: Vec<u8> = self.hashes.iter().flat_map(|h| rlp(0x80, h)).collect();
        let mut items = rlp(0x80, &self.types);
        items.extend(rlp(0xc0, &sizes).into_iter().chain(rlp(0xc0, &hashes)));
        let mut out = vec![STEM_POOLED_TX_HASHES_MSG_ID];
        out.extend(rlp(0xc0, &items));
        BytesMut::from(&out[..])
    }
}
fn enc_u32(s: u32) -> Vec<u8> {
    rlp(0x80, &s.to_be_bytes()[(s.leading_zeros() / 8) as usize..])
}
fn rlp(bias: u8, body: &[u8]) -> Vec<u8> {
    let (len, be) = (body.len(), body.len().to_be_bytes());
    let zeros = be.iter().take_while(|&&byte| byte == 0).count();
    let head: Vec<u8> = match (len < 56, bias == 0x80 && len == 1 && body[0] < 0x80) {
        (_, true) => Vec::new(),
        (true, _) => vec![bias + len as u8],
        _ => [vec![bias + 55 + (8 - zeros) as u8], be[zeros..].to_vec()].concat(),
    };
    [head, body.to_vec()].concat()
}
