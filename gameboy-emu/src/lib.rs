use libretro_rs::{
    RetroAudioInfo, RetroCore, RetroEnvironment, RetroGame, RetroLoadGameResult, RetroRuntime,
    RetroSystemInfo, RetroVideoInfo, libretro_core,
};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const DMG_PALETTE: [(u8, u8, u8); 4] = [
    (15, 56, 15), // darkest
    (48, 98, 48),
    (139, 172, 15),
    (155, 188, 15), // lightest
];

fn dmg_to_rgb565(level: u8) -> u16 {
    let (r, g, b) = DMG_PALETTE[level as usize];
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

/*

Implementing the libretro backend to easily implement a fronted!

https://crates.io/crates/libretro-backend
https://www.libretro.com/index.php/api/


Run like so :  retroarch --verbose -L ./target/debug/libgameboy_emu.so ../cpu_instrs.gb

*/

struct RustBoiCore {
    framebuffer: [u16; WIDTH * HEIGHT],
}

impl RetroCore for RustBoiCore {
    fn init(_env: &RetroEnvironment) -> Self {
        let mut core = Self {
            framebuffer: [0; WIDTH * HEIGHT],
        };

        // Fill background with the lightest DMG color
        let light = dmg_to_rgb565(3);
        core.framebuffer.fill(light);

        // Draw a dark square (for testing)
        let dark = dmg_to_rgb565(0);
        let square_size = 64;
        let start_x = (WIDTH - square_size) / 2;
        let start_y = (HEIGHT - square_size) / 2;

        for y in start_y..(start_y + square_size) {
            for x in start_x..(start_x + square_size) {
                core.framebuffer[y * WIDTH + x] = dark;
            }
        }
        core
    }

    fn get_system_info() -> RetroSystemInfo {
        RetroSystemInfo::new("RustBoi", "1.0").with_valid_extensions(&["gb", "gbc", ".gb", ".gbc"])
    }

    fn reset(&mut self, _env: &RetroEnvironment) {
        self.framebuffer = [0xFF; WIDTH * HEIGHT];
    }

    fn run(&mut self, _env: &RetroEnvironment, runtime: &RetroRuntime) {
        // Convert u16 framebuffer to &[u8] for upload
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.framebuffer.as_ptr() as *const u8,
                self.framebuffer.len() * 2, // 2 bytes per pixel
            )
        };

        runtime.upload_video_frame(bytes, WIDTH as u32, HEIGHT as u32, WIDTH * 2);
    }

    fn load_game(&mut self, _env: &RetroEnvironment, _game: RetroGame) -> RetroLoadGameResult {
        let video = RetroVideoInfo::new(
            59.7275, // GB framerate
            WIDTH as u32,
            HEIGHT as u32,
        )
        .with_pixel_format(libretro_rs::RetroPixelFormat::RGB565);

        let audio = RetroAudioInfo::new(44100.0);
        RetroLoadGameResult::Success { audio, video }
    }
}
libretro_core!(RustBoiCore);
