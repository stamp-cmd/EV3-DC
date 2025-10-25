//! Command generation function.
//! Not all command will be here, only some of them.

use crate::{ encode, Command, DataType, Encoding::*, ValError };
use crate::utils::ChainByte;

/// LED Color
pub enum LedColor {
   Red,
   Orange,
   Green,
   Off
}

/// LED Animation
pub enum LedEffect {
    Static,
    Blink,
    Pulse
}

/// Rotate motor with speed
pub fn motor_speed(port: u8, speed: i8, layer: u8) -> Result<Vec<u8>, ValError> {
    if port > 15 { return Err(ValError::InvalidRange(port as i32, 0, 15)); }
    if !(-100..=100).contains(&speed) { return Err(ValError::InvalidRange(speed as i32, -100, 100)) }
    if layer > 3 { return Err(ValError::InvalidRange(layer as i32, 0, 3)) }
    let mut byte = ChainByte::new();
    byte.push(0xA5)
        .add(encode(LC0(layer as i8))?)
        .add(encode(LC0(port as i8))?)
        .add(encode(LC1(speed))?)
        .push(0xA6)
        .add(encode(LC0(layer as i8))?)
        .add(encode(LC0(port as i8))?);
    Ok(byte.bytes)
}

/// Stop motor at port 
pub fn stop_motor(port: u16, layer: u8, hard: bool) -> Result<Vec<u8>, ValError> {
    if port > 15 { return Err(ValError::InvalidRange(port as i32, 0, 15)); }
    if layer > 3 { return Err(ValError::InvalidRange(layer as i32, 0, 3)); }
    let mut byte = ChainByte::new();
    byte.push(0xA3)
        .add(encode(LC0(layer as i8))?)
        .add(encode(LC0(port as i8))?)
        .add(encode(LC0(match hard {
            true => 1, false => 0
        }))?);
    Ok(byte.bytes)
}

/// Get battery percentage
/// Return bytecodes and vector of `DataType`
pub fn battery_percentage(cmd: &mut Command) -> Result<(Vec<u8>, Vec<DataType>), ValError> {
    let mut byte = ChainByte::new();
    byte.add(vec![0x81, 0x12])
        .add(cmd.allocate(DataType::DATA8, true)?);
    Ok((byte.bytes, vec![DataType::DATA8]))
}

/// Show LED
pub fn show_led(color: LedColor, effect: LedEffect) -> Vec<u8> {
    let mut byte = ChainByte::new();
    byte.add(vec![0x82, 0x1B]);
    if let LedColor::Off = color {
        byte.add(encode(LC0(0)).unwrap());
    }
    let mut code: i8 = 0;
    code += match color {
        LedColor::Green => 1,
        LedColor::Red => 2,
        LedColor::Orange => 3,
        _ => 0
    };
    code += match effect {
        LedEffect::Static => 0,
        LedEffect::Blink => 3,
        LedEffect::Pulse => 6
    };
    byte.add(encode(LC0(code)).unwrap());
    byte.bytes
}


