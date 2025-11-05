#![allow(dead_code)]
mod cpu;

use crate::cpu::CPU;

fn main() {
    let rom: Vec<u8> = std::fs::read("gb-test-roms/cpu_instrs/individual/02-interrupts.gb").expect("Failed to load ROM");
    let mut cpu = CPU::new(rom.clone()); //Remove clone after

    cpu.run();

}

