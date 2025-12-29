use hidapi::{ HidApi, HidDevice };
use ev3_dc::{ encode, Command, DataType::*, Encoding::*, PID, VID };
use ev3_dc::utils::{ ChainByte, read_string, device_id, port_read };
use ev3_dc::parser::Reply;
use ev3_dc::funcs::battery_percentage;

// This function show data transmission
fn comm(packet: &[u8], buf: &mut [u8], dev: &HidDevice) {
    let _ = dev.write(packet);
    println!("> SENT: {:02X?}", packet);
    let _ = dev.read(buf);
    println!("< RECV: {:02X?}", buf);
}

fn main() {
    // Create HID context and open EV3's VendorId & ProductId
    let hid = HidApi::new().expect("Unable to create HID context");
    let dev = hid.open(VID, PID).expect("EV3 Not found");
    // Create new command
    let mut cmd = Command::new();
    let mut byte = ChainByte::new();
    let mut buf = vec![0_u8; 40];
    byte.add(vec![0xD3, 0x0D])
        .add(encode(LC0(13)).unwrap()) // `encode(Encoding::*)` is used for allocating local
                                       // constant. (i.e. motor speed, led color)
        .add(cmd.allocate(DATAS(12), true).unwrap()); // `Command.allocate(DataType::*)` is used for
                                                      // allocating global variable
                                                      // (i.e. battery percentage, sensor value)
    cmd.bytecode = byte.bytes; // `Command.bytecode = ChainByte.bytes` insert compiled bytecode from
                               // `ChainByte` into `Command`'s bytecodes, Which are the instructions.
    comm(&cmd.gen_bytes(), &mut buf, &dev); // `Command.gen_bytes()` return compiled bytes to be sent
                                            // to EV3
    let mut rep = Reply::parse(&buf[..(4 + cmd.reserved_bytes())]); // `Command.reserved_bytes()`
                                                                    // return reserved space from
                                                                    // allocating global variable.
                                                                    // `Command.allocate()`
                                                                    // increment this value
    println!("Name: {}", read_string(rep.memory()).unwrap()); // `Reply::parse` parse returned
                                                              // bytes from EV3 with header and
                                                              // contents. `Reply.memory()` returns
                                                              // return memory content
    cmd.mem_free(); // Clear allocated memory. Used when reusing same `Command`
    byte = ChainByte::new();
    byte.add(vec![0x081, 0x0A])
        .add(encode(LC0(7)).unwrap())
        .add(cmd.allocate(DATAS(6), true).unwrap());
    cmd.bytecode = byte.bytes;
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    rep = Reply::parse(&buf[..(4 + cmd.reserved_bytes())]);
    println!("Firmware: {}", read_string(rep.memory()).unwrap());
    cmd.mem_free();
    cmd.bytecode = battery_percentage(&mut cmd).unwrap().0;
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    rep = Reply::parse(&buf[..(5 + cmd.reserved_bytes())]);
    println!("Battery: {}%", rep.memory()[0]);
    cmd.mem_free();
    byte = ChainByte::new();
    byte.push(0x98)
        .add(encode(LC1(32)).unwrap())
        .add(cmd.allocate(DATAN(32), true).unwrap())
        .add(cmd.allocate(DATA8, true).unwrap());
    cmd.bytecode = byte.bytes;
    comm(&cmd.gen_bytes(), &mut buf, &dev);
    rep = Reply::parse(&buf[..(5 + cmd.reserved_bytes())]);
    let map = ["1", "2", "3", "4", "A", "B", "C", "D"];
    let ports = port_read(&rep.memory()[..32], 0).unwrap();
    for i in 0..8 {
        println!("PORT {}: {}", map[i], device_id(ports[i]))
    }
}
