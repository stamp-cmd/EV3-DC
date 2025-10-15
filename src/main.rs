use hidapi::{HidApi, HidDevice};
use ev3_dc::{ Command, PID, VID, PORT, encode, Encoding::* };
use ev3_dc::utils::{self, ChainByte};
use ev3_dc::parser;
use std::{ thread, time::Duration };
use ev3_dc::image::IMAGE;

fn comm(packet: &[u8], buffer: &mut [u8], dev: &HidDevice) {
    println!("SENT: {:02X?}", packet);
    let _ = dev.write(packet);
    let _ = dev.read(buffer);
    println!("RECV: {:02X?}", buffer);
}

fn main() {
    let hid = HidApi::new().expect("Cannot create HID context");
    let dev = hid.open(VID, PID).expect("No such device");
    let mut cmd = Command::new();
    let mut buf: Vec<u8> = vec![0; 5 + cmd.reserved_bytes()];
    let mut byte = ChainByte::new();
    byte.add(vec![0x84, 0x13])
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(vec![0x84, 0x12])
        .add(encode(LC0(0)).unwrap());
    cmd.bytecode = byte.bytes;
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    let lines = utils::run_length(&IMAGE);
    // for line in lines {
        // println!("({}, {}) -> ({}, {})", line.0, line.1, line.2, line.3);
    // }
    let packet = utils::printer(&lines);
    let com = utils::package_bytes(&packet);
    for pack in com {
        cmd.bytecode = pack;
        comm(&cmd.gen_bytes(), &mut buf, &dev);
    }
    cmd.bytecode = vec![0x84, 0x00];
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    // let _ = dev.read(&mut buf).unwrap();
    // let reply = parser::Reply::parse(&buf);
    // println!("RECV: {} | {} | {} | {:?}", reply.length(), reply.id(), reply.error(), reply.memory());
}
