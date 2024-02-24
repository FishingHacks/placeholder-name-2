use raylib::{
    color::Color, drawing::RaylibDraw, ffi::GuiControl, math::Rectangle, rgui::RaylibDrawGui, text::measure_text
};

use crate::{identifier::GlobalString, inventory::NUM_SLOTS_PLAYER, world::World};

use super::{player_inventory_screen::tooltip, CurrentScreen, Screen};

#[derive(Default)]
pub struct ContainerInventoryScreen {
    selected_slot: Option<(usize, bool)>,
    pos_x: i32,
    pos_y: i32,
    num_slots: u32,
    name: GlobalString,
}

const ITEM_W: u32 = 40;
const ITEM_H: u32 = 40;
const BUTTON_PAD: u32 = 7;
const BUTTON_MARGIN: u32 = 10;
const BUTTONS_PER_ROW: u32 = 5;

impl ContainerInventoryScreen {
    pub fn new(pos_x: i32, pos_y: i32, num_slots: u32, name: GlobalString) -> Self {
        Self {
            num_slots,
            pos_x,
            pos_y,
            name,
            selected_slot: None,
        }
    }
}

macro_rules! some_or_close_screen {
    ($val: expr) => {
        match $val {
            None => {
                CurrentScreen::close();
                return;
            }
            Some(v) => v,
        }
    };
}

impl Screen for ContainerInventoryScreen {
    fn name(&mut self) -> GlobalString {
        self.name
    }
    fn rect(&mut self, _: &super::ScreenDimensions) -> super::ScreenDimensions {
        super::ScreenDimensions {
            width: ((ITEM_W + BUTTON_MARGIN * 2 + BUTTON_PAD * 2) * (BUTTONS_PER_ROW * 2 + 1))
                as i32,
            height: ((ITEM_H + BUTTON_MARGIN * 2 + BUTTON_PAD * 2) * self.num_slots as u32)
                .div_ceil(BUTTONS_PER_ROW) as i32,
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
        world: &mut World,
    ) {
        let border_pressed = Color::get_color(renderer.gui_get_style(GuiControl::DEFAULT, 3));
        let button_pressed = Color::get_color(renderer.gui_get_style(GuiControl::DEFAULT, 4));

        let mut switch_slots = ((0, false), (0, false));
        let inventory = some_or_close_screen!(world
            .get_block_at_mut(self.pos_x, self.pos_y)
            .and_then(|block| block.0.get_inventory_capability()));

        let mut idx: Option<(usize, bool)> = None;
        let pos = renderer.get_mouse_position();

        for slot in 0..inventory.size() {
            let item = inventory.get_item(slot);
            let row = slot as u32 % BUTTONS_PER_ROW;
            let col = slot as u32 / BUTTONS_PER_ROW;
            let x =
                x + (BUTTON_MARGIN + row * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_W)) as i32;
            let y =
                y + (BUTTON_MARGIN + col * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_H)) as i32;

            if idx.is_none()
                && Rectangle::new(
                    x as f32,
                    y as f32,
                    (ITEM_W + BUTTON_PAD * 2) as f32,
                    (ITEM_H + BUTTON_PAD * 2) as f32,
                )
                .check_collision_point_rec(pos)
            {
                idx = Some((slot, false));
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

                    if selected_slot.1 || selected_slot.0 != slot {
                        if (selected_slot.0 < inventory.size() || selected_slot.1)
                            || (selected_slot.0 < NUM_SLOTS_PLAYER || !selected_slot.1)
                        {
                            switch_slots = (selected_slot, (slot, false));
                        }
                    }

                    if (selected_slot.0 != slot || selected_slot.1)
                        && (!selected_slot.1 && selected_slot.0 < inventory.size())
                    {
                    }
                } else {
                    self.selected_slot = Some((slot, false));
                }
            }
            if matches!(self.selected_slot, Some(selected_slot) if selected_slot.0 == slot && !selected_slot.1)
            {
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

        for slot in 0..NUM_SLOTS_PLAYER {
            let item = cfg.inventory.get_item(slot);
            let row = slot as u32 % BUTTONS_PER_ROW + BUTTONS_PER_ROW + 1;
            let col = slot as u32 / BUTTONS_PER_ROW;
            let x =
                x + (BUTTON_MARGIN + row * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_W)) as i32;
            let y =
                y + (BUTTON_MARGIN + col * (BUTTON_MARGIN * 2 + BUTTON_PAD * 2 + ITEM_H)) as i32;

            if idx.is_none()
                && Rectangle::new(
                    x as f32,
                    y as f32,
                    (ITEM_W + BUTTON_PAD * 2) as f32,
                    (ITEM_H + BUTTON_PAD * 2) as f32,
                )
                .check_collision_point_rec(pos)
            {
                idx = Some((slot, true));
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

                    if !selected_slot.1 || selected_slot.0 != slot {
                        if (selected_slot.0 < inventory.size() || selected_slot.1)
                            || (selected_slot.0 < NUM_SLOTS_PLAYER || !selected_slot.1)
                        {
                            switch_slots = (selected_slot, (slot, true));
                        }
                    }

                    if (selected_slot.0 != slot || selected_slot.1)
                        && (!selected_slot.1 && selected_slot.0 < inventory.size())
                    {
                    }
                } else {
                    self.selected_slot = Some((slot, true));
                }
            }
            if matches!(self.selected_slot, Some(selected_slot) if selected_slot.0 == slot && selected_slot.1)
            {
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

        if switch_slots.0 .0 != switch_slots.1 .0 || switch_slots.0 .1 != switch_slots.1 .1 {
            if switch_slots.0 .1 && switch_slots.1 .1 {
                cfg.inventory
                    .switch_items(switch_slots.0 .0, switch_slots.1 .0);
            } else if !switch_slots.0 .1 && !switch_slots.1 .1 {
                inventory.switch_items(switch_slots.0 .0, switch_slots.1 .0);
            } else {
                let item_a = if switch_slots.0 .1 {
                    cfg.inventory.take_item(switch_slots.0 .0)
                } else {
                    inventory.take_item(switch_slots.0 .0)
                };
                let item_b = if switch_slots.1 .1 {
                    cfg.inventory.take_item(switch_slots.1 .0)
                } else {
                    inventory.take_item(switch_slots.1 .0)
                };
                if let Some(item_b) = item_b {
                    if switch_slots.0 .1 {
                        cfg.inventory.add_item(item_b, switch_slots.0 .0);
                    } else {
                        inventory.add_item(item_b, switch_slots.0 .0);
                    };
                }
                if let Some(item_a) = item_a {
                    if switch_slots.1 .1 {
                        cfg.inventory.add_item(item_a, switch_slots.1 .0);
                    } else {
                        inventory.add_item(item_a, switch_slots.1 .0);
                    };
                }
            }
        }

        if let Some((slot, player_inv)) = idx {
            let item = if player_inv {
                cfg.inventory.get_item(slot)
            } else {
                inventory.get_item(slot)
            };
            if let Some(item) = item {
                tooltip(item, renderer);
            }
        }
    }
}
