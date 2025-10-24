//! Module for parsing direct reply
use crate::DataType;

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
    pub fn length(&self) -> u16 { self.length }
    pub fn id(&self) -> u16 { self.id }
    pub fn error(&self) -> bool { self.error }
    pub fn memory(&self) -> &Vec<u8> { &self.memory }
}

pub fn extract_data<T: Iterator<Item = u8>>(bytes: T, dtype: DataType) -> Vec<u8> {
    let len = match dtype {
        DataType::DATA8 => 1,
        DataType::DATA16 => 2,
        DataType::DATA32 | DataType::DATAF => 4,
        DataType::DATAN(length) => length
    };
    bytes.take(len).collect::<Vec<u8>>()
}
