use lazy_static::lazy_static;
use raylib::{
    color::Color, drawing::{RaylibDraw, RaylibDrawHandle}, ffi::GuiControl, math::Rectangle, rgui::RaylibDrawGui, text::{measure_text, measure_text_ex}
};

use crate::{identifier::GlobalString, inventory::NUM_SLOTS_PLAYER, items::Item};

use super::{get_colors, Screen};

#[derive(Default)]
pub struct PlayerInventoryScreen {
    selected_slot: Option<usize>,
}

const ITEM_W: u32 = 40;
const ITEM_H: u32 = 40;
const BUTTON_PAD: u32 = 7;
const BUTTON_MARGIN: u32 = 10;
const BUTTONS_PER_ROW: u32 = 9;

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Inventory");
}

pub fn tooltip(item: &Box<dyn Item>, renderer: &mut RaylibDrawHandle) {
    let colors = get_colors();

    let text_size = measure_text_ex(renderer.get_font_default(), item.description(), 10.0, 1.0);
    let name_width = measure_text(item.name().as_str(), 20);
    let mut width = name_width.max(text_size.x as i32) + 10;
    let mut height = 30 + text_size.y as i32;
    if width > 170 {
        height += 10 * width / 170;
        width = 170;
    }

    let mouse_pos = renderer.get_mouse_position();
    let x =
        mouse_pos.x as i32 + 5 + (renderer.get_screen_width() - (width + mouse_pos.x as i32 + 5)).min(0);
    let y =
        mouse_pos.y as i32 + 5 + (renderer.get_screen_height() - (height + mouse_pos.y as i32 + 5)).min(0);

    renderer.draw_rectangle_rounded(Rectangle::new(x as f32, y as f32, width as f32, height as f32), 0.2, 1, colors.bg);
    renderer.draw_rectangle_rounded_lines(
        Rectangle::new(x as f32, y as f32, width as f32, height as f32),
        0.2,
        1,
        2,
        colors.border,
    );
    renderer.draw_text_rec(
        renderer.get_font_default(),
        &item.name().as_str(),
        Rectangle::new((x + 5) as f32, (y + 5) as f32, (width - 10) as f32, 20.0),
        20.0,
        2.0,
        false,
        colors.text,
    );
    renderer.draw_text_rec(
        renderer.get_font_default(),
        &item.description(),
        Rectangle::new((x + 5) as f32, (y + 25) as f32, (width - 10) as f32, (height - 30) as f32),
        10.0,
        1.0,
        false,
        colors.text,
    );
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
        let border_pressed = Color::get_color(renderer.gui_get_style(GuiControl::DEFAULT, 3));
        let button_pressed = Color::get_color(renderer.gui_get_style(GuiControl::DEFAULT, 4));

        let mut switch_slots = (0, 0);
        let pos = renderer.get_mouse_position();
        let mut idx = NUM_SLOTS_PLAYER;

        for slot in 0..NUM_SLOTS_PLAYER {
            let item = cfg.inventory.get_item(slot);
            let row = slot as u32 % BUTTONS_PER_ROW;
            let col = slot as u32 / BUTTONS_PER_ROW;
            let x =
                x + (BUTTON_MARGIN + row * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_W)) as i32;
            let y =
                y + (BUTTON_MARGIN + col * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_H)) as i32;

            if idx >= NUM_SLOTS_PLAYER
                && Rectangle::new(
                    x as f32,
                    y as f32,
                    (ITEM_W + BUTTON_PAD * 2) as f32,
                    (ITEM_H + BUTTON_PAD * 2) as f32,
                )
                .check_collision_point_rec(pos)
            {
                idx = slot;
            }

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
                    button_pressed,
                );
                renderer.draw_rectangle_lines_ex(
                    Rectangle::new(
                        x as f32,
                        y as f32,
                        (BUTTON_PAD * 2 + ITEM_W) as f32,
                        (BUTTON_PAD * 2 + ITEM_H) as f32,
                    ),
                    2,
                    border_pressed,
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

        if idx < NUM_SLOTS_PLAYER {
            if let Some(item) = cfg.inventory.get_item(idx) {
                tooltip(item, renderer);
            }
        }
    }
}
