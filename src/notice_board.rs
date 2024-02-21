use std::{
    ops::Add,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    text::measure_text,
};

use crate::{
    blocks::Block,
    items::Item,
    world::{ChunkBlockMetadata, Direction}, RenderLayer,
};

pub enum NoticeboardEntryRenderable {
    String(String),
    Block(Box<dyn Block>, Direction),
    NamedBlock(Box<dyn Block>, Direction),
    Item(Box<dyn Item>),
    NamedItem(Box<dyn Item>),
    Joiner(
        Box<NoticeboardEntryRenderable>,
        Box<NoticeboardEntryRenderable>,
    ),
}

impl NoticeboardEntryRenderable {
    pub fn render(&self, x: i32, y: i32, renderer: &mut RaylibDrawHandle) -> i32 {
        match self {
            Self::String(str) => {
                let width = measure_text(str.as_str(), 20) + 10;
                renderer.draw_rectangle(x, y, width, ENTRY_SIZE, Color::WHITE.fade(0.5));
                renderer.draw_text(str.as_str(), x + 5, y + 5, 20, Color::BLACK);
                width
            }
            Self::Joiner(a, b) => {
                let width = a.render(x, y, renderer);
                width + b.render(x + width, y, renderer)
            }
            Self::Block(block, dir) => {
                renderer.draw_rectangle(x, y, ENTRY_SIZE, ENTRY_SIZE, Color::WHITE.fade(0.5));
                block.render(
                    renderer,
                    x + 3,
                    y + 3,
                    ENTRY_SIZE - 6,
                    ENTRY_SIZE - 6,
                    ChunkBlockMetadata::from(*dir),
                    RenderLayer::default_preview(),
                );

                ENTRY_SIZE
            }
            Self::NamedBlock(blk, dir) => {
                renderer.draw_rectangle(x, y, ENTRY_SIZE, ENTRY_SIZE, Color::WHITE.fade(0.5));
                blk.render(
                    renderer,
                    x + 3,
                    y + 3,
                    ENTRY_SIZE - 6,
                    ENTRY_SIZE - 6,
                    ChunkBlockMetadata::from(*dir),
                    RenderLayer::default_preview(),
                );

                let width = measure_text(blk.name().as_str(), 20) + 10;
                renderer.draw_rectangle(
                    x + ENTRY_SIZE,
                    y,
                    width,
                    ENTRY_SIZE,
                    Color::WHITE.fade(0.5),
                );
                renderer.draw_text(
                    blk.name().as_str(),
                    x + 5 + ENTRY_SIZE,
                    y + 5,
                    20,
                    Color::BLACK,
                );
                width + ENTRY_SIZE
            }
            Self::Item(item) => {
                renderer.draw_rectangle(x, y, ENTRY_SIZE, ENTRY_SIZE, Color::WHITE.fade(0.5));
                item.render(renderer, x + 3, y + 3, ENTRY_SIZE - 6, ENTRY_SIZE - 6);

                ENTRY_SIZE
            }
            Self::NamedItem(item) => {
                renderer.draw_rectangle(x, y, ENTRY_SIZE, ENTRY_SIZE, Color::WHITE.fade(0.5));
                item.render(renderer, x + 3, y + 3, ENTRY_SIZE - 6, ENTRY_SIZE - 6);

                let width = measure_text(item.name().as_str(), 20) + 10;
                renderer.draw_rectangle(
                    x + ENTRY_SIZE,
                    y,
                    width,
                    ENTRY_SIZE,
                    Color::WHITE.fade(0.5),
                );
                renderer.draw_text(
                    item.name().as_str(),
                    x + 5 + ENTRY_SIZE,
                    y + 5,
                    20,
                    Color::BLACK,
                );
                width + ENTRY_SIZE
            }
        }
    }
}

struct NoticeboardEntry {
    contents: NoticeboardEntryRenderable,
    should_decay: SystemTime,
}

static NOTICE_BOARD: Mutex<Vec<NoticeboardEntry>> = Mutex::new(Vec::new());

pub fn add_entry(contents: NoticeboardEntryRenderable, time_in_seconds: u32) {
    let entry = NoticeboardEntry {
        contents,
        should_decay: SystemTime::now().add(Duration::new(time_in_seconds as u64, 0)),
    };

    NOTICE_BOARD.lock().unwrap().push(entry);
}

pub fn update_entries() {
    let mut board = NOTICE_BOARD.lock().unwrap();
    // let mut for_removal: Vec<usize> = Vec::with_capacity(board.len());

    // for i in 0..board.len() {
    //     if match board[i].should_decay.duration_since(SystemTime::now()) {
    //         Err(..) => true,
    //         Ok(v) => v.is_zero(),
    //     } {
    //         for_removal.push(i);
    //     }
    // }

    // for_removal.sort();
    // for idx in for_removal {
    //     if idx < board.len() {
    //         board.remove(idx);
    //     }
    // }

    let mut i = 0;
    while i < board.len() {
        if match board[i].should_decay.duration_since(SystemTime::now()) {
            Err(..) => true,
            Ok(v) => v.is_zero(),
        } {
            board.remove(i);
        } else {
            i += 1;
        }
    }
}

pub const ENTRY_SIZE: i32 = 30;

pub fn render_entries(renderer: &mut RaylibDrawHandle, h: i32, full_screen_height: i32) {
    let board = NOTICE_BOARD.lock().unwrap();

    for i in 0..board.len().min((h / (ENTRY_SIZE + 5)).max(0) as usize) {
        board[i].contents.render(
            10,
            full_screen_height - i as i32 * (ENTRY_SIZE + 5) - ENTRY_SIZE - 10,
            renderer,
        );
    }
}

pub fn reset() {
    NOTICE_BOARD.lock().unwrap().clear();
}
