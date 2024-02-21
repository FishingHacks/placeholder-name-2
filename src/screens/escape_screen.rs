use lazy_static::lazy_static;
use raylib::{drawing::RaylibDrawHandle, math::Rectangle, rgui::RaylibDrawGui};

use crate::{cstr, identifier::GlobalString, scheduler::{schedule_task, Task}, GameConfig};

use super::{Screen, ScreenDimensions};

pub struct EscapeScreen;

const SCREEN_DIMENSIONS: ScreenDimensions = ScreenDimensions { width: 180, height: 20 /* top + bottom padding (10 px each) */ + 24 /* first button */ + 38 * 2 /* other buttons */ };

pub const EXIT_GAME: &std::ffi::CStr = cstr!("Quit Game");
const CLOSE_WORLD: &std::ffi::CStr = cstr!("Back to the Main Menu");
const RESUME: &std::ffi::CStr = cstr!("Resume");

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Options");
}

impl Screen for EscapeScreen {
    fn rect(&mut self, _: &ScreenDimensions) -> ScreenDimensions {
        SCREEN_DIMENSIONS
    }

    fn render(&mut self, _: &mut GameConfig, renderer: &mut RaylibDrawHandle, x: i32, y: i32, _: i32, _: i32, _: &mut crate::World) {
        if renderer.gui_button(Rectangle::new((x + 10) as f32, (y + 10) as f32, 160.0, 24.0), Some(RESUME)) {
            self.close();
        }
        if renderer.gui_button(Rectangle::new((x + 10) as f32, (y + 10 + 38 * 1) as f32, 160.0, 24.0), Some(CLOSE_WORLD)) {
            schedule_task(Task::CloseWorld);
        }
        if renderer.gui_button(Rectangle::new((x + 10) as f32, (y + 10 + 38 * 2) as f32, 160.0, 24.0), Some(EXIT_GAME)) {
            schedule_task(Task::ExitGame);
        }
    }

    fn name(&mut self) -> GlobalString {
        *NAME
    }
}