use hidapi::{ HidApi, HidDevice };
use ev3_dc::{ VID, PID, Command, encode, Encoding::* };
use ev3_dc::utils::{ package_bytes, run_length, printer, ChainByte };
use core::panic;
use std::{ env, fs, io::{BufRead, BufReader, Read}, path::Path } ;

fn comm(packet: &[u8], buffer: &mut [u8], dev: &HidDevice) {
    println!("SENT: {:02X?}", packet);
    let _ = dev.write(packet);
    let _ = dev.read(buffer);
    println!("RECV: {:02X?}", buffer);
}

fn main() {
    let file = fs::File::open(Path::new(&env::args().nth(1).unwrap()))
        .unwrap_or_else(|_| { panic!("File not found: {}", env::args().nth(1).unwrap()) });
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let _ = reader.read_line(&mut buffer);
    if buffer.as_str() != "P1\n" {
        panic!("File header isn't NetPBM file");
    }
    buffer.clear();
    let _ = reader.read_line(&mut buffer);
    while buffer.starts_with("#") {
        buffer.clear();
        let _ = reader.read_line(&mut buffer);
    }
    let mut dim = buffer.split(" ").filter(|x| { !x.is_empty() });
    if dim.next().unwrap() != "178" && dim.next().unwrap() != "128" {
        panic!("File dimension isn't 178x128");
    }
    buffer.clear();
    let _ = reader.read_to_string(&mut buffer);
    let image: Vec<u8> = buffer.split("").map(|x| { match x { "1" => 1, "0" => 0, _ => 2 } }).filter(|x| { *x != 2 }).collect();
    println!("{}", image.len());
    let hid = HidApi::new().expect("Cannot create HID context!");
    let dev = hid.open(VID, PID).unwrap_or_else(|_| { panic!("Device with VID: {} & PID: {} not found!", VID, PID) });
    let mut cmd = Command::new();
    let mut byte = ChainByte::new();
    let mut buf: [u8; 5] = [0; 5];
    byte.add(vec![0x84, 0x13])
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(vec![0x84, 0x12])
        .add(encode(LC0(0)).unwrap());
    cmd.bytecode = byte.bytes;
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    let lines = run_length(&image).unwrap();
    let packets = printer(&lines);
    let packed = package_bytes(&packets);
    for pack in packed {
        cmd.bytecode = pack;
        comm(&cmd.gen_bytes(), &mut buf, &dev);
        // refresh every packet
        // cmd.bytecode = vec![0x84, 0x00];
        // comm(&cmd.gen_bytes(), &mut buf, &dev);
    }
    cmd.bytecode = vec![0x84, 0x00]; 
    comm(&cmd.gen_bytes(), &mut buf, &dev);
}
