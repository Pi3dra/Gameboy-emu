#![allow(dead_code)]
mod bus;
mod cpu;
mod gameboi;
mod ppu;
use crate::gameboi::GameBoi;

//TODO: Implement loop for running the cpu and ppu,
//And a function to load various roms in the cpu,
fn main() {
    let mut rustboi = GameBoi::new();
    rustboi.load_rom_from_path("gb-test-roms/cpu_instrs/individual/01-special.gb");
    loop {
        rustboi.step();
    }
}
