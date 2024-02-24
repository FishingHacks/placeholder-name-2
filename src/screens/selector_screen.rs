use lazy_static::lazy_static;
use raylib::{
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Rectangle,
    rgui::RaylibDrawGui,
    text::measure_text,
};

use crate::{
    blocks::BLOCKS, identifier::GlobalString, world::ChunkBlockMetadata, GameConfig, RenderLayer,
};

use super::{get_colors, Screen, ScreenDimensions};

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
        w: i32,
        h: i32,
        _: &mut crate::World,
    ) {
        let w_preview = w / 4;
        let x_preview = x + w - w_preview;
        let w = if cfg.current_selected_block.is_none() {
            w
        } else {
            w / 4 * 3 - 10
        };
        let buttons_per_row = w.max(0) as u32 / (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + BLOCK_W);

        let mut block_idx: usize = 0;
        for i in unsafe { 1..BLOCKS.len() } {
            let blk = unsafe { &BLOCKS[i] };
            if blk.is_none() {
                continue;
            }
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

        if !cfg.current_selected_block.is_none() {
            let colors = get_colors();

            renderer.draw_rectangle(x + w + 4, y - 6, 2, h + 10, colors.border);

            renderer.draw_rectangle_lines_ex(
                Rectangle::new(
                    (x_preview + ((w_preview - 72) / 2)) as f32,
                    (y + 5) as f32,
                    72.0,
                    72.0,
                ),
                4,
                colors.border,
            );
            cfg.current_selected_block.render(
                renderer,
                x_preview + ((w_preview - 72) / 2 + 4),
                y + 9,
                64,
                64,
                ChunkBlockMetadata::default(),
                RenderLayer::Block,
            );

            let text = cfg.current_selected_block.name().as_str();
            let text_size = measure_text(text, 20);
            let text_x = x_preview + (w_preview - text_size - 8).max(0) / 2 + 4;
            renderer.draw_text_rec(
                renderer.get_font_default(),
                &text,
                Rectangle::new(
                    text_x as f32,
                    (y + 90) as f32,
                    (w_preview - 8) as f32,
                    20.0,
                ),
                20.0,
                2.0,
                false,
                colors.text,
            );

            renderer.draw_text_rec(
                renderer.get_font_default(),
                cfg.current_selected_block.description(),
                Rectangle::new(
                    (x_preview + 4) as f32,
                    (y + 130) as f32,
                    (w_preview - 8) as f32,
                    (h - 130) as f32,
                ),
                10.0,
                2.0,
                false,
                colors.text,
            );
        }
    }
}
