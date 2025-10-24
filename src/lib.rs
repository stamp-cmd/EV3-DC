//! Core crate
use thiserror::Error;
pub mod utils;
pub mod parser;
pub mod funcs;

pub enum DataType {
    DATA8,
    DATA16,
    DATA32,
    DATAF,
    DATAN(usize) // Custom length allocation
}

/// LCx: Local constant value
/// LVx: Local variable address
/// GVx: Global variable address
/// GV4 & LV4 are unusable in direct command
pub enum Encoding<'a> {
    /// LC0 only allow values from -31 to 31
    LC0(i8),
    LC1(i8),
    LC2(i16),
    LC4(i32),
    LCF(f32),
    /// LV0 only allow value up to 31
    LV0(u8),
    LV1(u8),
    LV2(u16),
    /// GV0 only allow value up to 31
    GV0(u8),
    GV1(u8),
    GV2(u16),
    LCS(&'a str),
}
/// The packet that get sent to EV3
/// Can contain more than 1 OpCode
pub struct Command {
    pub id: u16,
    pub reply: bool,
    allocation: u16,
    pub bytecode: Vec<u8>,
}

// Constants
pub const VID: u16 = 0x0694;
pub const PID: u16 = 0x0005;

#[allow(non_snake_case)]
pub struct Port {
    pub A: i8,
    pub B: i8,
    pub C: i8,
    pub D: i8,
    pub ALL: i8
}
/// PORT Constants. Add them together to use multiple ports
pub const PORT: Port = Port { A: 1, B: 2, C: 4, D: 8, ALL: 15};

#[derive(Error, Debug)]
pub enum ValError {
    /// Error for encoding
    #[error("Encode error: value: {0} overflowed (maximum: {1})")]
    PosOverflow(u32, u32),
    #[error("Encode error: value: {0} overflowed (minimum: {1})")]
    NegOverflow(i32, i32),
    /// Error for functions
    #[error("Allocation error: cannot allocate data with sized {0}. Allocated {3} memory: {1} / {2}")]
    MemOverflow(u16, u16, u16, String),
    #[error("Invalid range: expect {1} - {2}, got {0}")]
    InvalidRange(i32, i32, i32),
    #[error("Invalid value: expect {1}, got {0}")]
    InvalidValue(i32, i32)
}

impl Command {
    pub fn new() -> Self { Command::default() }
    /// Generate direct command bytecode
    pub fn gen_bytes(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = vec![0x00, 0x00];
        packet.extend(self.id.to_le_bytes());
        packet.push(match self.reply {
            true => 0x00,
            false => 0x80,
        });
        packet.extend(self.allocation.to_le_bytes());
        packet.extend(&self.bytecode);
        let ln = ((5 + self.bytecode.len()) as u16).to_le_bytes();
        packet[0] = ln[0];
        packet[1] = ln[1];
        packet
    }
    /// Get reserved bytes
    /// Can be use to initialized response buffer, with length of `5 + command.reserved_bytes()`.
    pub fn reserved_bytes(&self) -> usize {
        ((self.allocation >> 10) + (self.allocation & ((1 << 10) - 1))) as usize
    }
    /// Free all allocated memory
    /// Implementation is safe, usage is not.
    /// MAKE SURE YOUR COMMAND'S BYTECODES DOES NOT ALLOCATE ANY MEMORY 
    pub fn unsafe_free(&mut self) {
        self.allocation = 0;
    }
}

