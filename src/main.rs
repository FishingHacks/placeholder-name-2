use std::{sync::Mutex, time::Instant};

use assets::update_textures;
use blocks::{
    empty_block, get_block_by_id, load_block_files, register_blocks, Block, BLOCK_EMPTY,
    BLOCK_RESOURCE_NODE_BLUE,
};
use inventory::{Inventory, NUM_SLOTS_PLAYER};
use items::register_items;
use notice_board::NoticeboardEntryRenderable;
use raylib::{
    color::Color, drawing::RaylibDraw, ffi::KeyboardKey, math::{Rectangle, Vector2}, RaylibHandle
};
use scheduler::{get_tasks, schedule_task, Task};
use screens::{
    close_screen, CurrentScreen, EscapeScreen, MainScreen, PlayerInventoryScreen, ScreenDimensions,
    SelectorScreen,
};
use serialization::load_game;
use world::{ChunkBlockMetadata, Direction, World, BLOCK_H, BLOCK_W};

pub mod as_any;
pub mod assets;
pub mod blocks;
pub mod identifier;
pub mod initialized_data;
mod inventory;
pub mod items;
pub mod notice_board;
pub mod scheduler;
mod screens;
pub mod serialization;
pub mod ui;
mod world;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Block,
    OverlayItems,
    OverlayGUI,
}

impl RenderLayer {
    pub fn default_preview() -> Self {
        Self::Block
    }
}

pub const RENDER_LAYERS: [RenderLayer; 3] = [
    RenderLayer::Block,
    RenderLayer::OverlayItems,
    RenderLayer::OverlayGUI,
];

#[macro_export]
macro_rules! cstr {
    ($str: expr) => {
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes()) }
    };
}

fn make_abs(val: i32) -> u32 {
    if val >= 0 {
        val as u32
    } else {
        0
    }
}

#[derive(Clone)]
pub struct GameConfig {
    current_selected_block: &'static Box<dyn Block>,
    direction: Direction,
    inventory: Inventory,
}

impl GameConfig {
    pub fn default() -> Self {
        Self {
            current_selected_block: get_block_by_id(*BLOCK_RESOURCE_NODE_BLUE)
                .unwrap_or(empty_block()),
            direction: Direction::North,
            inventory: Inventory::new(NUM_SLOTS_PLAYER, true),
        }
    }
}

pub const TPS: u32 = 20;
pub const MSPT: u128 = (1000 / TPS) as u128;

pub enum RenderFn {
    None,
    Game(World, GameConfig),
    StartMenu,
}

impl RenderFn {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }
}

static RENDER_STEP: Mutex<RenderFn> = Mutex::new(RenderFn::StartMenu);

