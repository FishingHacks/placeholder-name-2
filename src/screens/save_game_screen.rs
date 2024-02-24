use std::{ffi::CStr, thread};

use lazy_static::lazy_static;
use raylib::{drawing::RaylibDrawHandle, ffi::KeyboardKey, math::Rectangle, rgui::RaylibDrawGui};

use crate::{
    asset, cstr, identifier::GlobalString, notice_board::{self, NoticeboardEntryRenderable}, scheduler::{schedule_task, Task}, screens::EscapeScreen, serialization::save_game, ui::{gui_textbox, TextboxState}, world::World, GameConfig
};

use super::{Screen, ScreenDimensions};

#[derive(Default)]
pub struct SavegameScreen(TextboxState); // max file size + 1

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Save Game");
}

const FILE_LABEL: &CStr = cstr!("File:");
const SAVE: &CStr = cstr!("Save");
const CANCEL: &CStr = cstr!("Cancel");

impl Screen for SavegameScreen {
    fn rect(&mut self, _: &ScreenDimensions) -> ScreenDimensions {
        ScreenDimensions {
            width: 288,
            height: 120,
        }
    }

    fn name(&mut self) -> GlobalString {
        *NAME
    }

    fn render(
        &mut self,
        cfg: &mut GameConfig,
        renderer: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        _: i32,
        _: i32,
        world: &mut World,
    ) {
        renderer.gui_label(
            Rectangle::new((x + 24) as f32, (y + 24) as f32, 48.0, 24.0),
            Some(FILE_LABEL),
        );
        if gui_textbox(
            renderer,
            Rectangle::new((x + 72) as f32, (y + 24) as f32, 192.0, 24.0),
            &mut self.0,
            Some(255),
            Some("Save Name"),
        ) {
            if renderer.is_key_pressed(KeyboardKey::KEY_ENTER) && self.0.active {
                self.save(world, cfg)
            } else {
                self.0.active = !self.0.active;
            }
        }
        if renderer.gui_button(
            Rectangle::new((x + 24) as f32, (y + 72) as f32, 96.0, 24.0),
            Some(SAVE),
        ) {
            self.save(world, cfg);
        }
        if renderer.gui_button(
            Rectangle::new((x + 168) as f32, (y + 72) as f32, 96.0, 24.0),
            Some(CANCEL),
        ) {
            self.close();
        }
    }

    fn close(&self) {
        schedule_task(Task::OpenScreenCentered(Box::new(EscapeScreen)));
    }
}

impl SavegameScreen {
    fn save(&mut self, world: &World, cfg: &GameConfig) {
        if self.0.str.len() < 1 {
            return;
        }
        println!("Save uwu: {}", self.0.str);
        self.0.str.push_str(".pn2s");
        let path = asset!("worlds", self.0.str.clone());
        notice_board::add_entry(NoticeboardEntryRenderable::StringRef("Saving Game..."), 5);
        let world = (*world).clone();
        let cfg = (*cfg).clone();

        thread::spawn(move || {
            let result = match save_game(&world, &cfg, path) {
                Err(e) => format!("Couldn't save game: {:?}", e),
                Ok(bytes) => format!("Game Saved ({bytes} bytes)"),
            };
            notice_board::add_entry(NoticeboardEntryRenderable::String(result), 5);
        });
        self.close();
    }
}
