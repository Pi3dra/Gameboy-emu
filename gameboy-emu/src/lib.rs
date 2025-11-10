use libretro_backend::{
    AudioVideoInfo, Core, CoreInfo, GameData, JoypadButton, LoadGameResult, PixelFormat, Region,
    RuntimeHandle,
};

/*

Implementing the libretro backend to easily implement a fronted!

https://crates.io/crates/libretro-backend
https://www.libretro.com/index.php/api/

*/

#[derive(Default)]
struct RustBoi {
    loaded_rom: Option<GameData>,
    bruh: u8,
}

impl libretro_backend::Core for RustBoi {
    fn info() -> CoreInfo {
        CoreInfo::new("RustBoi", "1.0")
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {}

    fn on_reset(&mut self) {}

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        //TODO
        LoadGameResult::Failed(game_data)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.loaded_rom.take().unwrap()
    }
}
