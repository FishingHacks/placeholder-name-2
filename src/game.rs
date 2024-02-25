use std::time::Instant;

use crate::{
    assets::update_textures,
    blocks::{empty_block, Block, BLOCK_EMPTY},
    inventory::{Inventory, NUM_SLOTS_PLAYER},
    notice_board::{self, NoticeboardEntryRenderable},
    scheduler::{get_tasks, schedule_task, Task},
    screens::{
        close_screen, CurrentScreen, EscapeScreen, PlayerInventoryScreen, ScreenDimensions,
        SelectorScreen,
    },
    serialization::{self, Deserialize, SerializationTrap, Serialize},
    world::{ChunkBlockMetadata, Direction, Vec2i, World, BLOCK_H, BLOCK_W},
    RenderFn, RENDER_STEP,
};
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    RaylibHandle,
};
use raylib::{drawing::RaylibDraw, ffi::KeyboardKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Block,
    OverlayItems,
}

impl RenderLayer {
    pub fn default_preview() -> Self {
        Self::Block
    }
}

pub const RENDER_LAYERS: [RenderLayer; 2] = [RenderLayer::Block, RenderLayer::OverlayItems];

fn make_abs(val: i32) -> u32 {
    if val >= 0 {
        val as u32
    } else {
        0
    }
}

#[derive(Clone)]
pub struct GameConfig {
    pub current_selected_block: &'static Box<dyn Block>,
    pub direction: Direction,
    pub inventory: Inventory,
    pub player: Vec2i,
    pub interaction_mode: InteractionMode,
}

#[derive(Debug, Clone)]
pub enum InteractionMode {
    None,
    Building,
    Dismantling,
}

impl Serialize for GameConfig {
    fn required_length(&self) -> usize {
        SerializationTrap::required_length()
            + self.inventory.required_length()
            + self.player.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::GameCfg.serialize(buf);
        self.player.serialize(buf);
        self.inventory.serialize(buf);
    }
}

impl Deserialize for GameConfig {
    fn try_deserialize(
        buf: &mut serialization::Buffer,
    ) -> Result<Self, serialization::SerializationError> {
        SerializationTrap::GameCfg.try_deserialize(buf)?;
        let player = Vec2i::try_deserialize(buf)?;
        let inventory = Inventory::try_deserialize(buf)?;

        Ok(Self {
            player,
            inventory,
            ..Self::default()
        })
    }
}

impl GameConfig {
    pub fn default() -> Self {
        Self {
            current_selected_block: empty_block(),
            direction: Direction::North,
            inventory: Inventory::new(NUM_SLOTS_PLAYER, true),
            player: Vec2i::ZERO,
            interaction_mode: InteractionMode::None,
        }
    }
}

pub const TPS: u32 = 20;
pub const MSPT: u128 = (1000 / TPS) as u128;

