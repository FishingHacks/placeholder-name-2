pub mod conveyor;
pub mod extractor;
mod macros;
pub mod splitter;

use crate::{
    as_any::AsAny,
    block_impl_details,
    blocks::{conveyor::ConveyorBlock, extractor::ExtractorBlock, splitter::ConveyorSplitter},
    derive_as_any, downcast_for, empty_serializable,
    identifier::{GlobalString, Identifier},
    inventory::Inventory,
    items::{get_item_by_id, register_block_item, Item, COAL_IDENTIFIER},
    register_blocks as m_register_blocks,
    scheduler::{schedule_task, Task},
    screens::ContainerInventoryScreen,
    serialization::{Buffer, Deserialize, SerializationError, Serialize},
    world::{ChunkBlockMetadata, Direction},
    GameConfig, game::{RenderLayer, RENDER_LAYERS},
};
use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    RaylibHandle, RaylibThread,
};

lazy_static! {
    pub static ref BLOCK_EMPTY: Identifier = Identifier::from(("placeholder_name_2", "empty"));
    pub static ref BLOCK_RESOURCE_NODE_BROWN: Identifier =
        Identifier::from(("placeholder_name_2", "resource_node_brown"));
    pub static ref BLOCK_STORAGE_CONTAINER: Identifier =
        Identifier::from(("placeholder_name_2", "storage_container"));
    pub static ref EMPTY_NAME: GlobalString = GlobalString::from("ENAMENOTSET");
    pub static ref COAL_NODE_NAME: GlobalString = GlobalString::from("Coal Node");
    pub static ref CONTAINER_NAME: GlobalString = GlobalString::from("Storage Container");
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Self {
        self.clone_block()
    }
}

pub trait BlockImplDetails: Send + Sync + AsAny {
    fn clone_block(&self) -> Box<dyn Block>;
}

pub trait Block: BlockImplDetails {
    fn is_none(&self) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn init(&mut self, meta: ChunkBlockMetadata) {}
    fn description(&self) -> &'static str;
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: ChunkBlockMetadata,
        render_layer: RenderLayer,
    );
    fn render_all(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: ChunkBlockMetadata,
    ) {
        for l in &RENDER_LAYERS {
            self.render(d, x, y, w, h, meta, *l);
        }
    }
    fn destroy_items(&self) -> Vec<Box<dyn Item>> {
        Vec::new()
    }
    fn is_building(&self) -> bool {
        false
    }
    fn identifier(&self) -> Identifier;
    fn supports_interaction(&self) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn interact(&mut self, meta: ChunkBlockMetadata, config: &mut GameConfig) {}
    fn name(&self) -> GlobalString {
        *EMPTY_NAME
    }
    fn custom_interact_message(&self) -> Option<String> {
        None
    }
    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        None
    }
    #[allow(unused_variables)]
    fn has_capability_push(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn has_capability_pull(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn can_push(&self, _side: Direction, item: &Box<dyn Item>, meta: ChunkBlockMetadata) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn push(
        &mut self,
        side: Direction,
        item: Box<dyn Item>,
        meta: ChunkBlockMetadata,
    ) -> Option<Box<dyn Item>> {
        Some(item)
    }
    #[allow(unused_variables)]
    fn can_pull(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn pull(
        &mut self,
        side: Direction,
        meta: ChunkBlockMetadata,
        num_items: u32,
    ) -> Option<Box<dyn Item>> {
        None
    }

    #[allow(unused_variables)]
    /// schedule your update fn if u want
    fn update(&mut self, meta: ChunkBlockMetadata) {}
    fn serialize(&self, buf: &mut Vec<u8>);
    fn try_deserialize(&mut self, buf: &mut Buffer) -> Result<(), SerializationError>;
    fn required_length(&self) -> usize;
}

block_impl_details!(default EmptyBlock);
impl Block for EmptyBlock {
    empty_serializable!();
    fn is_none(&self) -> bool {
        true
    }
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        _meta: ChunkBlockMetadata,
        _layer: RenderLayer,
    ) {
        d.draw_rectangle_lines(x, y, w, h, Color::GRAY);
    }
    fn description(&self) -> &'static str {
        "*scared* wh- why can u see me :tbhcry:"
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_EMPTY
    }
}

