use lazy_static::lazy_static;
use raylib::{drawing::RaylibDraw, math::Rectangle, rgui::RaylibDrawGui};

use crate::{identifier::GlobalString, styles};

use super::{get_colors, Screen};

#[derive(Default)]
pub struct OptionsScreen();

impl OptionsScreen {
    pub fn new() -> Box<Self> {
        Box::new(Self::default())
    }
}

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Options");
}

impl Screen for OptionsScreen {
    fn name(&mut self) -> GlobalString {
        *NAME
    }

    fn rect(&mut self, screen: &super::ScreenDimensions) -> super::ScreenDimensions {
        super::ScreenDimensions { width: 500, height: screen.height - 80 }
    }

    fn render(&mut self, _: &mut crate::GameConfig, renderer: &mut raylib::prelude::RaylibDrawHandle, x: i32, orig_y: i32, _: i32, _: i32, _: &mut crate::world::World) {
        let colors = get_colors();

        renderer.draw_text("Style", x + 25, orig_y + 10, 20, colors.text);
        for i in 0..styles::STYLES.len() {
            let y = i as i32;
            if renderer.gui_button(Rectangle::new((x + 40 + (y % 2) * 230) as f32, (orig_y + 40 + 38 * (y / 2)) as f32, 190.0, 24.0), Some(styles::STYLES[i].0)) {
                styles::STYLES[i].1();
            }
        }
        
    }
}