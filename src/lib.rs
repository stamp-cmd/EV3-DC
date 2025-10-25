//! Low-level EV3 Direct command library.
//! Library for direct command bytecode generation and basic direct reply parsing.
//!
//! More information about direct commands is available at
//! [LEGO MINDSTORMS Firmware Developer Kit](https://assets.education.lego.com/v3/assets/blt293eea581807678a/blt09ac3101d9df2051/5f88037a69efd81ab4debf2e/lego-mindstorms-ev3-communication-developer-kit.pdf?locale=en-us)
//!
//! # Example
//! ## Show blinking green LED
//! ```
//! use ev3_dc::{ Command, Encoding::*, encode };
//!
//! let mut cmd = Command::new();
//! let mut byte = vec![0x84, 0x1B]; // OpUI_Write, LED
//! byte.extend(encode(LC0(0x04)).unwrap()); // Green flashing
//! cmd.bytecode = byte;
//! println!("SENT: {:02X?}", cmd.gen_bytes());
//! // and send actual bytes via HID, or Bluetooth, etc.
//! ```

use thiserror::Error;
pub mod utils;
pub mod parser;
pub mod funcs;

/// EV3 DataType. DATAN is for custom array
pub enum DataType {
    /// 8-bits value
    DATA8,
    /// 16-bits value
    DATA16,
    /// 32-bits value
    DATA32,
    /// IEEE-754 single precision float i.e. [`f32`]
    DATAF,
    /// Array
    DATAN(usize), // Custom length allocation
    /// Zero-terminated string
    DATAS(usize)
}

/// LCx: Local constant value
///
/// LVx: Local variable address
///
/// GVx: Global variable address
///
/// GV4 & LV4 & LV2 are unusable in direct command
pub enum Encoding<'a> {
    /// 5-bits constant 
    LC0(i8),
    /// 7-bits constant
    LC1(i8),
    /// 15-bits constant 
    LC2(i16),
    /// 31-bits constant
    LC4(i32),
    /// IEEE-754 single precision constant
    LCF(f32),
    /// 5-bits local address
    LV0(u8),
    /// 7-bits local address
    LV1(u8),
    /// 5-bits global address 
    GV0(u8),
    /// 7-bits global address
    GV1(u8),
    /// 15-bits global address
    GV2(u16),
    /// String (auto zero-terminated)
    LCS(&'a str),
}

/// The packets that get sent to EV3. 
/// Can contain more than 1 OpCode
/// # Example
/// ```
/// let mut cmd = Command::new();
/// let mut byte = vec![];
/// // Add bytecode to byte
/// cmd.bytecode = byte;
/// println!("SENT: {:02X?}", cmd.gen_bytes());
/// ```
pub struct Command {
    /// Command ID
    pub id: u16,
    /// Reply to direct command
    pub reply: bool,
    allocation: u16,
    /// Bytes containing OpCodes and Parameter 
    pub bytecode: Vec<u8>,
}

// Constants
/// USB VendorId of EV3
pub const VID: u16 = 0x0694;
/// USB ProductId of EV3
pub const PID: u16 = 0x0005;

#[allow(non_snake_case)]
/// Port struct. Use defined [`PORT`] constant instead.
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
/// ev3_dc Error type
pub enum ValError {
    // Error for encoding
    /// [`encode`] failed to encode overflowed value
    #[error("Encode error: value: {0} overflowed (maximum: {1})")]
    PosOverflow(u32, u32),
    /// [`encode`] failed to encode underflowed value
    #[error("Encode error: value: {0} underflowed (minimum: {1})")]
    NegOverflow(i32, i32),
    // Error for functions
    /// [`Command::allocate`] failed to allocated variable in memory
    #[error("Allocation error: cannot allocate data with sized {0}. Allocated {3} memory: {1} / {2}")]
    MemOverflow(u16, u16, u16, String),
    #[error("Invalid range: expect {1} - {2}, got {0}")]
    /// Value isn't in valid range
    InvalidRange(i32, i32, i32),
    /// Value isn't validi
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
    /// Can be use to initialize response buffer, with length of `5 + reserved_bytes()`.
    pub fn reserved_bytes(&self) -> usize {
        ((self.allocation >> 10) + (self.allocation & ((1 << 10) - 1))) as usize
    }
    /// Free all allocated memory
    /// **Causing any variables in bytecode to not work**
    pub fn mem_free(&mut self) {
        self.allocation = 0;
    }
}

/// Encode value to parameter encoding.
/// For encoding constant value or encoding address to variable directly.
/// Use [`Command::allocate`] to encode variable without specifying pointer directly
/// # Example
/// ```
/// let byte: Vec<u8> = encode(LC1(42)).unwrap();
/// println("Bytecode: {:02X?}", byte);
/// ```
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
        Encoding::GV2(val) => {
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
    /// Create variable bytecode and allocate space in [`Command`]
    /// Allocating global variables allow the values to be read in reply
    pub fn allocate(&mut self, data: DataType, global: bool) -> Result<Vec<u8>, ValError> {
        let local: u8 = (self.allocation >> 10) as u8;
        let glob: u16 = self.allocation & ((1 << 10) - 1);
        let address: u16 = if global { local.into() } else { glob };
        let mem: u16 = match data {
            DataType::DATA8 => 1,
            DataType::DATA16 => 2,
            DataType::DATA32 | DataType::DATAF => 4,
            DataType::DATAN(length) => u16::try_from(length).unwrap(),
            DataType::DATAS(length) => u16::try_from(length + 1).unwrap()
        };
        if local + (mem as u8) > (1 << 7) - 1 { return Err(ValError::MemOverflow(mem, local as u16, (1 << 7) - 1, "local".to_string())); }
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
                }else {
                    Err(ValError::PosOverflow(mem.into(), (1 << 7) - 1)) // Shouldn't be called
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
