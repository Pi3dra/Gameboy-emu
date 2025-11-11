//LCD control
const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
//Scrolling and misc
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const DMA: u16 = 0xFF46;
//Palletes
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
//Window position
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

use crate::bus::Bus;
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

enum State {
    Idle,
    OAMSearch,
    PixelTranser,
    HBlank,
    VBlank,
}

struct Obj {
    x: u8,
    y: u8,
    tile_number: u8,
    priority: u8,
    flipx: bool,
    fipy: bool,
    palette: u8,
}

pub struct PPU{
    bus: Rc<RefCell<Bus>>,
    background: [u8; 256 * 256],
    viewport: [u8; WIDTH * HEIGHT],

    state: State,
    fetcher: PixelFetcher,
    fifo: PixelFIFO,

    clock: u16,
    current_line: u8,
    line_objs: Vec<u8>,
}

use State::*;
impl PPU {

    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        let background = [0xFF; 256 * 256];
        let viewport = [0xFF; WIDTH * HEIGHT];
        let state = Idle;
        let fetcher = PixelFetcher::new();
        let fifo = PixelFIFO::new();
        let clock = 0;
        let current_line = 0;
        let line_objs = vec![];

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

    fn oamsearch() {
        panic!("TODO");
    }
    fn pixeltransfer() {
        panic!("TODO");
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
