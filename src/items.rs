use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
};

use crate::{
    blocks::{get_block_by_id, Block},
    identifier::{GlobalString, Identifier},
    world::{ChunkBlockMetadata, Direction},
};

pub trait Item: Send + Sync {
    fn clone_item(&self) -> Box<dyn Item>;
    fn identifier(&self) -> Identifier;
    fn name(&self) -> GlobalString;
    /// either durability or stack size
    fn metadata(&self) -> u32;
    fn metadata_is_stack_size(&self) -> bool {
        true
    }
    fn render(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32);
    fn set_metadata(&mut self, new_data: u32);
}

lazy_static! {
    pub static ref COAL_IDENTIFIER: Identifier = Identifier::from(("placeholder_name_2", "coal"));
    pub static ref COAL_NAME: GlobalString = GlobalString::from("Coal");
}

pub struct ItemCoal(u32);

impl Item for ItemCoal {
    fn clone_item(&self) -> Box<dyn Item> {
        Box::new(Self(self.0))
    }
    fn identifier(&self) -> Identifier {
        *COAL_IDENTIFIER
    }
    fn name(&self) -> GlobalString {
        *COAL_NAME
    }
    fn metadata(&self) -> u32 {
        self.0
    }
    fn render(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32) {
        renderer.draw_ellipse(
            x + w / 2,
            y + h / 2,
            w as f32 / 3.0,
            h as f32 / 2.0,
            Color::BLACK,
        );
    }
    fn set_metadata(&mut self, new_data: u32) {
        self.0 = new_data
    }
}

pub struct BlockItem(u32, Box<dyn Block>);

impl Item for BlockItem {
    fn clone_item(&self) -> Box<dyn Item> {
        Box::new(Self(self.0, self.1.clone_block()))
    }
    fn identifier(&self) -> Identifier {
        self.1.identifier()
    }
    fn name(&self) -> GlobalString {
        self.1.name()
    }
    fn metadata(&self) -> u32 {
        self.0
    }
    fn render(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32) {
        self.1.render(
            renderer,
            x,
            y,
            w,
            h,
            ChunkBlockMetadata::from(Direction::North),
        )
    }
    fn set_metadata(&mut self, new_data: u32) {
        self.0 = new_data
    }
}

pub static mut ITEMS: Vec<Box<dyn Item>> = Vec::new();

pub fn register_items() {
    unsafe {
        ITEMS.push(Box::new(ItemCoal(1)));
    }
}

pub fn register_item(item: Box<dyn Item>) {
    unsafe {
        ITEMS.push(item);
    }
}

pub fn register_block_item(block: Box<dyn Block>) {
    unsafe {
        ITEMS.push(Box::new(BlockItem(0, block)));
    }
}

pub fn get_item_by_id(id: Identifier) -> Option<&'static Box<dyn Item>> {
    unsafe {
        for blk in &ITEMS {
            if blk.identifier() == id {
                return Some(blk);
            }
        }
    }
    None
}

pub fn empty_item() -> &'static Box<dyn Item> {
    unsafe { &ITEMS[0] }
}
