#![allow(dead_code)]
mod cpu;

use crate::cpu::CPU;

/*TESTS TO PASS
 01-special.gb PASSED              
 02-interrupts.gb PENDING          
 03-op sp,hl.gb PASSED
 04-op r,imm.gb PASSED
 05-op rp.gb PASSED
 06-ld r,r.gb PASSED
 07-jr,jp,call,ret,rst.gb 07 PASSED
 08-misc instrs.gb PASSED
 09-op r,r.gb FLASHED!
 10-bit ops.gb FLASHED!
 11-op a,(hl).gb FLASHED!
* */


fn main() {
    let rom: Vec<u8> = std::fs::read("gb-test-roms/cpu_instrs/individual/02-interrupts.gb")
        .expect("Failed to load ROM");
    let mut cpu = CPU::new(rom.clone()); //Remove clone after

    cpu.run();
}
