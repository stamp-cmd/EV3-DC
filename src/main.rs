use hidapi::{HidApi, HidDevice};
use ev3_dc::{ Command, PID, VID, encode, Encoding::*, DataType::* };
use ev3_dc::utils::ChainByte;

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
    // byte.push(0x98)
    //     .add(encode(LC1(33)).unwrap())
    //     .add(cmd.allocate(DATAN(33), true).unwrap())
    //     .add(cmd.allocate(DATA8, true).unwrap());
    byte.add(vec![0x81, 0x03])
        .add(encode(LC1(20)).unwrap())
        .add(cmd.allocate(DATAN(20), true).unwrap());
    cmd.bytecode = byte.bytes;
    let mut buf: Vec<u8> = vec![0; 5 + cmd.reserved_bytes()];
    comm(&cmd.gen_bytes(), &mut buf, &dev);
}
