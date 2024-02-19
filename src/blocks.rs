use crate::{
    identifier::{GlobalString, Identifier},
    inventory::Inventory,
    items::{get_item_by_id, register_block_item, ItemCoal, COAL_IDENTIFIER},
    notice_board::{self, NoticeboardEntryRenderable},
    scheduler::{schedule_task, Task},
    screens::{ContainerInventoryScreen, CurrentScreen},
    world::ChunkBlockMetadata,
    GameConfig,
};
use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
};

lazy_static! {
    pub static ref BLOCK_EMPTY: Identifier = Identifier::from(("placeholder_name_2", "empty"));
    pub static ref BLOCK_RESOURCE_NODE_BLUE: Identifier =
        Identifier::from(("placeholder_name_2", "resource_node_blue"));
    pub static ref BLOCK_RESOURCE_NODE_GREEN: Identifier =
        Identifier::from(("placeholder_name_2", "resource_node_green"));
    pub static ref BLOCK_RESOURCE_NODE_BROWN: Identifier =
        Identifier::from(("placeholder_name_2", "resource_node_brown"));
    pub static ref BLOCK_STORAGE_CONTAINER: Identifier =
        Identifier::from(("placeholder_name_2", "storage_container"));
    pub static ref EMPTY_NAME: GlobalString = GlobalString::from("ENAMENOTSET");
    pub static ref COAL_NODE_NAME: GlobalString = GlobalString::from("Coal Node");
    pub static ref CONTAINER_NAME: GlobalString = GlobalString::from("Storage Container");
}

pub trait Block: Send + Sync {
    fn clone_block(&self) -> Box<dyn Block>;
    fn init(&mut self, _meta: ChunkBlockMetadata) {}
    fn render(
        &self,
        _d: &mut RaylibDrawHandle,
        _x: i32,
        _y: i32,
        _w: i32,
        _h: i32,
        _meta: ChunkBlockMetadata,
    );
    fn is_building(&self) -> bool {
        false
    }
    fn identifier(&self) -> Identifier;
    fn supports_interaction(&self) -> bool {
        false
    }
    fn interact(&self, _meta: ChunkBlockMetadata, config: &mut GameConfig) {}
    fn name(&self) -> GlobalString {
        *EMPTY_NAME
    }
    fn custom_interact_message(&self) -> Option<String> {
        None
    }
    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        None
    }
    /// schedule your update fn if u want
    fn update(&self, _meta: ChunkBlockMetadata) {}
}

#[derive(Clone)]
pub struct EmptyBlock;
impl Block for EmptyBlock {
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        _meta: ChunkBlockMetadata,
    ) {
        d.draw_rectangle_lines(x, y, w, h, Color::GRAY);
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_EMPTY
    }
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(Self)
    }
}

#[derive(Clone)]
pub struct ResourceNodeGreen;
impl Block for ResourceNodeGreen {
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        sc_x: i32,
        sc_y: i32,
        sc_w: i32,
        sc_h: i32,
        _meta: ChunkBlockMetadata,
    ) {
        d.draw_rectangle(sc_x, sc_y, sc_w, sc_h, Color::GREEN)
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_RESOURCE_NODE_GREEN
    }
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(Self)
    }
}

#[derive(Clone)]
pub struct ResourceNodeBlue;
impl Block for ResourceNodeBlue {
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        sc_x: i32,
        sc_y: i32,
        sc_w: i32,
        sc_h: i32,
        _meta: ChunkBlockMetadata,
    ) {
        d.draw_rectangle(sc_x, sc_y, sc_w, sc_h, Color::BLUE)
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_RESOURCE_NODE_BLUE
    }
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(Self)
    }
}

pub struct ResourceNodeBrown;
impl Block for ResourceNodeBrown {
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(Self)
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_RESOURCE_NODE_BROWN
    }
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        sc_x: i32,
        sc_y: i32,
        sc_w: i32,
        sc_h: i32,
        meta: ChunkBlockMetadata,
    ) {
        d.draw_rectangle(sc_x, sc_y, sc_w, sc_h, Color::BROWN);

        let dir = meta.direction;

        match dir {
            crate::world::Direction::North => {
                d.draw_rectangle(sc_x, sc_y + sc_h - 5, sc_w, 5, Color::BLACK)
            }
            crate::world::Direction::South => d.draw_rectangle(sc_x, sc_y, sc_w, 5, Color::BLACK),
            crate::world::Direction::West => d.draw_rectangle(sc_x, sc_y, 5, sc_h, Color::BLACK),
            crate::world::Direction::East => {
                d.draw_rectangle(sc_x + sc_w - 5, sc_y, 5, sc_h, Color::BLACK)
            }
        }
    }
    fn supports_interaction(&self) -> bool {
        true
    }
    fn interact(&self, _meta: ChunkBlockMetadata, config: &mut GameConfig) {
        let mut item = get_item_by_id(*COAL_IDENTIFIER).unwrap().clone_item();
        item.set_metadata(8);
        config.inventory.try_add_item(item);
    }
    fn custom_interact_message(&self) -> Option<String> {
        Some("Press F to mine Coal Ore".to_string())
    }
    fn name(&self) -> GlobalString {
        *COAL_NODE_NAME
    }
}


pub struct StorageContainer(Inventory);

impl Default for StorageContainer {
    fn default() -> Self {
        Self(Inventory::new(5 * 9, false))
    }
}

impl Block for StorageContainer {
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(Self(self.0.clone()))
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_STORAGE_CONTAINER
    }
    fn name(&self) -> GlobalString {
        *CONTAINER_NAME
    }
    fn interact(&self, meta: ChunkBlockMetadata, config: &mut GameConfig) {
        schedule_task(Task::OpenScreenCentered(Box::new(
            ContainerInventoryScreen::new(meta.position.0, meta.position.1, self.0.size() as u32, self.name()),
        )))
    }
    fn is_building(&self) -> bool {
        true
    }
    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        Some(&mut self.0)
    }
    fn supports_interaction(&self) -> bool {
        true
    }
    fn init(&mut self, _meta: ChunkBlockMetadata) {
        self.0.resize(5 * 9);
    }
    fn render(
            &self,
            d: &mut RaylibDrawHandle,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            _meta: ChunkBlockMetadata,
        ) {
        d.draw_rectangle(x, y, w, h, Color::MAGENTA);
    }
}

pub static mut BLOCKS: Vec<Box<dyn Block>> = Vec::new();

pub fn register_blocks() {
    register_block(Box::new(EmptyBlock));
    register_block(Box::new(ResourceNodeBlue));
    register_block(Box::new(ResourceNodeGreen));
    register_block(Box::new(ResourceNodeBrown));
    register_block(Box::new(StorageContainer::default()));
}

pub fn register_block(block: Box<dyn Block>) {
    unsafe {
        BLOCKS.push(block.clone_block());
        register_block_item(block);
    }
}

pub fn get_block_by_id(id: Identifier) -> Option<&'static Box<dyn Block>> {
    unsafe {
        for blk in &BLOCKS {
            if blk.identifier() == id {
                return Some(blk);
            }
        }
    }
    None
}

pub fn empty_block() -> &'static Box<dyn Block> {
    unsafe { &BLOCKS[0] }
}
