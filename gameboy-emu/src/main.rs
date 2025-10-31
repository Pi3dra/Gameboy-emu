#![allow(dead_code)]
mod cpu;

use crate::cpu::CPU;

fn main() {
    let rom: Vec<u8> = std::fs::read("../cpu_instrs.gb").expect("Failed to load ROM");
    let mut cpu = CPU::new(rom.clone()); //Remove clone after

    cpu.run();

}

