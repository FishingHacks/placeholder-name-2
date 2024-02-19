use lazy_static::lazy_static;
use raylib::{
    color::Color, drawing::RaylibDraw, math::Rectangle, rgui::RaylibDrawGui, text::measure_text,
};

use crate::{identifier::GlobalString, inventory::NUM_SLOTS_PLAYER};

use super::Screen;

#[derive(Default)]
pub struct PlayerInventoryScreen {
    selected_slot: Option<usize>,
}

const ITEM_W: u32 = 40;
const ITEM_H: u32 = 40;
const BUTTON_PAD: u32 = 7;
const BUTTON_MARGIN: u32 = 10;
const BUTTONS_PER_ROW: u32 = 9;

const BORDER_PRESSED: Color = Color::new(0x04, 0x92, 0xc7, 0xff);
const BUTTON_PRESSED: Color = Color::new(0x97, 0xe8, 0xff, 0xff);

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Inventory");
}

impl Screen for PlayerInventoryScreen {
    fn name(&mut self) -> crate::identifier::GlobalString {
        *NAME
    }
    fn rect(&mut self, _: &super::ScreenDimensions) -> super::ScreenDimensions {
        super::ScreenDimensions {
            width: ((ITEM_W + BUTTON_MARGIN * 2 + BUTTON_PAD * 2) * BUTTONS_PER_ROW) as i32,
            height: ((ITEM_H + BUTTON_MARGIN * 2 + BUTTON_PAD * 2) * NUM_SLOTS_PLAYER as u32
                / BUTTONS_PER_ROW) as i32,
        }
    }
    fn render(
        &mut self,
        cfg: &mut crate::GameConfig,
        renderer: &mut raylib::prelude::RaylibDrawHandle,
        x: i32,
        y: i32,
        _: i32,
        _: i32,
        _: &mut crate::World,
    ) {
        let mut switch_slots = (0, 0);
        for slot in 0..NUM_SLOTS_PLAYER {
            let item = cfg.inventory.get_item(slot);
            let row = slot as u32 % BUTTONS_PER_ROW;
            let col = slot as u32 / BUTTONS_PER_ROW;
            let x =
                x + (BUTTON_MARGIN + row * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_W)) as i32;
            let y =
                y + (BUTTON_MARGIN + col * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_H)) as i32;

            if renderer.gui_button(
                Rectangle::new(
                    x as f32,
                    y as f32,
                    (BUTTON_PAD * 2 + ITEM_W) as f32,
                    (BUTTON_PAD * 2 + ITEM_H) as f32,
                ),
                None,
            ) {
                if let Some(selected_slot) = self.selected_slot {
                    self.selected_slot = None;
                    if selected_slot != slot && selected_slot < NUM_SLOTS_PLAYER {
                        switch_slots = (selected_slot, slot);
                    }
                } else {
                    self.selected_slot = Some(slot);
                }
            }
            if matches!(self.selected_slot, Some(selected_slot) if selected_slot == slot) {
                renderer.draw_rectangle(
                    x,
                    y,
                    (BUTTON_PAD * 2 + ITEM_W) as i32,
                    (BUTTON_PAD * 2 + ITEM_H) as i32,
                    BUTTON_PRESSED,
                );
                renderer.draw_rectangle_lines_ex(
                    Rectangle::new(
                        x as f32,
                        y as f32,
                        (BUTTON_PAD * 2 + ITEM_W) as f32,
                        (BUTTON_PAD * 2 + ITEM_H) as f32,
                    ),
                    2,
                    BORDER_PRESSED,
                );
            }

            if let Some(item) = item {
                item.render(
                    renderer,
                    x + BUTTON_PAD as i32,
                    y + BUTTON_PAD as i32,
                    ITEM_W as i32,
                    ITEM_H as i32,
                );

                let sz = format!(
                    "x{}",
                    if item.metadata_is_stack_size() {
                        item.metadata()
                    } else {
                        1
                    }
                );
                let len = measure_text(sz.as_str(), 20);
                renderer.draw_rectangle(
                    x + BUTTON_PAD as i32 + ITEM_W as i32 - 3 - len / 2,
                    y + ITEM_H as i32 + (BUTTON_PAD * 2) as i32 - 11,
                    len + 6,
                    22,
                    Color::ORANGE,
                );
                renderer.draw_text(
                    sz.as_str(),
                    x + BUTTON_PAD as i32 + ITEM_W as i32 - len / 2,
                    y + ITEM_H as i32 + (BUTTON_PAD * 2) as i32 - 10,
                    20,
                    Color::WHITE,
                );
            }
        }

        if switch_slots.0 != switch_slots.1 {
            cfg.inventory.switch_items(switch_slots.0, switch_slots.1);
        }
    }
}
