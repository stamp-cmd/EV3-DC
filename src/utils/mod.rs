//! Visualization / utilities for ev3 dc

#[derive(Default)]
pub struct ChainByte {
    pub bytes: Vec<u8>
}

/// Chainable byte vector
impl ChainByte {
    pub fn new() -> Self {
        ChainByte { bytes: vec![] }
    }
    /// Same as `Vec.push()`, but returns itself
    pub fn push(&mut self, byte: u8) -> &mut Self {
        self.bytes.push(byte);
        self
    }
    // Same as `Vec.extend()`, but returns itself
    pub fn extend(&mut self, bytes: Vec<u8>) -> &mut Self {
        self.bytes.extend(bytes);
        self
    }
}
