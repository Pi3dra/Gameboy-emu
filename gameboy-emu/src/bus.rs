use crate::ppu::PPU;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

const DMA: u16 = 0xFF46;

pub struct Bus {
    memory: Memory,
    ppu: Weak<RefCell<PPU>>,
}

/*
self.bus.read(addr,true)

We can prevent ppu from self blocking itself by forcing the implementation
to use BusAccess and passing who is requesting the bus.

Currently we only need to separate CPU and PPU acces so a bool is enough,
but might change to an enum later on?

*/

pub trait BusAccess {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

/*
impl BusAccess for PPU{
    fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr,false)
    }

    fn write(&mut self, addr: u16, value: u8){
        self.bus.read(addr,false)
    }

}
*/

//TODO: Handle CPU blocking depending on PPU state
impl Bus {
    pub fn new(rom: Vec<u8>) -> Rc<RefCell<Self>> {
        let memory = Memory::new(rom);
        Rc::new(RefCell::new(Self {
            memory,
            ppu: Weak::new(),
        })) // Initially no PPU
    }

    pub fn set_ppu(&mut self, ppu: Weak<RefCell<PPU>>) {
        self.ppu = ppu;
    }

    pub fn write(&mut self, address: u16, value: u8, _cpuread: bool) {
        self.memory.write(address, value);

        if address == DMA {
            let mut new_oam = [0; 160];
            let base_address = (value as u16) << 8;
            for i in 0..160 {
                new_oam[i] = self.memory.read(base_address + i as u16);
            }
            self.memory.oam = new_oam;
            //TODO: Advance cpu clock by 160 M Cycles
            //Note this is not cycle accurate, normally the dma transfer is done by parts, in which
            //the cpu can either be executing nops or small procedures in HRAM
        }
    }

    pub fn read(&mut self, address: u16, _cpuread: bool) -> u8 {
        self.memory.read(address)
    }
}

struct Memory {
    rom0: [u8; 16_384],
    romn: [u8; 16_384],

    vram: [u8; 8_192],
    ram: [u8; 8_192],

    wram1: [u8; 4_096],
    wram2: [u8; 4_096],
    hram: [u8; 127],

    oam: [u8; 160],

    io: [u8; 128],
    interrupt: [u8; 1],
}

impl Memory {
    pub fn check_serial(&mut self) -> Option<String> {
        // Get the serial registers
        let sb = self.io[0x01]; // 0xFF01: serial data (byte to send)
        let sc = self.io[0x02]; // 0xFF02: serial control (start transfer if 0x81)

        if sc == 0x81 {
            // Blargg just requested a serial transfer
            self.io[0x02] = 0; // emulate completion: clear "transfer in progress"
            return Some((sb as char).to_string()); // return the output character
        }

        Option::None // nothing to print this step
    }

    pub fn new(rom: Vec<u8>) -> Self {
        // Split ROM into 0x0000–0x3FFF (bank 0) and 0x4000–0x7FFF (bank 1)
        let mut rom0 = [0u8; 0x4000];
        let mut romn = [0u8; 0x4000];

        for (i, byte) in rom.iter().enumerate() {
            if i < 0x4000 {
                rom0[i] = *byte;
            } else if i < 0x8000 {
                romn[i - 0x4000] = *byte;
            } else {
                break; // ignore any extra bytes (Game Boy ROM limit per bank)
            }
        }

        Self {
            rom0,
            romn,
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            wram1: [0; 0x1000],
            wram2: [0; 0x1000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            interrupt: [0; 1],
        }
    }

    fn handle_blarg_output(&mut self, address: u16, value: u8) {
        if address == 0xFF02 && value == 0x81 {
            let c = self.io[0x01] as char; // Read the byte to send
            print!("{}", c); // Print immediately
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            self.io[0x02] = 0; // Clear "transfer in progress" flag
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        // Handle serial transfer for Blargg tests
        self.handle_blarg_output(address, value);

        let (region, address, _, writable) = self.map(address);

        if writable {
            region[address] = value;
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        if address == 0xFF44 {
            return 0x90; //for some gameboy-doctor tests
        }
        let (region, address, readable, _) = self.map(address);

        //println!("R: addr: 0x{:04X}, value: 0x{:02X}", address , region[address]);
        if readable {
            return region[address];
        }

        0xFF
    }

    fn map(&mut self, address: u16) -> (&mut [u8], usize, bool, bool) {
        match address {
            0x0000..=0x3FFF => (&mut self.rom0, address as usize, true, false),
            0x4000..=0x7FFF => (&mut self.romn, (address - 0x4000) as usize, true, false),

            0x8000..=0x9FFF => (&mut self.vram, (address - 0x8000) as usize, true, true),
            0xA000..=0xBFFF => (&mut self.ram, (address - 0xA000) as usize, true, true),

            0xC000..=0xCFFF => (&mut self.wram1, (address - 0xC000) as usize, true, true),
            0xD000..=0xDFFF => (&mut self.wram2, (address - 0xD000) as usize, true, true),

            // Echo RAM mirrors C000–DDFF
            0xE000..=0xEFFF => (&mut self.wram1, (address - 0xE000) as usize, true, true),
            0xF000..=0xFDFF => (&mut self.wram2, (address - 0xF000) as usize, true, true),

            0xFE00..=0xFE9F => (&mut self.oam, (address - 0xFE00) as usize, true, true),
            0xFEA0..=0xFEFF => (&mut self.oam, 0, false, false), // not usable; return dummy

            0xFF00..=0xFF7F => (&mut self.io, (address - 0xFF00) as usize, true, true),
            0xFF80..=0xFFFE => (&mut self.hram, (address - 0xFF80) as usize, true, true),

            0xFFFF => (&mut self.interrupt, 0, true, true),
        }
    }
}
