#![allow(dead_code)]
mod cpu;
mod ppu;
mod bus;

use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::bus::Bus;

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

struct GameBoi {
    cpu : CPU,
    ppu : Rc<RefCell<PPU>>, 
    bus : Rc<RefCell<Bus>>,

}
 

impl GameBoi {

    fn new(rom: Vec<u8>) -> Self {
        let bus = Bus::new(rom);
        let ppu = Rc::new(RefCell::new(PPU::new(bus.clone())));
        bus.borrow_mut().set_ppu(Rc::downgrade(&ppu));
        let cpu = CPU::new(bus.clone());
        Self { cpu, ppu, bus }
    }

    fn run(&mut self){
        self.cpu.run();
    }
}


fn main() {
    let rom: Vec<u8> = std::fs::read("gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb")
        .expect("Failed to load ROM");
    let mut rustboi = GameBoi::new(rom);
    rustboi.run();
}
