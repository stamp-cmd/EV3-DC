# EV3-DC
Low-level EV3 direct command library

## Features
 - Allocate global and local memory
 - Packet generation from bytecodes
 - Direct reply basic parser
 - Utility library for Run-Length-Encoding, packets merging, bytecode builder

## Bytecode documentation
LEGO Mindstorms :tm: EV3 Firmware Developer Kit [[Link]](https://www.lego.com/cdn/cs/set/assets/blt77bd61c3ac436ea3/LEGO_MINDSTORMS_EV3_Firmware_Developer_Kit.pdf)

## Example
This example turn on motor on port A & B with 50% power clockwise
```rust
use ev3_dc::{ Command, encode, Encoding, PORT };
use ev3_dc::utils::ChainByte;
use ev3_dc::parser::Reply;

// Create new packet. Packet can contains many OpCodes
let mut cmd = Command::new();
// Chainable vector operations
let mut byte = ChainByte::new();
byte.push(0xA4) // opOutput_Power
    .add(encode(LC0(0)).unwrap()) // Layer
    .add(encode(LC0(PORT.A + PORT.B)).unwrap()) // Port
    .add(encode(LC1(50)).unwrap()) // Power
    .push(0xA6) // opOutput_Power
    .add(encode(LC0(0)).unwrap()) // Layer
    .add(encode(LC0(PORT.A + PORT.B)).unwrap()); // Port
let mut buf = [0_u8; 5 + cmd.reserved_bytes()]; // Create reply buffer (SIZE SHOULD BE ATLEAST 5 + reserved_bytes)
println!("SENT: {:?}", cmd.gen_bytes()); // Generate direct command packet
not_real_function::read(&mut buf);
let rep = Reply::parse(&buf); // Parse direct reply
println!("RECV: {:?} | SIZE: {}, ID: {}, ERROR: {}, MEMORY: {}", buf, rep.length(), rep.id(), rep.error(), rep.memory());
```

## Binary
 - `bin/image.rs` Display 178x128 [PBM](https://en.wikipedia.org/wiki/Netpbm#PBM_example) image on EV3 screen
    ```bash
    cargo run --bin image.rs example.pbm
    ```
