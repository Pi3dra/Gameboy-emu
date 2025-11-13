//LCD controlppu
const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41; //Scrolling and misc
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
//Palletes
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
//Window position
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;
//For requesting interrupts
const IF: u16 = 0xFF0F;
const INT_VBLANK: u8 = 0;
const INT_STAT: u8 = 1;

use crate::bus::Bus;
use crate::bus::BusAccess;
use std::cell::RefCell;
use std::rc::Rc;

/*
TECHCNICAL INFO:

Screen
160x144 Pixels
4 shades of gray
8x8 pixel tile-based, 20x18

40 spirites(10 per line)

8KB VRAM

Backgroun Tile Data holds 256 tile-based

The background has 32x32 tiles -> 256, 256 pixels

The viewport is 20 by 18 tiles

OAM Entry
- Position X
- Position Y
- Tile Number
- Priority
- Flip X
- Flip Y
- Palette (OBP 0 or OBP1)


If CPU wants to write to VRAM or OAM, it should pass through the PPU,

If PPU blocks the bus then CPU won't write or will readd FFFF

The idea is to do:

clocks:    20              43             51   <- T Cycles, multiply by 2 for M cycles
        OAM Search -> Pixel transfer -> H-Blank

During Pixel transfer CPU can't acces VRAM

During OAM Search or Pixel transfer CPU can't acces OAM RAM

Then for pixel transfer there's the whole FIFO pipeline:

Re watch this: https://www.youtube.com/watch?v=HyzD8pNlpwI&t=1734s
*/

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

enum TileIndexing {
    Unsigned,
    Signed,
}

struct LcdcRegister {
    ppu_enabled: bool,
    window_tilemap: bool, // 0->0x9800-0x9BFF 1->0x9C00-0x9FFF
    window_enabled: bool,
    bg_window_tiles: bool, // 0->0x8800-0x97FF 1->0x8000-0x8FFF
    bg_tilemap: bool,      // 0->0x9800-0x9BFF 1->0x9C00-0x9FFF
    obj_size: bool,
    obj_enable: bool,
    priority: bool,
}

impl LcdcRegister {
    fn new(register: u8) -> Self {
        Self {
            ppu_enabled: register & 0x80 != 0,     // Bit 7
            window_tilemap: register & 0x40 != 0,  // Bit 6
            window_enabled: register & 0x20 != 0,  // Bit 5
            bg_window_tiles: register & 0x10 != 0, // Bit 4
            bg_tilemap: register & 0x08 != 0,      // Bit 3
            obj_size: register & 0x04 != 0,        // 0 -> 8 , 1 -> 16
            obj_enable: register & 0x02 != 0,      // Bit 1
            priority: register & 0x01 != 0,        // Bit 0
        }
    }
}

struct StatRegister {
    lyc_select: bool,
    mode2: bool,
    mode1: bool,
    mode0: bool,
    lyc_compare: bool,
    ppu_state: u8,
}

impl StatRegister {
    fn new(register: u8) -> Self {
        Self {
            //These Bits allows the CPU, to tell the PPU, when to enable a STAT interrupt!
            lyc_select: register & 0x40 != 0,
            mode2: register & 0x20 != 0, // -> Interrupt on OAM Search
            mode1: register & 0x10 != 0, // -> Interrupt on VBlank
            mode0: register & 0x08 != 0, // -> Interrupt on HBlank
            // Other flags
            lyc_compare: register & 0x04 != 0, //  LY==LYC
            ppu_state: register & 0x03,        // 0: HBlank  1:VBlank 2:OAM 3:Drawing
        }
    }
}

#[derive(Copy, Clone)]
struct Obj {
    x: u8,
    y: u8,
    tile_index: u8,
    priority: u8,
    flipx: bool,
    flipy: bool,
    palette: u8,
}

impl Obj {
    fn default() -> Obj {
        Obj {
            x: 0,
            y: 0,
            tile_index: 0,
            priority: 0,
            flipx: false,
            flipy: false,
            palette: 0,
        }
    }
}

// ============ PPU ============

type TileBytes = [u8; 16];
type Tile = [[u8; 8]; 8];
type TileMapIndexed = [[u8; 32]; 32];
type TileMapTiles = [[Tile; 32]; 32];

