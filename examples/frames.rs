use hidapi::{ HidApi, HidDevice };
use ev3_dc::{ VID, PID, Command, encode, Encoding::* };
use ev3_dc::utils::{ package_bytes, ChainByte };
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::env::{args};

fn send(dev: &HidDevice, data: &[u8], buf: &mut [u8]) {
    let _ = dev.write(data);
    println!("SENT> {:?}", data);
    let _ = dev.read(buf);
    println!("RECV> {:?}", buf);
}

fn pack(pac: &mut Vec<Vec<u8>>, start: u8, end: u8, y: u8, col: u8) {
    let mut r: Vec<u8> = vec![0x84, 0x02];
    r.extend(encode(LC0(col as i8)).unwrap());
    r.extend(encode(LC2(start as i16)).unwrap());
    r.extend(encode(LC2(y as i16)).unwrap());
    if end as i16 - start as i16 > 0 {
        r[1] = 0x03;
        r.extend(encode(LC2(end as i16)).unwrap());
        r.extend(encode(LC2(y as i16)).unwrap());
    }
    pac.push(r);
}

fn delta(prev: &[u8], next: &[u8]) -> Vec<Vec<u8>> {
    let mut res: Vec<Vec<u8>> = vec![];
    if prev.len() != next.len() || prev.len() != 178 * 128 {
        panic!("Image dimension does not match 178x128!");
    }
    let mut dif: Vec<u8> = vec![];
    for i in 0..178 * 128 {
        if prev[i] == next[i] { dif.push(2); }
        else { dif.push(next[i]); }
    }
    for y in 0..128 {
        let mut state: u8 = 2;
        let mut start: u8 = 0;
        let mut end: u8 = 0;
        let mut prog: bool = false;
        for x in 0..178 {
            if next[y * 178 + x] == state { continue; }
            if !prog {
                start = x as u8;
                state = next[y * 178 + x];
                prog = true;
            }else {
                end = x as u8 - 1;
                if state != 2 {
                    pack(&mut res, start, end, y as u8, state);
                }
                state = next[y * 178 + x];
                start = x as u8;
            }
        }
        end = 177;
        if state != 2 {
            pack(&mut res, start, end, y as u8, state);
        }
    }
    res
}

fn main() {
    // Connect to EV3 via USB
    let hid = HidApi::new().expect("Cannot initialize HID context");
    let dev = hid.open(VID, PID).unwrap_or_else(|_| { panic!("EV3 not found: (PID: {}, VID: {})", VID, PID); });
    
    // Parse PBM file
    let args: Vec<String> = args().collect();
    let file_prev = File::open(args[1].clone()).unwrap();
    let file_next = File::open(args[2].clone()).unwrap();
    let mut read_prev = BufReader::new(file_prev);
    let mut read_next = BufReader::new(file_next);
    let mut buffer_prev = String::new();
    let mut buffer_next = String::new();
    // No extra check, since expecting ffmpeg & pnmtopnm output
    let _ = read_prev.read_line(&mut buffer_prev);
    let _ = read_next.read_line(&mut buffer_next);
    if buffer_prev.trim() != "P1" || buffer_prev != buffer_next {
        panic!("File not ASCII PBM");
    }
    buffer_next.clear();
    buffer_prev.clear();
    let _ = read_prev.read_line(&mut buffer_prev);
    let _ = read_next.read_line(&mut buffer_next);
    if buffer_prev.trim() != "178 128" || buffer_prev != buffer_next {
        panic!("File dimension is not 178x128");
    }
    buffer_next.clear();
    buffer_prev.clear();
    let _ = read_prev.read_to_string(&mut buffer_prev);
    let _ = read_next.read_to_string(&mut buffer_next);
    let image_prev: Vec<u8> = buffer_prev.split("").map(|x| { match x { "0" => 0, "1" => 1, _ => 2 } })
        .filter(|x| { *x != 2 }).collect();
    let image_next: Vec<u8> = buffer_next.split("").map(|x| { match x { "0" => 0, "1" => 1, _ => 2 } })
        .filter(|x| {*x != 2}).collect();

    // Send commands
    let mut cmd = Command::new();
    let mut byte = ChainByte::new();
    byte.add(vec![0x84, 0x13])
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(encode(LC0(0)).unwrap())
        .add(vec![0x84, 0x12])
        .add(encode(LC0(0)).unwrap());
    cmd.bytecode = byte.bytes;
    let mut buf: [u8; 32] = [0; 32];
    send(&dev, &cmd.gen_bytes(), &mut buf);
    let diff: Vec<Vec<u8>> = delta(&image_prev, &image_next);
    for con in package_bytes(&diff) {
        cmd.bytecode = con;
        send(&dev, &cmd.gen_bytes(), &mut buf);
        cmd.bytecode = vec![0x84, 0x00];
        send(&dev, &cmd.gen_bytes(), &mut buf);
    }
    cmd.bytecode = vec![0x84, 0x00];
    send(&dev, &cmd.bytecode, &mut buf);
}
