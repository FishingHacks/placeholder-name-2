use std::time::{Duration, Instant};

use crate::{
    assets::update_textures,
    blocks::{empty_block, Block, BLOCK_EMPTY},
    inventory::{Inventory, NUM_SLOTS_PLAYER},
    notice_board::{self, NoticeboardEntryRenderable},
    scheduler::{get_tasks, schedule_task, Task},
    screens::{
        close_screen, CurrentScreen, EscapeScreen, PlayerInventoryScreen, ScreenDimensions, SelectorScreen
    },
    serialization::{self, Deserialize, SerializationTrap, Serialize},
    world::{ChunkBlockMetadata, Direction, Vec2i, World, BLOCK_DEFAULT_H, BLOCK_DEFAULT_W},
    RenderFn, RENDER_STEP,
};
use raylib::{
    color::Color,
    drawing::RaylibDrawHandle,
    math::{Rectangle, Vector2},
    RaylibHandle,
};
use raylib::{drawing::RaylibDraw, ffi::KeyboardKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Block,
    OverlayItems,
    Preview,
}

impl RenderLayer {
    pub fn default_preview() -> Self {
        Self::Preview
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

macro_rules! lerp_step {
    ($lerp: expr, $step: expr, $num_steps: expr) => {{
        let _ = $lerp / 1.0_f32;
        let _ = $step / 1.0_f32;
        let _ = $num_steps / 1.0_f32;
        let computed_step_off: f32 = 1.0 / $num_steps * $step;
        if $lerp < computed_step_off {
            0.0
        } else if $lerp >= 1.0 / $num_steps * ($step + 1.0) {
            1.0
        } else {
            (($lerp - computed_step_off) * $num_steps)
        }
    }};
}

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

    let mut dismantle_timer: Option<Instant> = None;
    let mut dismantle_timer_start: Option<Instant> = None;
    let mut dismantle_positions: Vec<Vec2i> = Vec::new();

    let blk_w = BLOCK_DEFAULT_W;
    let blk_h = BLOCK_DEFAULT_H;

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
                // Task::Custom(func) => func(),
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
            if rl.is_key_down(KeyboardKey::KEY_W) {
                direction.y -= (dt * 0.8) as f32;
            }
            if rl.is_key_down(KeyboardKey::KEY_S) {
                direction.y += (dt * 0.8) as f32;
            }
            if rl.is_key_down(KeyboardKey::KEY_A) {
                direction.x -= (dt * 0.8) as f32;
            }
            if rl.is_key_down(KeyboardKey::KEY_D) {
                direction.x += (dt * 0.8) as f32;
            }
            if direction.x != 0.0 && direction.y != 0.0 {
                direction.x *= 0.7;
                direction.y *= 0.7;
            }
            if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                direction.x *= 1.5;
                direction.y *= 1.5;
            }
            // if rl.is_key_pressed(KeyboardKey::KEY_ZERO) && is_ctrl!(rl) {
            //     blk_w = BLOCK_DEFAULT_W;
            //     blk_h = BLOCK_DEFAULT_H;
            // }
            // if rl.is_key_pressed(KeyboardKey::KEY_UP) && is_ctrl!(rl) {
            //     blk_w += 5;
            //     blk_h += 5;
            // }
            // if rl.is_key_pressed(KeyboardKey::KEY_DOWN) && is_ctrl!(rl) && blk_w > 8 && blk_h > 8 {
            //     blk_w -= 8;
            //     blk_h -= 8;
            // }
            config.player.x += direction.x as i32;
            config.player.y += direction.y as i32;
            if rl.is_key_down(KeyboardKey::KEY_TAB) {
                CurrentScreen::open_centered(
                    Box::new(PlayerInventoryScreen::default()),
                    &screen_size,
                );
            }
            if rl.is_key_pressed(KeyboardKey::KEY_B) {
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
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
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
        let mut cursor_x = (cursor_pos.x as i32 + config.player.x) / blk_w as i32;
        let mut cursor_y = (cursor_pos.y as i32 + config.player.y) / blk_h as i32;

        if (cursor_pos.x as i32 + config.player.x) < 0 {
            cursor_x -= 1;
        }
        if (cursor_pos.y as i32 + config.player.y) < 0 {
            cursor_y -= 1;
        }

        let mut off_x = config.player.x % blk_w as i32;
        let mut off_y = config.player.y % blk_h as i32;
        if off_x < 0 {
            off_x += blk_w as i32;
        }
        if off_y < 0 {
            off_y += blk_w as i32;
        }

        let overlay_x =
            (make_abs(cursor_pos.x as i32 + off_x).wrapping_div(blk_w) * blk_w) as i32 - off_x;
        let overlay_y =
            (make_abs(cursor_pos.y as i32 + off_y).wrapping_div(blk_h) * blk_h) as i32 - off_y;

        let (can_build, can_dismantle) = {
            let blk = world.get_block_at(cursor_x, cursor_y);
            (
                blk.map(|blk| blk.0.is_none()).unwrap_or(false),
                blk.map(|blk| !blk.0.is_none()).unwrap_or(false),
            )
        };

        if (rl.is_key_pressed(KeyboardKey::KEY_LEFT_SHIFT)
            || rl.is_key_pressed(KeyboardKey::KEY_RIGHT_SHIFT))
            && game_focused
            && can_dismantle
            && matches!(config.interaction_mode, InteractionMode::Dismantling)
        {
            if let Some(idx) = dismantle_positions
                .iter()
                .position(|val| val.x == cursor_x && val.y == cursor_y)
            {
                dismantle_positions.remove(idx);
            } else {
                dismantle_positions.push(Vec2i::new(cursor_x, cursor_y));
            }
        }

        if rl.is_mouse_button_down(raylib::ffi::MouseButton::MOUSE_LEFT_BUTTON) && game_focused {
            match config.interaction_mode {
                InteractionMode::Building if can_build => {
                    let mut blk = config.current_selected_block.clone_block();
                    blk.on_before_place(
                        ChunkBlockMetadata::new(config.direction, Vec2i::new(cursor_x, cursor_y)),
                        &mut world,
                    );
                    world.set_block_at(cursor_x, cursor_y, blk, config.direction);
                }
                InteractionMode::Dismantling if can_dismantle || dismantle_positions.len() > 0 => {
                    if let Some(timer) = dismantle_timer {
                        if timer <= Instant::now() {
                            if can_dismantle {
                                if let Some((mut blk, meta)) = world.destroy_block_at(cursor_x, cursor_y, &mut config.inventory) {
                                    blk.on_after_dismantle(meta, &mut world);
                                }
                            }
                            for vec in &dismantle_positions {
                                if let Some((mut blk, meta)) = world.destroy_block_at(vec.x, vec.y, &mut config.inventory) {
                                    blk.on_after_dismantle(meta, &mut world);
                                }
                            }
                            dismantle_positions.clear();
                            let mut now = Instant::now();
                            now += Duration::new(2, 0);
                            dismantle_timer = Some(now);
                            dismantle_timer_start = Some(Instant::now());
                        }
                    } else {
                        let mut now = Instant::now();
                        now += Duration::new(2, 0);
                        dismantle_timer = Some(now);
                        dismantle_timer_start = Some(Instant::now());
                    }
                }
                _ => {}
            }
        }
        if (dismantle_timer.is_some() || dismantle_timer_start.is_some())
            && (!rl.is_mouse_button_down(raylib::ffi::MouseButton::MOUSE_LEFT_BUTTON)
                || !game_focused
                || !matches!(config.interaction_mode, InteractionMode::Dismantling)
                || (!can_dismantle && dismantle_positions.len() < 1))
        {
            dismantle_timer.take();
            dismantle_timer_start.take();
        }
        if dismantle_positions.len() > 0
            && !matches!(config.interaction_mode, InteractionMode::Dismantling)
        {
            dismantle_positions.clear();
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
                    blk_w,
                    blk_h,
                );
            }
        }

        if game_focused {
            match config.interaction_mode {
                InteractionMode::Building if can_build => {
                    config.current_selected_block.render_build_overlay(
                        &mut d,
                        overlay_x,
                        overlay_y,
                        blk_w as i32,
                        blk_h as i32,
                        ChunkBlockMetadata::new(config.direction, Vec2i::new(cursor_x, cursor_y)),
                        config.player,
                    );
                    d.draw_rectangle(
                        overlay_x,
                        overlay_y,
                        blk_w as i32,
                        blk_h as i32,
                        Color::GRAY.fade(0.5),
                    );
                }
                InteractionMode::Dismantling if can_dismantle || dismantle_positions.len() > 0 => {
                    if let Some(timer_start) = dismantle_timer_start {
                        let lerp = (Instant::now() - timer_start).as_millis() as f32 / 2000 as f32;
                        if can_dismantle {
                            draw_dismantle_animation(
                                &mut d,
                                lerp,
                                overlay_x as i32,
                                overlay_y as i32,
                                &screen_size,
                                blk_w,
                                blk_h,
                            );
                        }
                        for pos in dismantle_positions
                            .iter()
                            .filter(|pos| pos.x != cursor_x || pos.y != cursor_y)
                            .map(|&pos| {
                                world.get_effective_render_position(
                                    pos,
                                    config.player,
                                    blk_w,
                                    blk_h,
                                )
                            })
                        {
                            draw_dismantle_animation(
                                &mut d,
                                lerp,
                                pos.x,
                                pos.y,
                                &screen_size,
                                blk_w,
                                blk_h,
                            );
                        }
                    }
                    for pos in dismantle_positions
                        .iter()
                        .filter(|pos| pos.x != cursor_x || pos.y != cursor_y)
                        .map(|&pos| {
                            world.get_effective_render_position(pos, config.player, blk_w, blk_h)
                        })
                    {
                        d.draw_rectangle(
                            pos.x,
                            pos.y,
                            blk_w as i32,
                            blk_h as i32,
                            Color::RED.fade(0.25),
                        );
                    }
                    if can_dismantle {
                        d.draw_rectangle(
                            overlay_x,
                            overlay_y,
                            blk_w as i32,
                            blk_h as i32,
                            Color::RED.fade(0.25),
                        );
                    }
                }
                _ => {}
            }

            if let Some((block, data)) = world.get_block_at_mut(cursor_x, cursor_y) {
                if block.supports_interaction() {
                    d.draw_text(
                        block
                            .custom_interact_message()
                            .unwrap_or_else(|| format!("Press F to interact with {}", block.name()))
                            .as_str(),
                        overlay_x,
                        overlay_y + blk_h as i32 + 5,
                        20,
                        Color::BLACK,
                    );
                    if d.is_key_pressed(KeyboardKey::KEY_F) {
                        block.interact(data, &mut config);
                    }
                }
            }
        }

        match config.interaction_mode {
            InteractionMode::Building => {
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
            InteractionMode::Dismantling => {
                d.draw_text(
                    "Dismantling",
                    20 + 1,
                    screen_size.height - 67,
                    20,
                    Color::BLACK,
                );
                d.draw_text(
                    "Dismantling",
                    20 + 2,
                    screen_size.height - 66,
                    20,
                    Color::BLACK,
                );
                d.draw_text("Dismantling", 20, screen_size.height - 68, 20, Color::RED);
            }
            InteractionMode::None => {}
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

fn draw_dismantle_animation(
    d: &mut RaylibDrawHandle,
    lerp: f32,
    x: i32,
    y: i32,
    screen: &ScreenDimensions,
    blk_w: u32,
    blk_h: u32,
) {
    if ((x + blk_w as i32) < 0 && (y + blk_h as i32) < 0)
        || (x >= screen.width && y >= screen.height)
    {
        return;
    }

    d.draw_rectangle(x, y, blk_w as i32, blk_h as i32, Color::BLACK.fade(0.5));

    let lerp_step_1 = lerp_step!(lerp, 0.0, 4.0) * blk_w as f32;
    let lerp_step_2 = lerp_step!(lerp, 1.0, 4.0) * blk_h as f32;
    let lerp_step_3 = lerp_step!(lerp, 2.0, 4.0) * (blk_w - 1) as f32;
    let lerp_step_4 = lerp_step!(lerp, 3.0, 4.0) * (blk_h - 1) as f32;

    d.draw_rectangle(x, y, lerp_step_1 as i32, 2, Color::RED);
    d.draw_rectangle(x + blk_w as i32 - 2, y, 2, lerp_step_2 as i32, Color::RED);
    d.draw_line_ex(
        Vector2::new((x + blk_w as i32 - 1) as f32, (y + blk_h as i32 - 1) as f32),
        Vector2::new(
            x as f32 + blk_w as f32 - 1.0 - lerp_step_3,
            (y + blk_h as i32 - 1) as f32,
        ),
        2.0,
        Color::RED,
    );
    d.draw_line_ex(
        Vector2::new(x as f32 + 1.0, (y + blk_h as i32 - 1) as f32),
        Vector2::new(x as f32 + 1.0, (y + blk_h as i32 - 1) as f32 - lerp_step_4),
        2.0,
        Color::RED,
    );
}
