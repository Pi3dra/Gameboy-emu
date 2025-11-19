use crate::bus::Bus;
use crate::cpu::CPU;
use crate::ppu::PPU;
use std::cell::RefCell;
use std::rc::Rc;

pub struct GameBoi {
    cpu: CPU,
    ppu: PPU,
    bus: Rc<RefCell<Bus>>,
}

impl GameBoi {
    pub fn new() -> Self {
        let bus = Bus::empty();
        let ppu = PPU::new(bus.clone());
        let cpu = CPU::new(bus.clone());
        Self { cpu, ppu, bus }
    }

    pub fn load_rom_from_path(&mut self, rom_path: &str) {
        self.bus.borrow_mut().load_rom(rom_path);
    }

    pub fn load_rom_from_data(&mut self, rom_data: &[u8]) {
        self.bus.borrow_mut().load_rom_data(rom_data);
    }

    pub fn step(&mut self) -> [u8; 23040] {
        while !self.ppu.is_frame_ready() {
            let cycles = self.cpu.step();
            //self.cpu.print_state();
            self.ppu.step(cycles / 2);
            //ppu.print_state();
        }
        let frame = self.ppu.yield_frame();
        println!("YIELDED FRAME");
        self.ppu.clear_buffer();
        frame
    }
}