#[derive(Clone)]
enum State {
    Idle = 4,
    OAMSearch = 2,
    PixelTransfer = 3,
    HBlank = 0,
    VBlank = 1,
}

pub struct PPU {
    bus: Rc<RefCell<Bus>>,
    background: [u8; 256 * 256],
    viewport: [u8; WIDTH * HEIGHT],

    state: State,
    fetcher: PixelFetcher,
    fifo: PixelFIFO,

    clock: u16,
    current_line: u8,
    line_objs: Option<Vec<Obj>>,
}

impl BusAccess for PPU {
    fn read(&self, addr: u16) -> u8 {
        self.bus.borrow_mut().read(addr, false)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.bus.borrow_mut().write(addr, value, false);
    }
}

use State::*;

const TILE_MAP0_ADDRESS: u16 = 0x9800; // To 0x9BFF
const TILE_MAP1_ADDRESS: u16 = 0x9C00; // To 0x9BFF
const TILE_DATA_BASE_UNSIGNED: u16 = 0x8000;
const TILE_DATA_BASE_SIGNED: u16 = 0x8800;
const OAM: u16 = 0xFE00; // TO 0xFE9F

/*

Things to do in order

- Fetch and decode OAM
- Implement DMA -> Done but cycle inacurate,
either detect DMA on CPU BusAccess implementation, and augment clock correctly
or share a shared clock between the ppu, cpu and bus in the gameboy struct, would be nice to have
a central thing to handle clock timing?

- Implement FIFO & Fetcher



*/

impl PPU {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        let background = [0xFF; 256 * 256];
        let viewport = [0xFF; WIDTH * HEIGHT];
        let state = Idle;
        let fetcher = PixelFetcher::new();
        let fifo = PixelFIFO::new();
        let clock = 0;
        let current_line = 0;
        let line_objs = None;

