#![doc = "Sources: user-specified dstem type layout; docs/eth68s-stem-spec.md §1 (proposed eth68s capability)."]

pub type TxHash = [u8; 32];
pub type PeerId = [u8; 64];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolCapability {
    pub name: [u8; 8],
    pub version: u8,
}

impl Default for ProtocolCapability {
    fn default() -> Self {
        Self {
            name: *b"eth68s\0\0",
            version: 1,
        }
    }
}

impl ProtocolCapability {
    pub fn new() -> Self {
        Self::default()
    }
}
