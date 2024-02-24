use std::{sync::Mutex, thread, time::Instant};

use assets::update_textures;
use blocks::{empty_block, load_block_files, register_blocks, Block, BLOCK_EMPTY};
use inventory::{Inventory, NUM_SLOTS_PLAYER};
use items::register_items;
use notice_board::NoticeboardEntryRenderable;
use raylib::{
    color::Color,
    drawing::RaylibDraw,
    ffi::KeyboardKey,
    math::{Rectangle, Vector2},
    RaylibHandle,
};
use scheduler::{get_tasks, schedule_task, Task};
use screens::{
    close_screen, CurrentScreen, EscapeScreen, MainScreen, PlayerInventoryScreen, ScreenDimensions,
    SelectorScreen,
};
use serialization::{load_game, Deserialize, SerializationTrap, Serialize};
use world::{ChunkBlockMetadata, Direction, Vec2i, World, BLOCK_H, BLOCK_W};

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
}

impl RenderLayer {
    pub fn default_preview() -> Self {
        Self::Block
    }
}

pub const RENDER_LAYERS: [RenderLayer; 2] = [RenderLayer::Block, RenderLayer::OverlayItems];

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
    player: Vec2i,
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
            if rl.get_mouse_wheel_move() != 0.0 {
                let right = rl.get_mouse_wheel_move() > 0.0;
                config.direction = config.direction.next(right);
            }
        }
        if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_ESCAPE) {
            if !game_focused {
                CurrentScreen::close();
            } else if !config.current_selected_block.is_none() {
                config.current_selected_block = empty_block();
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
                    config.player.x,
                    config.player.y,
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

    styles::dark();

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
                Task::__OpnWrld(world, cfg) => {
                    *RENDER_STEP.lock().unwrap() = RenderFn::Game(world, cfg);
                    return;
                }
                Task::OpenWorld(file) => {
                    thread::spawn(move || match load_game(file) {
                        Ok((world, cfg, _)) => {
                            schedule_task(Task::__OpnWrld(world, cfg));
                        }
                        Err(e) => {
                            notice_board::add_entry(
                                NoticeboardEntryRenderable::String(format!(
                                    "Couldn't load World: {e:?}"
                                )),
                                20,
                            );
                            schedule_task(Task::CloseScreen);
                        }
                    });
                }
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

pub mod styles {
    use std::ffi::CStr;

    macro_rules! apply_set_style {
        ($(p $ctrl: expr, $prop: expr, $val: expr,)*) => {
            // unsafe raylib ffi functions :fearful:
            unsafe {
                $(
                    raylib::ffi::GuiSetStyle($ctrl as i32, $prop as i32, i32::from_le_bytes(u32::to_le_bytes($val)));
                )*
            }
        };
    }

    pub fn jungle() {
        apply_set_style!(
            p 00, 00, 0x60827dff,
            p 00, 01, 0x2c3334ff,
            p 00, 02, 0x82a29fff,
            p 00, 03, 0x5f9aa8ff,
            p 00, 04, 0x334e57ff,
            p 00, 05, 0x6aa9b8ff,
            p 00, 06, 0xa9cb8dff,
            p 00, 07, 0x3b6357ff,
            p 00, 08, 0x97af81ff,
            p 00, 09, 0x5b6462ff,
            p 00, 10, 0x2c3334ff,
            p 00, 11, 0x666b69ff,
            p 00, 18, 0x638465ff,
            p 00, 19, 0x2b3a3aff,
            p 00, 20, 0x00000012,
        );
    }

    pub fn lavenda() {
        apply_set_style!(
            p 00, 00, 0xab9bd3ff,
            p 00, 01, 0x3e4350ff,
            p 00, 02, 0xdadaf4ff,
            p 00, 03, 0xee84a0ff,
            p 00, 04, 0xf4b7c7ff,
            p 00, 05, 0xb7657bff,
            p 00, 06, 0xd5c8dbff,
            p 00, 07, 0x966ec0ff,
            p 00, 08, 0xd7ccf7ff,
            p 00, 09, 0x8fa2bdff,
            p 00, 10, 0x6b798dff,
            p 00, 11, 0x8292a9ff,
            p 00, 18, 0x84adb7ff,
            p 00, 19, 0x5b5b81ff,
        );
    }

    pub fn default() {
        unsafe {
            raylib::ffi::GuiLoadStyleDefault();
        }
    }

    pub fn cyber() {
        apply_set_style!(
            p 00, 00, 0x2f7486ff,
            p 00, 01, 0x024658ff,
            p 00, 02, 0x51bfd3ff,
            p 00, 03, 0x82cde0ff,
            p 00, 04, 0x3299b4ff,
            p 00, 05, 0xb6e1eaff,
            p 00, 06, 0xeb7630ff,
            p 00, 07, 0xffbc51ff,
            p 00, 08, 0xd86f36ff,
            p 00, 09, 0x134b5aff,
            p 00, 10, 0x02313dff,
            p 00, 11, 0x17505fff,
            p 00, 18, 0x81c0d0ff,
            p 00, 19, 0x00222bff,
        );
    }

    pub fn candy() {
        apply_set_style!(
            p 00, 00, 0xe58b68ff,
            p 00, 01, 0xfeda96ff,
            p 00, 02, 0xe59b5fff,
            p 00, 03, 0xee813fff,
            p 00, 04, 0xfcd85bff,
            p 00, 05, 0xfc6955ff,
            p 00, 06, 0xb34848ff,
            p 00, 07, 0xeb7272ff,
            p 00, 08, 0xbd4a4aff,
            p 00, 09, 0x94795dff,
            p 00, 10, 0xc2a37aff,
            p 00, 11, 0x9c8369ff,
            p 00, 18, 0xd77575ff,
            p 00, 19, 0xfff5e1ff,
        );
    }

    pub fn terminal() {
        apply_set_style!(
            p 00, 00, 0x1c8d00ff,
            p 00, 01, 0x161313ff,
            p 00, 02, 0x38f620ff,
            p 00, 03, 0xc3fbc6ff,
            p 00, 04, 0x43bf2eff,
            p 00, 05, 0xdcfadcff,
            p 00, 06, 0x1f5b19ff,
            p 00, 07, 0x43ff28ff,
            p 00, 08, 0x1e6f15ff,
            p 00, 09, 0x223b22ff,
            p 00, 10, 0x182c18ff,
            p 00, 11, 0x244125ff,
            p 00, 18, 0xe6fce3ff,
            p 00, 19, 0x0c1505ff,
        );
    }

    pub fn ashes() {
        apply_set_style!(
            p 00, 00, 0xf0f0f0ff,
            p 00, 01, 0x868686ff,
            p 00, 02, 0xe6e6e6ff,
            p 00, 03, 0x929999ff,
            p 00, 04, 0xeaeaeaff,
            p 00, 05, 0x98a1a8ff,
            p 00, 06, 0x3f3f3fff,
            p 00, 07, 0xf6f6f6ff,
            p 00, 08, 0x414141ff,
            p 00, 09, 0x8b8b8bff,
            p 00, 10, 0x777777ff,
            p 00, 11, 0x959595ff,
            p 00, 19, 0x6b6b6bff,
        );
    }

    pub fn bluish() {
        apply_set_style!(
            p 00, 00, 0x5ca6a6ff,
            p 00, 01, 0xb4e8f3ff,
            p 00, 02, 0x447e77ff,
            p 00, 03, 0x5f8792ff,
            p 00, 04, 0xcdeff7ff,
            p 00, 05, 0x4c6c74ff,
            p 00, 06, 0x3b5b5fff,
            p 00, 07, 0xeaffffff,
            p 00, 08, 0x275057ff,
            p 00, 09, 0x96aaacff,
            p 00, 10, 0xc8d7d9ff,
            p 00, 11, 0x8c9c9eff,
            p 00, 18, 0x84adb7ff,
            p 00, 19, 0xe8eef1ff,
        );
    }

    pub fn dark() {
        apply_set_style!(
            p 00, 00, 0x878787ff,
            p 00, 01, 0x2c2c2cff,
            p 00, 02, 0xc3c3c3ff,
            p 00, 03, 0xe1e1e1ff,
            p 00, 04, 0x848484ff,
            p 00, 05, 0x181818ff,
            p 00, 06, 0x000000ff,
            p 00, 07, 0xefefefff,
            p 00, 08, 0x202020ff,
            p 00, 09, 0x6a6a6aff,
            p 00, 10, 0x818181ff,
            p 00, 11, 0x606060ff,
            p 00, 18, 0x9d9d9dff,
            p 00, 19, 0x3c3c3cff,
            p 01, 05, 0xf7f7f7ff,
            p 01, 08, 0x898989ff,
            p 04, 05, 0xb0b0b0ff,
            p 05, 05, 0x848484ff,
            p 09, 05, 0xf5f5f5ff,
            p 10, 05, 0xf6f6f6ff,
        );
    }

    pub fn cherry() {
        apply_set_style!(
            p 00, 00, 0xda5757ff,
            p 00, 01, 0x753233ff,
            p 00, 02, 0xe17373ff,
            p 00, 03, 0xfaaa97ff,
            p 00, 04, 0xe06262ff,
            p 00, 05, 0xfdb4aaff,
            p 00, 06, 0xe03c46ff,
            p 00, 07, 0x5b1e20ff,
            p 00, 08, 0xc2474fff,
            p 00, 09, 0xa19292ff,
            p 00, 10, 0x706060ff,
            p 00, 11, 0x9e8585ff,
            p 00, 18, 0xfb8170ff,
            p 00, 19, 0x3a1720ff,
        );
    }

    pub fn light() {
        default();
    }

    const JUNGLE: &CStr = cstr!("jungle");
    const LAVENDA: &CStr = cstr!("lavenda");
    const CYBER: &CStr = cstr!("cyber");
    const CANDY: &CStr = cstr!("candy");
    const TERMINAL: &CStr = cstr!("terminal");
    const ASHES: &CStr = cstr!("ashes");
    const BLUISH: &CStr = cstr!("bluish");
    const DARK: &CStr = cstr!("dark");
    const CHERRY: &CStr = cstr!("cherry");
    const LIGHT: &CStr = cstr!("light");

    pub const STYLES: &[(&'static CStr, &'static dyn Fn() -> ())] = &[
        (JUNGLE, &jungle),
        (LAVENDA, &lavenda),
        (CYBER, &cyber),
        (CANDY, &candy),
        (TERMINAL, &terminal),
        (ASHES, &ashes),
        (BLUISH, &bluish),
        (DARK, &dark),
        (CHERRY, &cherry),
        (LIGHT, &light),
    ];
}
