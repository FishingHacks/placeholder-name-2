use lazy_static::lazy_static;
use raylib::{drawing::RaylibDrawHandle, math::Rectangle, rgui::RaylibDrawGui};

use crate::{blocks::BLOCKS, identifier::GlobalString, world::ChunkBlockMetadata, GameConfig, RenderLayer};

use super::{Screen, ScreenDimensions};

pub struct SelectorScreen;

const BLOCK_W: u32 = 40;
const BLOCK_H: u32 = 40;
const BUTTON_PAD: u32 = 7;
const BUTTON_MARGIN: u32 = 10;

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Building");
}

impl Screen for SelectorScreen {
    fn rect(&mut self, screen: &ScreenDimensions) -> ScreenDimensions {
        let mut dim = screen.clone();
        dim.width -= 130; // 60px padding on each side + 10 padding
        dim.height -= 155; // 60px padding on each side + 35 padding
        dim
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
        _: &mut crate::World,
    ) {
        let buttons_per_row = BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + BLOCK_W;

        let mut block_idx: usize = 0;
        for i in unsafe { 0..BLOCKS.len() } {
            let blk = unsafe { &BLOCKS[i] };
            // if !blk.is_building() {
            //     continue;
            // }
            let row = block_idx as u32 % buttons_per_row;
            let col = block_idx as u32 / buttons_per_row;
            let x =
                x + (BUTTON_MARGIN + row * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + BLOCK_W)) as i32;
            let y =
                y + (BUTTON_MARGIN + col * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + BLOCK_H)) as i32;

            if renderer.gui_button(
                Rectangle::new(
                    x as f32,
                    y as f32,
                    (BUTTON_PAD * 2 + BLOCK_W) as f32,
                    (BUTTON_PAD * 2 + BLOCK_H) as f32,
                ),
                None,
            ) {
                cfg.current_selected_block = blk;
            }
            blk.render(
                renderer,
                x + BUTTON_PAD as i32,
                y + BUTTON_PAD as i32,
                BLOCK_W as i32,
                BLOCK_H as i32,
                ChunkBlockMetadata::default(),
                RenderLayer::default_preview(),
            );
            block_idx += 1;
        }
    }
}