/// Encode value to parameter encoding.
/// Use for encoding constant value or encoding address to variable directly.
/// Use `Command::allocate` if you don't want to track current stack address.
pub fn encode(encoding: Encoding) -> Result<Vec<u8>, ValError> {
let mut bytes: Vec<u8> = vec![];
    let mut head: u8 = 0;
    match encoding {
        Encoding::LC0(val) => {
            if val > 31 { return Err(ValError::PosOverflow(val as u32, 31)) }
            if val < -31 { return Err(ValError::NegOverflow(val as i32, -31)) }
            if val < 0 { head += 1 << 5;}
            head += (val.abs() & 0b11111) as u8
        }
        Encoding::GV0(val) | Encoding::LV0(val) => {
            if val > 31 { return Err(ValError::PosOverflow(val as u32, 31)) }
            head += 1 << 6;
            if let Encoding::GV0(_) = encoding { head += 1 << 5; }
            head += val & 0b11111;
        }
        Encoding::LCS(_) => {
            head += 0b10000100;
        }
        _ => { head += 1 << 7; }
    }
    bytes.push(head);
    let res: Result<Vec<u8>, ValError> = match encoding {
        Encoding::LC1(val) => { 
            head += 1;
            if val == i8::MIN { return Err(ValError::NegOverflow(val as i32, i8::MIN as i32)) }
            if val < 0 { head += 1 << 5; }
            Ok((val.abs() & i8::MAX).to_le_bytes().to_vec())
        }
        Encoding::LC2(val) => {
            head += 2;
            if val == i16::MIN { return Err(ValError::NegOverflow(val as i32, i16::MIN as i32)) }
            if val < 0 { head += 1 << 5; }
            Ok((val.abs() & i16::MAX).to_le_bytes().to_vec())
        }
        Encoding::LC4(val) => {
            head += 3;
            if val == i32::MIN { return Err(ValError::NegOverflow(val, i32::MIN)) }
            if val < 0 { head += 1 << 5; }
            Ok((val.abs() & i32::MAX).to_le_bytes().to_vec())
        }
        Encoding::LV1(val) | Encoding::GV1(val) => {
            head += (1 << 5) + 1;
            Ok(val.to_le_bytes().to_vec())
        }
        Encoding::LV2(val) | Encoding::GV2(val) => {
            head += (1 << 5) + 2;
            Ok(val.to_le_bytes().to_vec())
        }
        Encoding::LCS(val) => {
            let mut tmp = val.as_bytes().to_vec();
            tmp.push(0);
            Ok(tmp)
        }
        Encoding::LCF(val) => {
            head += 3;
            Ok(val.to_le_bytes().to_vec())
        }
        _ => Ok(vec![]),
    };
    bytes.extend(res?);
    bytes[0] = head;
    Ok(bytes)
}

impl Command {
    /// Create variable bytecode and allocate space
    /// `global` - Allocate global variable
    pub fn allocate(&mut self, data: DataType, global: bool) -> Result<Vec<u8>, ValError> {
        let local: u8 = (self.allocation >> 10) as u8;
        let glob: u16 = self.allocation & ((1 << 10) - 1);
        let address: u16 = if global { local.into() } else { glob };
        let mem: u16 = match data {
            DataType::DATA8 => 1,
            DataType::DATA16 => 2,
            DataType::DATA32 | DataType::DATAF => 4,
            DataType::DATAN(length) => u16::try_from(length).unwrap()
        };
        if local + (mem as u8) > (1 << 6) - 1 { return Err(ValError::MemOverflow(mem, local as u16, (1 << 6) - 1, "local".to_string())); }
        if glob + mem > (1 << 10) - 1 { return Err(ValError::MemOverflow(mem, glob, (1 << 10) - 1, "global".to_string())); }
        self.allocation += if global { mem } else { mem << 10 };
        match address {
            0..=31 => {
                if global {
                    encode(Encoding::GV0(address as u8))
                } else {
                    encode(Encoding::LV0(address as u8))
                }
            }
            32..=254 => {
                if global {
                    encode(Encoding::GV1(address as u8))
                } else {
                    encode(Encoding::LV1(address as u8))
                }
            }
            _ => {
                if global {
                    encode(Encoding::GV2(address))
                } else {
                    encode(Encoding::LV2(address))
                }
            }
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Command {
            id: 170,
            reply: true, 
            allocation: 0,
            bytecode: vec![],
        }
    }
}
