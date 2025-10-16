use hidapi::{HidApi, HidDevice};
use ev3_dc::{ Command, PID, VID, PORT, encode, Encoding::*, DataType::* };
use ev3_dc::utils::ChainByte;
use ev3_dc::parser::Reply;

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
    let mut byte = ChainByte::new();
    byte.add(vec![0x81, 0x0A])
        .add(encode(LC0(10)).unwrap())
        .add(cmd.allocate(DATA8, true).unwrap());
    cmd.bytecode = byte.bytes;
    let mut buf = vec![0; 5 + cmd.reserved_bytes()];
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    let rep = Reply::parse(&buf);
    let load = u8::from_le_bytes([rep.memory()[0]]);
    for _ in 0..5 {
        byte = ChainByte::new();
        byte.push(0xC8)
            .add(encode(LC1(load as i8)).unwrap())
            .add(encode(LC0(0)).unwrap())
            .add(cmd.allocate(DATA8, true).unwrap());
        cmd.bytecode = byte.bytes;
        comm(&cmd.gen_bytes(), &mut buf, &dev);
        cmd.free_mem();
    } 
}
