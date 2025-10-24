//! Visualization / utilities functions for ev3-dc

use super::{ Encoding::*, encode, ValError };

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
pub fn run_length(image: &[u8]) -> Result<Vec<(u8, u8, u8, u8)>, ValError> {
    if image.len() != (178 * 128) { return Err(ValError::InvalidValue(image.len() as i32, 178 * 128)) } 
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
    Ok(buffer)
}

/// Convert vector of points from `run_length` to direct commands
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

/// Return name of device id
pub fn device_id(byte: u8) -> String {
    String::from(match byte {
        7 => "Large-Motor",
        8 => "Medium-Motor",
        16 => "Touch-Sensor",
        29 => "Color-Sensor",
        30 => "Ultrasonic-Sensor",
        32 => "Gyro-Sensor",
        33 => "IR-Sensor",
        126 => "None",
        127 => "Port-Error",
        _ => todo!("ID: {} Unimplemented!", byte) // For now, only support EV3 devices
    })
}

/// Read port from u8 slice. 0-3 are inputs, 4-7 are outputs
pub fn port_read(port: &[u8], layer: u8) -> Result<[u8; 8], ValError> {
    let mut ports = [0_u8; 8];
    if layer > 3 { return Err(ValError::InvalidRange(layer as i32, 0, 3)) }
    if port.len() < (20 + (layer * 4)) as usize { return Err(ValError::InvalidRange(port.len() as i32, (20 + (layer * 4)) as i32, 32)); }
    ports[0..4].copy_from_slice(&port[((layer * 4) as usize)..(((layer + 1) * 4) as usize)]);
    ports[4..8].copy_from_slice(&port[(16 + (layer * 4) as usize)..(16 + ((layer + 1) * 4) as usize)]);
    Ok(ports)
}
