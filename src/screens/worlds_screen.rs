use std::{ffi::CStr, fs::read_dir};

use lazy_static::lazy_static;
use raylib::{drawing::RaylibScissorModeExt, math::Rectangle, rgui::RaylibDrawGui};

use crate::{
    asset, cstr, identifier::GlobalString, notice_board::{self, NoticeboardEntryRenderable}, scheduler::{schedule_task, Task}, screens::DialogBox
};

use super::{Screen, ScreenDimensions};

pub struct WorldScreen(Vec<Vec<u8>>, u32);

lazy_static! {
    pub static ref NAME: GlobalString = GlobalString::from("Worlds");
    pub static ref NAME_LOADING: GlobalString = GlobalString::from("Loading");
}

const NEW: &CStr = cstr!("Create new World");

impl WorldScreen {
    pub fn new() -> std::io::Result<Box<Self>> {
        read_dir(asset!("worlds")).map(|dirs| {
            let mut entries: Vec<Vec<u8>> = Vec::with_capacity(30);

            for p in dirs {
                if let Ok(p) = p {
                    let mut vec = p.file_name().as_encoded_bytes().to_vec();
                    if vec[vec.len() - 1] != 0 {
                        vec.push(0);
                    }
                    entries.push(vec)
                } else {
                    continue;
                };
            }

            Box::new(Self(entries, 0))
        })
    }
}

const HEIGHT: i32 = 24;
const PADDING: i32 = 10;

impl Screen for WorldScreen {
    fn name(&mut self) -> GlobalString {
        *NAME
    }

    fn rect(&mut self, screen: &ScreenDimensions) -> ScreenDimensions {
        ScreenDimensions {
            width: 280,
            height: (screen.height - 35) / 4 * 3,
        }
    }

    fn render(
        &mut self,
        _: &mut crate::GameConfig,
        renderer: &mut raylib::prelude::RaylibDrawHandle,
        x: i32,
        mut y: i32,
        w: i32,
        h: i32,
        _: &mut crate::world::World,
    ) {
        let max_height =
            (self.0.len() as i32 * (HEIGHT + PADDING) + (HEIGHT + PADDING)).saturating_sub(h);

        if max_height > 0 {
            self.1 = renderer.gui_scroll_bar(
                Rectangle::new((x + w - 10) as f32, (y + 10) as f32, 10.0, (h - 20) as f32),
                self.1 as i32,
                0,
                max_height,
            ) as u32;
        } else {
            self.1 = 0;
        }

        let mut renderer = renderer.begin_scissor_mode(x, y, w, h);

        y -= self.1 as i32;

        for i in 0..self.0.len() {
            if renderer.gui_button(
                Rectangle::new(
                    (x + 20) as f32,
                    ((i + 1) as i32 * (HEIGHT + PADDING) + y + PADDING) as f32,
                    240.0,
                    24.0,
                ),
                unsafe { Some(CStr::from_bytes_with_nul_unchecked(self.0[i].as_slice())) },
            ) {
                if let Ok(mut name) = String::from_utf8(self.0[i].clone()) {
                    println!("Load {}", String::from_utf8_lossy(self.0[i].as_slice()));
                    schedule_task(Task::OpenScreenCentered(DialogBox::new_uncloseable(
                        Some(*NAME_LOADING),
                        format!(
                            "Loading world {}...",
                            String::from_utf8_lossy(&self.0[i][0..self.0[i].len() - 1])
                        ),
                    )));
                    name.pop();
                    let path = asset!("worlds", name);
                    schedule_task(Task::OpenWorld(path));
                } else {
                    self.close();
                    notice_board::add_entry(NoticeboardEntryRenderable::StringRef("Could not load savefile"), 5);
                }
            }
        }

        if renderer.gui_button(
            Rectangle::new((x + 20) as f32, (y + PADDING) as f32, 240.0, 24.0),
            Some(NEW),
        ) {
            schedule_task(Task::CreateWorld);
        }
    }
}
