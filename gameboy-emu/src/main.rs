#![allow(dead_code)]
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

mod cpu;
use crate::cpu::CPU;

fn main() {
    let rom_buffer = BufReader::new(File::open("../../Tetris (World) (Rev A).gb").unwrap());
    let mut instructions: Vec<u8> = Vec::new();

    for byte_or_error in rom_buffer.bytes() {
        let byte = byte_or_error.unwrap();
        instructions.push(byte);
    }
}
