use std::fs::read_dir;

use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibScissorModeExt},
    math::Rectangle,
    rgui::RaylibDrawGui,
};

use crate::{asset, identifier::GlobalString};

use super::{Screen, ScreenDimensions};

pub struct WorldScreen(Vec<String>, u32, u32);

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Worlds");
}

impl WorldScreen {
    pub fn new() -> std::io::Result<Box<Self>> {
        read_dir(asset!("worlds")).map(|dirs| {
            let mut entries: Vec<String> = Vec::with_capacity(30);

            for p in dirs {
                let p = if let Ok(p) = p {
                    p
                } else {
                    continue;
                };
                if let Some(str) = p.file_name().to_str() {
                    entries.push(str.to_string())
                } else {
                    continue;
                }
            }

            Box::new(Self(entries, 0, 10))
        })
    }
}

impl Screen for WorldScreen {
    fn name(&mut self) -> GlobalString {
        *NAME
    }

    fn rect(&mut self, screen: &ScreenDimensions) -> ScreenDimensions {
        ScreenDimensions {
            width: (screen.width - 10) / 4 * 3,
            height: (screen.height - 35) / 4 * 3,
        }
    }

    fn render(
        &mut self,
        cfg: &mut crate::GameConfig,
        renderer: &mut raylib::prelude::RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        world: &mut crate::world::World,
    ) {
        self.1 = renderer.gui_scroll_bar(
            Rectangle::new((x + w - 10) as f32, (y + 10) as f32, 10.0, (h - 20) as f32),
            self.1 as i32,
            0,
            self.2 as i32,
        ) as u32;

        let mut renderer = renderer.begin_scissor_mode(x, y, w, h);

        renderer.draw_rectangle(x, y, 20, 20, Color::BLACK);
    }
}
