use std::fmt::Debug;

use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
};

use crate::{
    blocks::Block, game::RenderLayer, identifier::{GlobalString, Identifier}, serialization::{Buffer, SerializationError}, world::{ChunkBlockMetadata, Direction}
};

impl Clone for Box<dyn Item> {
    fn clone(&self) -> Self {
        self.clone_item()
    }
}

pub trait Item: Send + Sync {
    fn clone_item(&self) -> Box<dyn Item>;
    fn identifier(&self) -> Identifier;
    fn name(&self) -> GlobalString;
    /// either durability or stack size
    fn metadata(&self) -> u32;
    fn metadata_is_stack_size(&self) -> bool {
        true
    }
    fn description(&self) -> &'static str;
    fn render(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32);
    fn set_metadata(&mut self, new_data: u32);
    fn serialize(&self, vec: &mut Vec<u8>);
    fn try_deserialize(&mut self, buf: &mut Buffer) -> Result<(), SerializationError>;
    fn required_length(&self) -> usize;
}

impl Debug for dyn Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}({:?})[{}]",
            self.name(),
            self.identifier(),
            if self.metadata_is_stack_size() {
                self.metadata()
            } else {
                1
            }
        ))
    }
}

lazy_static! {
    pub static ref COAL_IDENTIFIER: Identifier = Identifier::from(("placeholder_name_2", "coal"));
    pub static ref COAL_NAME: GlobalString = GlobalString::from("Coal");
}

macro_rules! empty_serializable {
    () => {
        fn serialize(&self, _: &mut Vec<u8>) {}
        fn try_deserialize(&mut self, _: &mut Buffer) -> Result<(), SerializationError> {Ok(())}
        fn required_length(&self) -> usize {0}
    };
}

pub struct ItemCoal(u32);

impl Item for ItemCoal {
    empty_serializable!();
    fn description(&self) -> &'static str {
        "Coal is most commonly used as a fuel for generators"
    }
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
    empty_serializable!();

    fn description(&self) -> &'static str {
        self.1.description()
    }
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
            RenderLayer::default_preview(),
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
        for item in &ITEMS {
            if item.identifier() == id {
                return Some(item);
            }
        }
    }
    None
}

pub fn empty_item() -> &'static Box<dyn Item> {
    unsafe { &ITEMS[0] }
}