block_impl_details!(default ResourceNodeBrown);
impl Block for ResourceNodeBrown {
    empty_serializable!();
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
        _layer: RenderLayer,
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
    fn interact(&mut self, _meta: ChunkBlockMetadata, config: &mut GameConfig) {
        let mut item = get_item_by_id(*COAL_IDENTIFIER).unwrap().clone_item();
        item.set_metadata(8);
        if config.inventory.try_add_item(item).is_some() {
            println!("Could not add item");
        }
    }
    fn custom_interact_message(&self) -> Option<String> {
        Some("Press F to mine Coal Ore".to_string())
    }
    fn name(&self) -> GlobalString {
        *COAL_NODE_NAME
    }
    fn has_capability_pull(&self, _: Direction, _: ChunkBlockMetadata) -> bool {
        true
    }
    fn can_pull(&self, _: Direction, _: ChunkBlockMetadata) -> bool {
        true
    }
    fn pull(&mut self, _: Direction, _: ChunkBlockMetadata, _: u32) -> Option<Box<dyn Item>> {
        let mut item = get_item_by_id(*COAL_IDENTIFIER)?.clone_item();
        item.set_metadata(1);
        Some(item)
    }
    fn description(&self) -> &'static str {
        "An Ore Node to extract coal from"
    }
}

block_impl_details!(StorageContainer, Inventory);

impl Default for StorageContainer {
    fn default() -> Self {
        Self(Inventory::new(5 * 9, false))
    }
}

impl Block for StorageContainer {
    fn destroy_items(&self) -> Vec<Box<dyn Item>> {
        self.0.destroy_items()
    }

    fn description(&self) -> &'static str {
        "A 5x9 Container able to hold a total of 11475 items"
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        self.0.serialize(buf);
    }
    fn try_deserialize(&mut self, buf: &mut Buffer) -> Result<(), SerializationError> {
        self.0 = Inventory::try_deserialize(buf)?;
        Ok(())
    }
    fn required_length(&self) -> usize {
        self.0.required_length()
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_STORAGE_CONTAINER
    }
    fn name(&self) -> GlobalString {
        *CONTAINER_NAME
    }
    fn interact(&mut self, meta: ChunkBlockMetadata, _: &mut GameConfig) {
        schedule_task(Task::OpenScreenCentered(Box::new(
            ContainerInventoryScreen::new(
                meta.position.x,
                meta.position.y,
                self.0.size() as u32,
                self.name(),
            ),
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
        _layer: RenderLayer,
    ) {
        d.draw_rectangle(x, y, w, h, Color::MAGENTA);
    }
    fn has_capability_push(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        side == meta.direction || side + Direction::South == meta.direction
    }
    fn has_capability_pull(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        side == meta.direction || side + Direction::South == meta.direction
    }
    fn can_pull(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        self.has_capability_pull(side, meta) && self.0.can_pull()
    }
    fn can_push(&self, side: Direction, item: &Box<dyn Item>, meta: ChunkBlockMetadata) -> bool {
        self.has_capability_push(side, meta) && self.0.can_push(item)
    }
    fn push(
        &mut self,
        _side: Direction,
        item: Box<dyn Item>,
        _meta: ChunkBlockMetadata,
    ) -> Option<Box<dyn Item>> {
        self.0.try_add_item(item)
    }
    fn pull(
        &mut self,
        _side: Direction,
        _meta: ChunkBlockMetadata,
        num_items: u32,
    ) -> Option<Box<dyn Item>> {
        self.0.try_pull(num_items)
    }
}

pub static mut BLOCKS: Vec<Box<dyn Block>> = Vec::new();

pub fn register_blocks() {
    m_register_blocks!(
        EmptyBlock,
        ResourceNodeBrown,
        StorageContainer,
        ExtractorBlock,
        ConveyorBlock,
        ConveyorSplitter
    );
}

pub fn register_block(block: Box<dyn Block>) {
    unsafe {
        BLOCKS.push(block.clone_block());
        register_block_item(block);
    }
}

pub fn load_block_files(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), String> {
    ConveyorBlock::load_block_files(rl, thread)?;

    Ok(())
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

downcast_for!(Block);
