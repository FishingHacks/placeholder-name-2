use std::ffi::CStr;

use lazy_static::lazy_static;
use raylib::{color::Color, drawing::RaylibDraw, math::Rectangle, rgui::RaylibDrawGui, text::measure_text};

use crate::{cstr, identifier::GlobalString};

use super::{Screen, ScreenDimensions};

lazy_static! {
    static ref NAME: GlobalString = GlobalString::from("Dialog Box");
}

const OK: &CStr = cstr!("Ok");

pub struct DialogBox(Option<GlobalString>, String, bool);

impl DialogBox {
    #[allow(dead_code)]
    pub fn new(titel: Option<GlobalString>, content: String) -> Box<Self> {
        Box::new(Self(titel, content, true))
    }
    
    #[allow(dead_code)]
    pub fn new_uncloseable(titel: Option<GlobalString>, content: String) -> Box<Self> {
        Box::new(Self(titel, content, false))
    }
}

impl Screen for DialogBox {
    fn name(&mut self) -> GlobalString {
        self.0.unwrap_or(*NAME)
    }

    fn rect(&mut self, _: &ScreenDimensions) -> ScreenDimensions {
        ScreenDimensions {
            width: measure_text(&self.1, 10) + 40,
            height: (self.1.chars().filter(|&char| char == '\n').count() * 10 + 10 + if self.2 { 44 } else { 0 }) as i32,
        }
    }

    fn render(&mut self, _: &mut crate::GameConfig, renderer: &mut raylib::prelude::RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32, _: &mut crate::world::World) {
        if self.2 {
            if renderer.gui_button(Rectangle::new(
                (x + (w - 48) / 2) as f32,
                (y + h - 34) as f32,
                48.0,
                24.0,
            ), Some(OK)) {
                self.close();
            }
        }
        renderer.draw_text(self.1.as_str(), x + 20, y, 10, Color::BLACK);
    }
}