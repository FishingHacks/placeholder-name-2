use std::ffi::CStr;

use lazy_static::lazy_static;
use raylib::{math::Rectangle, rgui::RaylibDrawGui};

use crate::{
    cstr, identifier::GlobalString, notice_board::{self, NoticeboardEntryRenderable}, scheduler::{schedule_task, Task}
};

use super::{escape_screen::EXIT_GAME, Screen, WorldScreen};

#[derive(Default)]
pub struct MainScreen;

const OPEN_WORLD: &CStr = cstr!("Open World");
const CREDITS: &CStr = cstr!("Credits");
const OPTIONS: &CStr = cstr!("Options");

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Placeholder Name 2");
}

impl Screen for MainScreen {
    fn rect(&mut self, _: &super::ScreenDimensions) -> super::ScreenDimensions {
        super::ScreenDimensions {
            width: 732,
            height: 456,
        }
    }

    fn render(
        &mut self,
        _: &mut crate::GameConfig,
        renderer: &mut raylib::prelude::RaylibDrawHandle,
        x: i32,
        y: i32,
        _: i32,
        _: i32,
        _: &mut crate::world::World,
    ) {
        if renderer.gui_button(
            Rectangle::new((x + 202) as f32, (y + 104) as f32, 328.0, 48.0),
            Some(OPEN_WORLD),
        ) {
            match WorldScreen::new() {
                Ok(sc) => schedule_task(Task::OpenScreenCentered(sc)),
                Err(e) => notice_board::add_entry(NoticeboardEntryRenderable::String(format!("Could not read worlds dir: {e:?}")), 5),
            }
        }

        renderer.gui_button(
            Rectangle::new((x + 202) as f32, (y + 200) as f32, 328.0, 48.0),
            Some(CREDITS),
        );
        if renderer.gui_button(
            Rectangle::new((x + 202) as f32, (y + 296) as f32, 140.0, 48.0),
            Some(EXIT_GAME),
        ) {
            schedule_task(Task::ExitGame);
        }
        renderer.gui_button(
            Rectangle::new((x + 390) as f32, (y + 296) as f32, 140.0, 48.0),
            Some(OPTIONS),
        );
    }

    fn name(&mut self) -> crate::identifier::GlobalString {
        *NAME
    }
}
