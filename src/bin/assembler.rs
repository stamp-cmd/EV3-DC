use std::fs;
use std::io::Read;

fn main() {
    let file = fs::File::open("./example.dcsm").expect("Cannot open file");
    println!("Hello World");
}
