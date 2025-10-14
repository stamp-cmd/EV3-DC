use hidapi::HidApi;
use ev3_dc::{ Command, PID, VID, PORT, Parameter, Encoding };
use ev3_dc::utils;
use ev3_dc::parser;

fn main() {
    let hid = HidApi::new().expect("Cannot create HID context");
    let dev = hid.open(VID, PID).expect("No such device");
    let mut cmd = Command::new();
    let mut bytes = utils::ChainByte::new();
    bytes.extend(vec![0x84, 0x13])
        .extend(Parameter::encode(Encoding::LC0(0)).unwrap())
        .extend(Parameter::encode(Encoding::LC0(0)).unwrap())
        .extend(Parameter::encode(Encoding::LC0(0)).unwrap())
        .extend(vec![0x84, 0x09])
        .extend(Parameter::encode(Encoding::LC0(1)).unwrap())
        .extend(Parameter::encode(Encoding::LC0(20)).unwrap())
        .extend(Parameter::encode(Encoding::LC0(20)).unwrap())
        .extend(Parameter::encode(Encoding::LC1(50)).unwrap())
        .extend(Parameter::encode(Encoding::LC1(50)).unwrap())
        .extend(vec![0x84, 0x00]);
    cmd.bytecode = bytes.bytes;
    let mut buf: Vec<u8> = vec![0; 5 + cmd.reserved_bytes()];
    println!("SENT: {:?}", cmd.gen_bytes());
    dev.write(&cmd.gen_bytes()).unwrap();
    let _ = dev.read(&mut buf).unwrap();
    let reply = parser::Reply::parse(&buf);
    println!("RECV: {} | {} | {} | {:?}", reply.length(), reply.id(), reply.error(), reply.memory());
}
