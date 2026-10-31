/// A deterministic digest of exact workspace source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceDigest(u64);

impl SourceDigest {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        }))
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
