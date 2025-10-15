//! Visualization / utilities functions for ev3-dc

use super::{ Encoding::*, encode };

#[derive(Default)]
pub struct ChainByte {
    pub bytes: Vec<u8>
}

const LEN_MAX: u16 = 1000; // LIMIT: Practical limit is 1000 for some reason.

/// Chainable byte vector
// maybe use velcro or vek instead
impl ChainByte {
    pub fn new() -> Self {
        ChainByte { bytes: vec![] }
    }
    /// Same as `Vec.push()`, but chainable
    pub fn push(&mut self, byte: u8) -> &mut Self {
        self.bytes.push(byte);
        self
    }
    // `Vec.extend()`, but chainable
    pub fn add(&mut self, bytes: Vec<u8>) -> &mut Self {
        self.bytes.extend(bytes);
        self
    }
}

/// Package vector of bytecodes into vector of larger bytecodes
pub fn package_bytes(bytecodes: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut packets: Vec<Vec<u8>> = vec![];
    let mut buffer: Vec<u8> = vec![];
    let mut size: u16 = 0;
    for bytes in bytecodes {
       if size + (bytes.len() as u16) > LEN_MAX {
            packets.push(buffer.clone()); // seems expensive
            buffer.clear();
            size = 0;
        }
        buffer.extend(bytes);
        size += bytes.len() as u16;
    }
    packets.push(buffer);
    packets
}

/// Run-Length-Encoding on 1D 178x128 image array.
/// Return (x1, y1, x2, y2) line.
pub fn run_length(image: &[u8; 178 * 128]) -> Vec<(u8, u8, u8, u8)> {
    let mut state;
    let mut buffer: Vec<(u8, u8, u8, u8)> = vec![];
    let mut line: (u8, u8, u8, u8) = (0, 0, 0, 0);
    for y in 0..128 {
        state = false;
        for x in 0..178 {
            if image[178 * y + x] == 1 && !state {
                state = true;
                line.0 = x as u8;
                line.1 = y as u8;
            }else if image[178 * y + x] == 0 && state {
                state = false;
                line.2 = (x - 1) as u8;
                line.3 = y as u8;
                buffer.push(line);
            }else if image[178 * y + x] == 1 && x == 177 && state {
                line.2 = 177;
                line.3 = y as u8;
                buffer.push(line);
            }
        }
    }
    buffer
}

pub fn printer(lines: &[(u8, u8, u8, u8)]) -> Vec<Vec<u8>> {
    let mut packets: Vec<Vec<u8>> = vec![];
    for line in lines {
        let mut bytecode = ChainByte::new();
        if line.0 == line.2 {
            bytecode.add(vec![0x84, 0x02, 0x01])
                .add(encode(LC2(line.0 as i16)).unwrap())
                .add(encode(LC2(line.1 as i16)).unwrap());
        }else {
            bytecode.add(vec![0x84, 0x03, 0x01])
                .add(encode(LC2(line.0 as i16)).unwrap())
                .add(encode(LC2(line.1 as i16)).unwrap())
                .add(encode(LC2(line.2 as i16)).unwrap())
                .add(encode(LC2(line.3 as i16)).unwrap());
        }
        packets.push(bytecode.bytes);
    }
    packets
}