fn run_game(
    rl: &mut RaylibHandle,
    thread: &raylib::prelude::RaylibThread,
    mut world: World,
    mut config: GameConfig,
) {
    world.init();

    let mut player_x: i32 = 0;
    let mut player_y: i32 = 0;

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
            match t {
                scheduler::Task::Custom(func) => func(),
                scheduler::Task::ExitGame => return,
                scheduler::Task::OpenScreenCentered(screen) => {
                    CurrentScreen::open_centered(screen, &screen_size)
                }
                scheduler::Task::CloseScreen => close_screen(),
                scheduler::Task::WorldUpdateBlock(func, meta) => {
                    had_gameupdate_scheduled = true;
                    func(meta, &mut world);
                }
                Task::CloseWorld => {
                    *RENDER_STEP.lock().unwrap() = RenderFn::StartMenu;
                    return;
                }
                Task::OpenWorld(..) | Task::CreateWorld => notice_board::add_entry(
                    NoticeboardEntryRenderable::String(
                        "WARN!! RECEIVED WORLD OPENING TASK IN RUN_GAME(..)".to_string(),
                    ),
                    20,
                ),
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
            player_x += direction.x as i32;
            player_y += direction.y as i32;
            if rl.is_key_down(raylib::ffi::KeyboardKey::KEY_TAB) {
                CurrentScreen::open_centered(
                    Box::new(PlayerInventoryScreen::default()),
                    &screen_size,
                );
            }
            if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_B) {
                CurrentScreen::open_centered(Box::new(SelectorScreen), &screen_size);
            }
            if rl.get_mouse_wheel_move() != 0.0 {
                let right = rl.get_mouse_wheel_move() > 0.0;
                config.direction = config.direction.next(right);
            }
        }
        if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_ESCAPE) {
            if !game_focused {
                CurrentScreen::close();
            } else {
                CurrentScreen::open_centered(Box::new(EscapeScreen), &screen_size);
            }
        }

        let cursor_pos = rl.get_mouse_position();
        let mut cursor_x = (cursor_pos.x as i32 + player_x) / BLOCK_W as i32;
        let mut cursor_y = (cursor_pos.y as i32 + player_y) / BLOCK_H as i32;

        if (cursor_pos.x as i32 + player_x) < 0 {
            cursor_x -= 1;
        }
        if (cursor_pos.y as i32 + player_y) < 0 {
            cursor_y -= 1;
        }

        let mut off_x = player_x % BLOCK_W as i32;
        let mut off_y = player_y % BLOCK_H as i32;
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

        if rl.is_mouse_button_down(raylib::ffi::MouseButton::MOUSE_LEFT_BUTTON)
            && game_focused
            && config.current_selected_block.identifier() != *BLOCK_EMPTY
        {
            world.set_block_at(
                cursor_x,
                cursor_y,
                config.current_selected_block.clone_block(),
                config.direction,
            );
        }
        if rl.is_mouse_button_down(raylib::ffi::MouseButton::MOUSE_RIGHT_BUTTON) && game_focused {
            world.set_block_at(
                cursor_x,
                cursor_y,
                empty_block().clone_block(),
                Direction::North,
            );
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
                    player_x,
                    player_y,
                    screen_size.width as u32,
                    screen_size.height as u32,
                    l,
                );
            }
        }

        if game_focused {
            if config.current_selected_block.identifier() != *BLOCK_EMPTY {
                d.draw_rectangle(
                    overlay_x,
                    overlay_y,
                    BLOCK_W as i32,
                    BLOCK_H as i32,
                    Color::GRAY.fade(0.5),
                );
            }

            if let Some((block, data)) = world.get_block_at(cursor_x, cursor_y) {
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

        if config.current_selected_block.identifier() != *BLOCK_EMPTY {
            config.current_selected_block.render_all(
                &mut d,
                20,
                screen_size.height - 68,
                48,
                48,
                ChunkBlockMetadata::from(config.direction),
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
            format!("X: {player_x} Y: {player_y} | Facing: {cursor_x} {cursor_y}").as_str(),
            5,
            25,
            20,
            Color::DARKGREEN,
        );

        CurrentScreen::render(&mut config, &mut d, &screen_size, &mut world);
        
        notice_board::render_entries(&mut d, screen_size.height / 2, screen_size.height);
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Placeholder Name 2")
        .build();
    #[cfg(not(target_os = "linux"))]
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Placeholder Name 2 with vsync")
        .vsync() // nvidia fucks with vsync :sob:
        .build();

    rl.set_exit_key(None);

    if let Err(e) = load_block_files(&mut rl, &thread) {
        panic!("Encountered an error while trying to load the block files:\n{e}");
    }
    register_blocks();
    register_items();

    while !rl.window_should_close() {
        let render_fn = RENDER_STEP.lock().unwrap().take();

        reset_all();

        match render_fn {
            RenderFn::None => return,
            RenderFn::StartMenu => render_menu(&mut rl, &thread),
            RenderFn::Game(world, cfg) => run_game(&mut rl, &thread, world, cfg),
        }
    }
}

pub fn render_menu(rl: &mut RaylibHandle, thread: &raylib::prelude::RaylibThread) {
    let mut cfg = GameConfig::default();
    let mut empty_world = World::new(0, 0);

    let mut old_sc = ScreenDimensions {
        width: rl.get_screen_width(),
        height: rl.get_screen_height(),
    };

    while !rl.window_should_close() {
        let sc = ScreenDimensions {
            width: rl.get_screen_width(),
            height: rl.get_screen_height(),
        };

        if old_sc.width != sc.width || old_sc.height != sc.height {
            old_sc.width = sc.width;
            old_sc.height = sc.height;
            CurrentScreen::move_to_center(&sc);
        }

        for t in get_tasks() {
            match t {
                Task::CloseWorld | Task::WorldUpdateBlock(..) => {}
                Task::CloseScreen => close_screen(),
                Task::OpenScreenCentered(screen) => CurrentScreen::open_centered(screen, &sc),
                Task::ExitGame => return,
                Task::Custom(func) => func(),
                Task::CreateWorld => {
                    *RENDER_STEP.lock().unwrap() =
                        RenderFn::Game(World::new(20, 20), GameConfig::default());
                    return;
                }
                Task::OpenWorld(file) => match load_game(file) {
                    Ok((world, cfg, _)) => {
                        *RENDER_STEP.lock().unwrap() = RenderFn::Game(world, cfg);
                        return;
                    }
                    Err(e) => notice_board::add_entry(
                        NoticeboardEntryRenderable::String(format!("Couldn't load World: {e:?}")),
                        20,
                    ),
                },
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_ESCAPE) {
            CurrentScreen::close();
        }

        let mut d = rl.begin_drawing(thread);

        d.clear_background(Color::new(0x1e, 0x1e, 0x2e, 0xff));

        if !CurrentScreen::is_screen_open() {
            CurrentScreen::open_centered(Box::new(MainScreen), &sc);
        }
        CurrentScreen::render(&mut cfg, &mut d, &sc, &mut empty_world);
        notice_board::render_entries(&mut d, sc.height / 2, sc.height);
    }
}

pub fn reset_all() {
    close_screen();
    get_tasks();
    notice_board::reset();
}
