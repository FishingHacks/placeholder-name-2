use std::{ffi::CStr, fmt::Display, sync::Mutex};

use raylib::{drawing::RaylibDrawHandle, math::Rectangle, rgui::RaylibDrawGui};

mod player_inventory_screen;
mod escape_screen;
mod selector_screen;
mod container_inventory_screen;
mod main_screen;
pub use selector_screen::SelectorScreen;
pub use escape_screen::EscapeScreen;
pub use player_inventory_screen::PlayerInventoryScreen;
pub use container_inventory_screen::ContainerInventoryScreen;
pub use main_screen::MainScreen;

use crate::{identifier::GlobalString, scheduler::{schedule_task, Task}, world::World, GameConfig};

#[derive(Debug, Clone, Copy)]
pub struct ScreenDimensions {
    pub width: i32,
    pub height: i32,
}

impl Display for ScreenDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Screen<{}x{}>", self.width, self.height))
    }
}


trait Screen {
    fn rect(&mut self, screen: &ScreenDimensions) -> ScreenDimensions;
    fn render(&mut self, cfg: &mut GameConfig, renderer: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32, world: &mut World);
    fn name(&mut self) -> GlobalString;
    fn close(&self) {
        schedule_task(Task::CloseScreen);
    }
}

pub trait GUIScreen: Send {
    fn render(&mut self, cfg: &mut GameConfig, renderer: &mut RaylibDrawHandle, x: i32, y: i32, screen: &ScreenDimensions, world: &mut World);
    fn get_dimensions(&mut self, screen: &ScreenDimensions) -> ScreenDimensions;
    fn close_screen(&self) {
        schedule_task(Task::CloseScreen);
    }
    fn name(&mut self) -> GlobalString;
    fn is_in_bounds(&mut self, x: i32, y: i32, screen: &ScreenDimensions) -> bool {
        let ScreenDimensions { width, height } = self.get_dimensions(screen);

        x < 0 || y < 0 || x >= width || y >= height
    }
}

static CURRENT_SCREEN: Mutex<(Option<Box<dyn GUIScreen>>, i32, i32)> = Mutex::new((None, 0, 0));

pub fn open_screen(screen: Box<dyn GUIScreen>, x: i32, y: i32) {
    let mut sc = CURRENT_SCREEN.lock().unwrap();
    *sc = (Some(screen), x, y);
}

pub fn move_screen(x: i32, y: i32) {
    let mut cur_screen = CURRENT_SCREEN.lock().unwrap();

    cur_screen.1 = x;
    cur_screen.2 = y;
}

pub fn close_screen() {
    *CURRENT_SCREEN.lock().unwrap() = (None, 0, 0);
}

impl<T: Screen + Send> GUIScreen for T {
    fn get_dimensions(&mut self, screen: &ScreenDimensions) -> ScreenDimensions {
        let mut dimensions = self.rect(screen);
        // padding: 5 px on each side and 30 on the top
        dimensions.width += 10;
        dimensions.height += 35;
        dimensions
    }

    fn name(&mut self) -> GlobalString {
        Screen::name(self)
    }

    fn render(&mut self, cfg: &mut GameConfig, renderer: &mut RaylibDrawHandle, x: i32, y: i32, screen: &ScreenDimensions, world: &mut World) {
        let ScreenDimensions { width, height } = self.rect(screen);

        let mut name = self.name().as_str().clone();
        name.push('\0');

        if renderer.gui_window_box(
            Rectangle::new(
                x as f32,
                y as f32,
                (width + 10) as f32,
                (height + 35) as f32,
            ),
            Some(CStr::from_bytes_with_nul(name.as_bytes()).unwrap()),
        ) {
            self.close();
        }

        Screen::render(self, cfg, renderer, x + 5, y + 30, width, height, world);
    }
}

pub struct CurrentScreen;

impl CurrentScreen {
    pub fn get_dimensions(screen: &ScreenDimensions) -> ScreenDimensions {
        match &mut CURRENT_SCREEN.lock().unwrap().0 {
            None => ScreenDimensions {
                width: 0,
                height: 0,
            },
            Some(sc) => sc.get_dimensions(screen),
        }
    }

    // pub fn is(name: &str) -> bool {
    //     match &mut CURRENT_SCREEN.lock().unwrap().0 {
    //         None => false,
    //         Some(v) => v.name().as_str() == name,
    //     }
    // }

    pub fn move_to_center(screen: &ScreenDimensions) {
        let dim = Self::get_dimensions(screen);
        let x = (screen.width - dim.width) / 2;
        let y = (screen.height - dim.height) / 2;
        move_screen(x, y);
    }

    pub fn render(cfg: &mut GameConfig, renderer: &mut RaylibDrawHandle, screen: &ScreenDimensions, world: &mut World) {
        let mut sc = CURRENT_SCREEN.lock().unwrap();
        let x = sc.1;
        let y = sc.2;
        match &mut sc.0 {
            None => {}
            Some(sc) => sc.render(cfg, renderer, x, y, screen, world),
        }
    }

    pub fn is_screen_open() -> bool {
        CURRENT_SCREEN.lock().unwrap().0.is_some()
    }

    pub fn close() {
        schedule_task(Task::CloseScreen);
    }

    pub fn open_centered(mut screen: Box<dyn GUIScreen>, window: &ScreenDimensions) {
        let screen_dims = screen.get_dimensions(window);
        let x = (window.width - screen_dims.width) / 2;
        let y = (window.height - screen_dims.height) / 2;

        open_screen(screen, x, y);
    }

    // pub fn is_in_bounds(x: i32, y: i32, screen: &ScreenDimensions) -> bool {
    //     let mut sc = CURRENT_SCREEN.lock().unwrap();
    //     let sc_x = sc.1;
    //     let sc_y = sc.2;
    //     match &mut sc.0 {
    //         None => false,
    //         Some(sc) => sc.is_in_bounds(x - sc_x, y - sc_y, screen),
    //     }
    // }
}
