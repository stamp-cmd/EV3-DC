//! Module for parsing direct reply
//!
//! # Example
//! Show information about reply
//! ```
//! let buf: Vec<u8> = vec![]; // Direct reply vector
//! let rep = Reply::parse(&buf);
//! println!("Length: {}, Id: {}, Error: {}, Memory: {:?}", rep.length(), rep.id(), rep.error(), rep.memory());
//! ```

use crate::DataType;

/// Reply object
pub struct Reply {
    length: u16,
    id: u16,
    error: bool,
    memory: Vec<u8>
}

impl Reply {
    /// Parse direct reply packet
    pub fn parse(packet: &[u8]) -> Self {
        let len = u16::from_le_bytes([packet[0], packet[1]]);
        let rid = u16::from_le_bytes([packet[2], packet[3]]);
        let err = packet[4] == 0x20;
        let mem = packet[5..].to_vec();
        Reply { length: len, id: rid, error: err, memory: mem }
    }
    /// Get reply's length excluding first 2 bytes
    pub fn length(&self) -> u16 { self.length }
    /// Get reply's id. Command and reply match up if they have same id
    pub fn id(&self) -> u16 { self.id }
    /// Check reply's error
    pub fn error(&self) -> bool { self.error }
    /// Get reply's global memory
    pub fn memory(&self) -> &[u8] { &self.memory }
}

/// Extract n bytes from specific [`DataType`] 
pub fn extract_data<T: Iterator<Item = u8>>(bytes: &mut T, dtype: DataType) -> Vec<u8> {
    let len = match dtype {
        DataType::DATA8 => 1,
        DataType::DATA16 => 2,
        DataType::DATA32 | DataType::DATAF => 4,
        DataType::DATAN(length) | DataType::DATAS(length) => length,
    };
    bytes.by_ref().take(len).collect::<Vec<u8>>()
}