        Self {
            bus,
            background,
            viewport,
            state,
            fetcher,
            fifo,
            clock,
            current_line,
            line_objs,
        }
    }

    /*
    Tile info:

    Tile data is stored in VRAM from 0x8000 to 0x97FF -> 384 Tiles

    A tile takes 16 bytes, 8x8 pixels, two bits per pixel = 64*2 = 128 = 16 bytes

    Tiles can be displayed on: Backgrounds, Windows and objects


    We have three blocks of 128 tiles each:

    Block 0 -> 0x8000-0x87FF
    Block 1 -> 0x8800-0x8FFF
    Block 2 -> 0x9000-0x97FF


    Tiles are always indexed using an 8-bit integer, but the addressing method may differ:
    The “$8000 method” uses $8000 as its base pointer and uses an unsigned addressing, meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.

    The “$8800 method” uses $9000 as its base pointer and uses a signed addressing, meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently, “$8800 addressing” takes tiles 0-127 from block 2 and tiles 128-255 from block 1.

    (You can notice that block 1 is shared by both addressing methods)

    Objects always use “$8000 addressing”, but the BG and Window can use either mode, controlled by LCDC bit 4.
    * */

    fn fetch_lcdc_register(&self) -> LcdcRegister {
        LcdcRegister::new(self.read(LCDC))
    }

    fn fetch_tile_bytes_unsigned(&self, index: u8) -> TileBytes {
        let mut bytes: [u8; 16] = [0x00; 16];
        //Starts at Block 0
        let address: u16 = TILE_DATA_BASE_UNSIGNED + (index as u16 * 16);

        for i in 0..16 {
            let byte = self.read(address + i);
            bytes[i as usize] = byte;
        }
        bytes
    }

    fn fetch_tile_bytes_signed(&self, index: i8) -> TileBytes {
        let mut bytes: [u8; 16] = [0x00; 16];
        //Starts at Block 1
        let address: u16 = TILE_DATA_BASE_UNSIGNED + (index as i16 * 16) as u16;

        for i in 0..16 {
            let byte = self.read(address + i);
            bytes[i as usize] = byte;
        }
        bytes
    }

    fn build_tile_from_bytes(bytes: [u8; 16]) -> Tile {
        let mut tile = [[0u8; 8]; 8];

        for row in 0..8 {
            let high_bits = bytes[row * 2 + 1];
            let low_bits = bytes[row * 2];

            //This is "zipping" the two bits
            for column in 0..8 {
                let mask = 1 << (7 - column);
                let low = if low_bits & mask != 0 { 1 } else { 0 };
                let high = if high_bits & mask != 0 { 1 } else { 0 };

                tile[row][column] = (high << 1) | low;
            }
        }

        tile
    }

    //This return a 32x32 grid of tile indexes -> 1024 u8;
    fn get_tile_map_indexes(&self, address: u16) -> TileMapIndexed {
        let mut tile_map = [[0u8; 32]; 32];

        for row in 0..32 {
            for column in 0..32 {
                let byte_address: u16 = address + (row * 32 + column) as u16;
                let byte = self.read(byte_address);
                tile_map[row as usize][column as usize] = byte;
            }
        }
        tile_map
    }

    fn tile_map_indexes_to_tiles(&self, tilemap: TileMapIndexed, signed: bool) -> TileMapTiles {
        let mut tilemap_tiles = [[[[0u8; 8]; 8]; 32]; 32];

        for row in 0..32 {
            for col in 0..32 {
                let index = tilemap[row][col];
                let tile_bytes = if signed {
                    self.fetch_tile_bytes_signed(index as i8)
                } else {
                    self.fetch_tile_bytes_unsigned(index)
                };
                tilemap_tiles[row][col] = Self::build_tile_from_bytes(tile_bytes);
            }
        }

        tilemap_tiles
    }

    fn get_current_tilemap(&self) -> TileMapTiles {
        let lcdc = self.fetch_lcdc_register();

        let tilemap_address = if lcdc.bg_tilemap {
            TILE_MAP1_ADDRESS
        } else {
            TILE_MAP0_ADDRESS
        };

        let signed = !lcdc.bg_window_tiles; // true if using 0x8800 signed addressing

        let indexes = self.get_tile_map_indexes(tilemap_address);
        self.tile_map_indexes_to_tiles(indexes, signed)
    }

    fn fetch_object(&self, address: u16) -> Obj {
        //Each object is 4 bytes long
        let y = self.read(address) + 16;
        let x = self.read(address + 1) + 8;
        let tile_index = self.read(address + 2);
        let flags = self.read(address + 3);

        Obj {
            x: x,
            y: y,
            tile_index: tile_index,
            priority: flags & 0x80,
            flipy: flags & 0x40 != 0,
            flipx: flags & 0x20 != 0,
            palette: (flags & 0x10) >> 4,
        }
    }

    fn fetch_objects_from_oam(&self) -> [Obj; 40] {
        let mut objects = [Obj::default(); 40];

        for i in 0..40 {
            objects[i] = self.fetch_object(OAM + (i as u16) * 4);
        }

        objects
    }

    // ============ State Functions ============

    fn oamsearch(&mut self, _cycles: u8) {
        if !matches!(self.line_objs, None) {
            return; // This means oamsearch has already been done, we are just stalling to simulate
            // cycles now
        }

        let lcdc = LcdcRegister::new(self.read(LCDC));
        let sprite_size = { if lcdc.obj_size { 16 } else { 8 } };

        let oam_data = self.fetch_objects_from_oam();
        let ly = self.read(LY);
        let mut objects_to_draw: Vec<Obj> = vec![];

        for object_data in oam_data {
            let should_be_drawn = object_data.y <= ly && object_data.y + sprite_size >= ly;
            let line_is_full = objects_to_draw.len() < 10;
            if should_be_drawn && !line_is_full {
                objects_to_draw.push(object_data);
            }
            if line_is_full {
                break;
            };
        }
        self.line_objs = Some(objects_to_draw);
    }

    fn pixeltransfer(&mut self, cycles: u8) {
        //See: Mode 3 length in
        todo!();
    }

    fn hblank(&mut self, cycles: u8) {
        //this does nothing
        todo!();
    }

    fn vblank(&mut self, cycles: u8) {
        //implement this
        //Count line
        todo!();
    }

    // ============ Changing States ===========

    fn set_state(&mut self, state: State) {
        self.state = state.clone();

        let mut stat = self.read(STAT);
        stat = (stat & !0b11) | (state as u8 & 0b11); // update mode bits only
        self.write(STAT, stat);
    }

    fn change_to_state(&mut self, state: State, remaining_cycles: u8) {
        //This supposes we are always right when changing state, and
        //We are thus  changing to state with a correct timing
        self.clock = 0;
        self.set_state(state.clone());
        self.check_stat_interrupt();

        match state {
            HBlank => {
                self.increment_ly();
                self.hblank(remaining_cycles);
            }
            VBlank => {
                self.request_interrupt(INT_VBLANK);
                self.vblank(remaining_cycles);
            }
            PixelTransfer => {
                self.pixeltransfer(remaining_cycles);
            }
            OAMSearch => {
                self.clock = 0;
                self.line_objs = None;
                self.oamsearch(remaining_cycles);
            }
            Idle => todo!(),
        }
    }

    fn request_interrupt(&mut self, bit: u8) {
        let if_val = self.read(IF);
        self.write(IF, if_val | (1 << bit));
    }

    fn check_stat_interrupt(&mut self) {
        let stat = StatRegister::new(self.read(STAT));

        let interrupt_enabled = match self.state {
            HBlank => stat.mode0,
            VBlank => stat.mode1,
            OAMSearch => stat.mode2,
            _ => false,
        };

        if interrupt_enabled {
            self.request_interrupt(INT_STAT);
        }
    }

    fn increment_ly(&mut self) {
        let new_ly = self.read(LY) + 1;

        let lyc_eq_ly = self.read(LYC) == new_ly;
        if lyc_eq_ly {
            let old_stat = self.read(STAT);
            let new_stat = old_stat & !0b100 | (lyc_eq_ly as u8) << 2;
            self.write(STAT, new_stat);
        }

        if lyc_eq_ly && (self.read(STAT) & (1 << 6)) != 0 {
            self.request_interrupt(INT_STAT);
        }

        if new_ly > 153 {
            self.write(LY, 0);
        } else {
            self.write(LY, new_ly);
        }
    }

    // =========== Running the PPU ==============

    fn state_duration(&self) -> u16 {
        match self.state {
            OAMSearch => 80,
            PixelTransfer => {
                let objs = self.line_objs.as_ref().unwrap().len();
                172 + objs as u16 * 12
            }
            HBlank => {
                let objs = self.line_objs.as_ref().unwrap().len();
                204 - objs as u16 * 12
            }
            VBlank => 4560,
            _ => unreachable!(),
        }
    }

    fn step(&mut self, cycles: u8) {
        let mut overflow = None;

        // Helper to consume cycles and detect overflow
        let mut consume = |duration: u16| -> u8 {
            let total = self.clock + cycles as u16;
            if total > duration {
                let remainder = (total - duration) as u8;
                overflow = Some(remainder);
                remainder
            } else {
                0
            }
        };

        let consumed = match self.state {
            OAMSearch => {
                let remaining = consume(self.state_duration());
                self.oamsearch(cycles.saturating_sub(remaining));
                if remaining > 0 {
                    self.change_to_state(PixelTransfer, remaining);
                }
                cycles - remaining
            }

            PixelTransfer => {
                let remaining = consume(self.state_duration());
                self.pixeltransfer(cycles.saturating_sub(remaining));
                if remaining > 0 {
                    self.change_to_state(HBlank, remaining);
                }
                cycles - remaining
            }

            HBlank => {
                let remaining = consume(self.state_duration());
                self.hblank(cycles);
                if remaining > 0 {
                    let next_state = if self.read(LY) == 143 {
                        VBlank
                    } else {
                        OAMSearch
                    };
                    self.change_to_state(next_state, remaining);
                }
                cycles - remaining
            }

            VBlank => {
                let remaining = consume(self.state_duration());
                self.vblank(cycles.saturating_sub(remaining));
                if remaining > 0 {
                    self.change_to_state(OAMSearch, remaining);
                }
                cycles - remaining
            }

            _ => todo!(),
        };

        // Update clock: if no overflow, add consumed cycles; otherwise, set to overflow
        self.clock = overflow
            .map(|o| o as u16)
            .unwrap_or_else(|| self.clock + consumed as u16);
    }
}

struct PixelFIFO {
    stub: u8,
}

struct PixelFetcher {
    start_address: u16,
    current_address: u16,
}

impl PixelFIFO {
    fn new() -> Self {
        Self { stub: 5 }
    }
}

impl PixelFetcher {
    fn new() -> Self {
        Self {
            start_address: 0,
            current_address: 0,
        }
    }
}