pub fn run_game(
    rl: &mut RaylibHandle,
    thread: &raylib::prelude::RaylibThread,
    mut world: World,
    mut config: GameConfig,
) {
    world.init();

    let mut last_update = Instant::now();
    let mut ticks_per_second = 20;

    let mut last_render_start = Instant::now();
    let mut last_screen_size = ScreenDimensions {
        width: 0,
        height: 0,
    };

    while !rl.window_should_close() {
        update_textures();

        let dt = Instant::now().duration_since(last_render_start).as_millis() as f64;
        if dt < 2.0 {
            continue;
        }
        last_render_start = Instant::now();

        let screen_size: ScreenDimensions = ScreenDimensions {
            width: rl.get_screen_width(),
            height: rl.get_screen_height(),
        };
        if last_screen_size.width != screen_size.width
            || last_screen_size.height != screen_size.height
        {
            last_screen_size.width = screen_size.width;
            last_screen_size.height = screen_size.height;
            CurrentScreen::move_to_center(&screen_size);
        }

        let tasks = get_tasks();

        // run updates
        let update_start = Instant::now();
        let mut had_gameupdate_scheduled = false;
        for t in tasks {
            if matches!(config.interaction_mode, InteractionMode::Building)
                && config.current_selected_block.identifier() == *BLOCK_EMPTY
            {
                config.interaction_mode = InteractionMode::None;
            }

            match t {
                Task::Custom(func) => func(),
                Task::ExitGame => return,
                Task::OpenScreenCentered(screen) => {
                    CurrentScreen::open_centered(screen, &screen_size)
                }
                Task::CloseScreen => close_screen(),
                Task::WorldUpdateBlock(func, meta) => {
                    had_gameupdate_scheduled = true;
                    func(meta, &mut world);
                }
                Task::CloseWorld => {
                    *RENDER_STEP.lock().unwrap() = RenderFn::StartMenu;
                    return;
                }
                Task::OpenWorld(..) | Task::CreateWorld | Task::__OpnWrld(..) => {
                    notice_board::add_entry(
                        NoticeboardEntryRenderable::String(
                            "WARN!! RECEIVED WORLD OPENING TASK IN RUN_GAME(..)".to_string(),
                        ),
                        20,
                    )
                }
            }
        }
        if had_gameupdate_scheduled {
            ticks_per_second = (1000
                / Instant::now()
                    .duration_since(update_start)
                    .as_millis()
                    .max(1))
            .min(20);
        }

        let game_focused = !CurrentScreen::is_screen_open();

        if game_focused {
            let mut direction: Vector2 = Vector2::default();
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_W) {
                direction.y -= (dt * 0.8) as f32;
            }
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_S) {
                direction.y += (dt * 0.8) as f32;
            }
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_A) {
                direction.x -= (dt * 0.8) as f32;
            }
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_D) {
                direction.x += (dt * 0.8) as f32;
            }
            if direction.x != 0.0 && direction.y != 0.0 {
                direction.x *= 0.7;
                direction.y *= 0.7;
            }
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_LEFT_SHIFT) {
                direction.x *= 1.5;
                direction.y *= 1.5;
            }
            config.player.x += direction.x as i32;
            config.player.y += direction.y as i32;
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_TAB) {
                CurrentScreen::open_centered(
                    Box::new(PlayerInventoryScreen::default()),
                    &screen_size,
                );
            }
            if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_B) {
                CurrentScreen::open_centered(Box::new(SelectorScreen), &screen_size);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_G) {
                config.interaction_mode = InteractionMode::Dismantling;
            }
            if rl.get_mouse_wheel_move() != 0.0 {
                let right = rl.get_mouse_wheel_move() > 0.0;
                config.direction = config.direction.next(right);
            }
        }
        if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_ESCAPE) {
            if !game_focused {
                CurrentScreen::close();
            } else if !config.current_selected_block.is_none()
                || matches!(
                    config.interaction_mode,
                    InteractionMode::Building | InteractionMode::Dismantling
                )
            {
                config.current_selected_block = empty_block();
                config.interaction_mode = InteractionMode::None;
            } else {
                CurrentScreen::open_centered(Box::new(EscapeScreen), &screen_size);
            }
        }

        let cursor_pos = rl.get_mouse_position();
        let mut cursor_x = (cursor_pos.x as i32 + config.player.x) / BLOCK_W as i32;
        let mut cursor_y = (cursor_pos.y as i32 + config.player.y) / BLOCK_H as i32;

        if (cursor_pos.x as i32 + config.player.x) < 0 {
            cursor_x -= 1;
        }
        if (cursor_pos.y as i32 + config.player.y) < 0 {
            cursor_y -= 1;
        }

        let mut off_x = config.player.x % BLOCK_W as i32;
        let mut off_y = config.player.y % BLOCK_H as i32;
        if off_x < 0 {
            off_x += BLOCK_W as i32;
        }
        if off_y < 0 {
            off_y += BLOCK_W as i32;
        }

        let overlay_x =
            (make_abs(cursor_pos.x as i32 + off_x).wrapping_div(BLOCK_W) * BLOCK_W) as i32 - off_x;
        let overlay_y =
            (make_abs(cursor_pos.y as i32 + off_y).wrapping_div(BLOCK_H) * BLOCK_H) as i32 - off_y;

        if rl.is_mouse_button_down(raylib::ffi::MouseButton::MOUSE_LEFT_BUTTON) && game_focused {
            match config.interaction_mode {
                InteractionMode::Building => {
                    world.set_block_at(
                        cursor_x,
                        cursor_y,
                        config.current_selected_block.clone_block(),
                        config.direction,
                    );
                }
                InteractionMode::Dismantling => {
                    world.destroy_block_at(cursor_x, cursor_y, &mut config.inventory);
                }
                InteractionMode::None => {}
            }
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        // schedule updates
        if Instant::now().duration_since(last_update).as_millis() >= MSPT {
            world.update();
            schedule_task(Task::WorldUpdateBlock(
                &|_, _| {},
                ChunkBlockMetadata::default(),
            ));
            notice_board::update_entries();
            last_update = Instant::now();
        }

        if screen_size.width >= 0 && screen_size.height >= 0 {
            for l in RENDER_LAYERS {
                world.render(
                    &mut d,
                    config.player.x,
                    config.player.y,
                    screen_size.width as u32,
                    screen_size.height as u32,
                    l,
                );
            }
        }

        if game_focused {
            if matches!(
                config.interaction_mode,
                InteractionMode::Building | InteractionMode::Dismantling
            ) {
                let col = match config.interaction_mode {
                    InteractionMode::Building => Color::GRAY.fade(0.5),
                    InteractionMode::Dismantling => Color::RED.fade(0.5),
                    InteractionMode::None => Color::BLANK,
                };
                d.draw_rectangle(overlay_x, overlay_y, BLOCK_W as i32, BLOCK_H as i32, col);
            }

            if let Some((block, data)) = world.get_block_at_mut(cursor_x, cursor_y) {
                if block.supports_interaction() {
                    d.draw_text(
                        block
                            .custom_interact_message()
                            .unwrap_or_else(|| format!("Press F to interact with {}", block.name()))
                            .as_str(),
                        overlay_x,
                        overlay_y + BLOCK_H as i32 + 5,
                        20,
                        Color::BLACK,
                    );
                    if d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_F) {
                        block.interact(data, &mut config);
                    }
                }
            }
        }

        if matches!(config.interaction_mode, InteractionMode::Building) {
            config.current_selected_block.render(
                &mut d,
                20,
                screen_size.height - 68,
                48,
                48,
                ChunkBlockMetadata::from(config.direction),
                RenderLayer::default_preview(),
            );
            d.draw_rectangle_lines_ex(
                Rectangle::new(17.0, (screen_size.height - 68 - 3) as f32, 54.0, 54.0),
                3,
                Color::BLACK,
            );
        }

        d.draw_fps(5, 45);
        d.draw_text(
            format!("TPS: {ticks_per_second}").as_str(),
            5,
            5,
            20,
            Color::DARKGREEN,
        );
        d.draw_text(
            format!(
                "X: {} Y: {} | Facing: {cursor_x} {cursor_y}",
                config.player.x, config.player.y
            )
            .as_str(),
            5,
            25,
            20,
            Color::DARKGREEN,
        );

        CurrentScreen::render(&mut config, &mut d, &screen_size, &mut world);

        notice_board::render_entries(&mut d, screen_size.height / 2, screen_size.height);
    }
}
