use std::time::Instant;

use crate::{
    as_any::AsAny,
    derive_as_any, downcast_for,
    identifier::{GlobalString, Identifier},
    inventory::Inventory,
    items::{get_item_by_id, register_block_item, Item, COAL_IDENTIFIER},
    scheduler::{schedule_task, Task},
    screens::ContainerInventoryScreen,
    world::{ChunkBlockMetadata, Direction, Vec2i, World},
    GameConfig, RenderLayer, RENDER_LAYERS,
};
use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Vector2,
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
    pub static ref BLOCK_EXTRACTOR: Identifier =
        Identifier::from(("placeholder_name_2", "extractor"));
    pub static ref BLOCK_CONVEYOR: Identifier =
        Identifier::from(("placeholder_name_2", "conveyor_mk1"));
    pub static ref EMPTY_NAME: GlobalString = GlobalString::from("ENAMENOTSET");
    pub static ref COAL_NODE_NAME: GlobalString = GlobalString::from("Coal Node");
    pub static ref CONTAINER_NAME: GlobalString = GlobalString::from("Storage Container");
    pub static ref EXTRACTOR_NAME: GlobalString = GlobalString::from("Extractor");
    pub static ref CONVEYOR_NAME: GlobalString = GlobalString::from("Conveyor Belt Tier 1");
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Self {
        self.clone_block()
    }
}

pub trait BlockImplDetails: Send + Sync + AsAny {
    fn clone_block(&self) -> Box<dyn Block>;
}

macro_rules! block_impl_details {
    ($name: ident) => {
        #[derive(Clone, Default)]
        pub struct $name;
        impl BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                Box::new(self.clone())
            }
        }
        derive_as_any!($name);
    };
    ($name: ident, $clone_fn: block) => {
        #[derive(Default)]
        pub struct $name;
        impl BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };
    ($name: ident, $($y:ty),*) => {
        #[derive(Clone)]
        pub struct $name($($y),*);
        impl BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                Box::new(self.clone())
            }
        }
        derive_as_any!($name);
    };
    ($name: ident, $clone_fn: expr, $($y:ty),*) => {
        pub struct $name($($y),*);
        impl BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };
}

macro_rules! block_impl_details_with_timer {
    ($name: ident, $duration: expr) => {
        block_impl_details!($name, Instant);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $clone_fn: block) => {
        block_impl_details!($name, $clone_fn, Instant);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $($y:ty),*) => {
        block_impl_details!($name, Instant, $($y),*);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $clone_fn: expr, $($y:ty),*) => {
        block_impl_details!($name, {$clone_fn}, Instant, $($y),*);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    (__ $name: ident, $duration: expr) => {
        impl $name {
            fn can_do_work(&self) -> bool {
                if Instant::now().saturating_duration_since(self.0).as_millis() >= ($duration as u128) {
                    true
                } else {
                    false
                }
            }

            #[allow(dead_code)]
            fn duration_lerp_value(&self) -> f32 {
                ((Instant::now().saturating_duration_since(self.0).as_millis().min($duration as u128)) as f32 / $duration as f32).min(1.0)
            }
        }
    };
}

macro_rules! reset_timer {
    ($self: expr) => {
        $self.0 = Instant::now();
    };
}

macro_rules! register_blocks {
    ($($block: ty),*) => {
        $(
            register_block(Box::new(<$block>::default()));
        )*
    };
}

pub trait Block: BlockImplDetails {
    #[allow(unused_variables)]
    fn init(&mut self, meta: ChunkBlockMetadata) {}
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
    fn is_building(&self) -> bool {
        false
    }
    fn identifier(&self) -> Identifier;
    fn supports_interaction(&self) -> bool {
        false
    }
    #[allow(unused_variables)]
    fn interact(&self, meta: ChunkBlockMetadata, config: &mut GameConfig) {}
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
}

// #[derive(Clone)]
// pub struct EmptyBlock;
// derive_as_any!(EmptyBlock);
block_impl_details!(EmptyBlock);
impl Block for EmptyBlock {
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
    fn identifier(&self) -> Identifier {
        *BLOCK_EMPTY
    }
}

// #[derive(Clone)]
// pub struct ResourceNodeGreen;
block_impl_details!(ResourceNodeGreen);
impl Block for ResourceNodeGreen {
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        sc_x: i32,
        sc_y: i32,
        sc_w: i32,
        sc_h: i32,
        _meta: ChunkBlockMetadata,
        _layer: RenderLayer,
    ) {
        d.draw_rectangle(sc_x, sc_y, sc_w, sc_h, Color::GREEN)
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_RESOURCE_NODE_GREEN
    }
}

// #[derive(Clone)]
// pub struct ResourceNodeBlue;
block_impl_details!(ResourceNodeBlue);
impl Block for ResourceNodeBlue {
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        sc_x: i32,
        sc_y: i32,
        sc_w: i32,
        sc_h: i32,
        _meta: ChunkBlockMetadata,
        _layer: RenderLayer,
    ) {
        d.draw_rectangle(sc_x, sc_y, sc_w, sc_h, Color::BLUE)
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_RESOURCE_NODE_BLUE
    }
}

// pub struct ResourceNodeBrown;
block_impl_details!(ResourceNodeBrown);
impl Block for ResourceNodeBrown {
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
    fn interact(&self, _meta: ChunkBlockMetadata, config: &mut GameConfig) {
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
}

block_impl_details!(StorageContainer, Inventory);

impl Default for StorageContainer {
    fn default() -> Self {
        Self(Inventory::new(5 * 9, false))
    }
}

impl Block for StorageContainer {
    fn identifier(&self) -> Identifier {
        *BLOCK_STORAGE_CONTAINER
    }
    fn name(&self) -> GlobalString {
        *CONTAINER_NAME
    }
    fn interact(&self, meta: ChunkBlockMetadata, _: &mut GameConfig) {
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

block_impl_details_with_timer!(ExtractorBlock, 250, Inventory);
impl Default for ExtractorBlock {
    fn default() -> Self {
        Self(Instant::now(), Inventory::new(1, false))
    }
}
impl Block for ExtractorBlock {
    fn identifier(&self) -> Identifier {
        *BLOCK_EXTRACTOR
    }
    fn name(&self) -> GlobalString {
        *EXTRACTOR_NAME
    }
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: ChunkBlockMetadata,
        layer: RenderLayer,
    ) {
        if layer == RenderLayer::Block {
            d.draw_rectangle(x, y, w, h, Color::ORANGE);
            let (vec_1, vec_2, vec_3) = match meta.direction {
                Direction::North => (
                    Vector2::new((x + 5) as f32, (y + h) as f32),
                    Vector2::new((x + w - 5) as f32, (y + h) as f32),
                    Vector2::new((x + w / 2) as f32, (y + h - w / 2) as f32),
                ),
                Direction::South => (
                    Vector2::new((x + w - 5) as f32, y as f32),
                    Vector2::new((x + 5) as f32, y as f32),
                    Vector2::new((x + w / 2) as f32, (y + w / 2) as f32),
                ),
                Direction::East => (
                    Vector2::new((x + w) as f32, (y + h - 5) as f32),
                    Vector2::new((x + w) as f32, (y + 5) as f32),
                    Vector2::new((x + h / 2) as f32, (y + h / 2) as f32),
                ),
                Direction::West => (
                    Vector2::new(x as f32, (y + 5) as f32),
                    Vector2::new(x as f32, (y + h - 5) as f32),
                    Vector2::new((x + w - h / 2) as f32, (y + h / 2) as f32),
                ),
            };
            d.draw_triangle(vec_1, vec_2, vec_3, Color::BLUE);
        } else if layer == RenderLayer::OverlayItems {
            if let Some(item) = &self.1.get_item(0) {
                let lerp = (self.duration_lerp_value() * w as f32).floor() as i32 - w / 2;
                let mut vec = Vec2i::new(x + 5, y + 5);
                vec.add_directional_assign(&meta.direction, lerp);
                item.render(d, vec.x, vec.y, w - 10, h - 10);
            }
        }
    }

    fn init(&mut self, _: ChunkBlockMetadata) {
        self.1.resize(1);
    }
    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        if !self.can_do_work() {
            return None;
        }
        Some(&mut self.1)
    }
    fn update(&mut self, meta: ChunkBlockMetadata) {
        schedule_task(Task::WorldUpdateBlock(&Self::update, meta));
    }
}

impl ExtractorBlock {
    fn update_pull(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let block_pull_pos = meta.position.add_directional(&meta.direction, -1);
        if let Some((me, _)) = world.get_block_at_mut(meta.position.x, meta.position.y) {
            let inv = me.get_inventory_capability()?;
            if inv.get_item(0).is_some() {
                return Some(());
            }
        }
        let item = world
            .get_block_at_mut(block_pull_pos.x, block_pull_pos.y)
            .and_then(|(blk, blk_meta)| {
                if blk.can_pull(meta.direction.opposite(), blk_meta) {
                    blk.pull(meta.direction.opposite(), blk_meta, 1)
                } else {
                    None
                }
            })?;
        let blk = world.get_block_at_mut(meta.position.x, meta.position.y)?.0;
        let blk = downcast_mut::<Self>(&mut **blk)?;
        reset_timer!(blk);
        *blk.1.get_item_mut(0) = Some(item);

        Some(())
    }

    fn update_push(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let block_push_pos = meta.position.add_directional(&meta.direction, 1);
        let mut item = world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .take_item(0)?;

        if let Some((blk, push_meta)) = world.get_block_at_mut(block_push_pos.x, block_push_pos.y) {
            item = blk.push(meta.direction, item, push_meta)?;
        }

        world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .add_item(item, 0);

        Some(())
    }

    fn update(meta: ChunkBlockMetadata, world: &mut World) {
        Self::update_pull(meta, world);
        Self::update_push(meta, world);
    }
}

block_impl_details_with_timer!(ConveyorBlock, 250, Inventory);
impl Default for ConveyorBlock {
    fn default() -> Self {
        Self(Instant::now(), Inventory::new(1, false))
    }
}
impl Block for ConveyorBlock {
    fn identifier(&self) -> Identifier {
        *BLOCK_CONVEYOR
    }
    fn name(&self) -> GlobalString {
        *CONVEYOR_NAME
    }
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: ChunkBlockMetadata,
        layer: RenderLayer,
    ) {
        if layer == RenderLayer::Block {
            d.draw_rectangle(x, y, w, h, Color::GRAY);
            let (vec_1, vec_2, vec_3) = match meta.direction {
                Direction::North => (
                    Vector2::new((x + 5) as f32, (y + h) as f32),
                    Vector2::new((x + w - 5) as f32, (y + h) as f32),
                    Vector2::new((x + w / 2) as f32, (y + h - w / 2) as f32),
                ),
                Direction::South => (
                    Vector2::new((x + w - 5) as f32, y as f32),
                    Vector2::new((x + 5) as f32, y as f32),
                    Vector2::new((x + w / 2) as f32, (y + w / 2) as f32),
                ),
                Direction::East => (
                    Vector2::new((x + w) as f32, (y + h - 5) as f32),
                    Vector2::new((x + w) as f32, (y + 5) as f32),
                    Vector2::new((x + h / 2) as f32, (y + h / 2) as f32),
                ),
                Direction::West => (
                    Vector2::new(x as f32, (y + 5) as f32),
                    Vector2::new(x as f32, (y + h - 5) as f32),
                    Vector2::new((x + w - h / 2) as f32, (y + h / 2) as f32),
                ),
            };
            d.draw_triangle(vec_1, vec_2, vec_3, Color::BLUE);
        } else if layer == RenderLayer::OverlayItems {
            if let Some(item) = &self.1.get_item(0) {
                let lerp = (self.duration_lerp_value() * w as f32).floor() as i32 - w / 2;
                let mut vec = Vec2i::new(x + 5, y + 5);
                vec.add_directional_assign(&meta.direction, lerp);
                item.render(d, vec.x, vec.y, w - 10, h - 10);
            }
        }
    }

    fn init(&mut self, _: ChunkBlockMetadata) {
        self.1.resize(1);
    }
    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        if !self.can_do_work() {
            return None;
        }
        Some(&mut self.1)
    }
    fn can_push(&self, side: Direction, _: &Box<dyn Item>, meta: ChunkBlockMetadata) -> bool {
        self.1.get_item(0).is_none() && self.has_capability_push(side, meta)
    }
    fn has_capability_push(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        side != meta.direction.opposite()
    }
    fn push(
        &mut self,
        side: Direction,
        mut item: Box<dyn Item>,
        meta: ChunkBlockMetadata,
    ) -> Option<Box<dyn Item>> {
        if side == meta.direction.opposite() {
            return Some(item);
        }
        let slot = self.1.get_item_mut(0);
        if slot.is_some() {
            return Some(item);
        }
        reset_timer!(self);
        if item.metadata_is_stack_size() && item.metadata() > 1 {
            let mut itm = item.clone_item();
            itm.set_metadata(1);
            *slot = Some(itm);
            item.set_metadata(item.metadata() - 1);
            Some(item)
        } else {
            *slot = Some(item);
            None
        }
    }
    fn update(&mut self, meta: ChunkBlockMetadata) {
        if !self.can_do_work() {
            return;
        }
        schedule_task(Task::WorldUpdateBlock(
            &|a, b| {
                Self::update(a, b);
            },
            meta,
        ));
    }
}

impl ConveyorBlock {
    pub fn update(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let mut item = world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .take_item(0)?;
        let pushto_pos = meta.position.add_directional(&meta.direction, 1);
        let (pushto, pushto_meta) = world.get_block_at_mut(pushto_pos.x, pushto_pos.y)?;

        if pushto.has_capability_push(meta.direction, pushto_meta)
            && pushto.can_push(meta.direction, &item, meta)
        {
            item = pushto.push(meta.direction, item, pushto_meta)?;
        }
        world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .add_item(item, 0);

        Some(())
    }
}

pub static mut BLOCKS: Vec<Box<dyn Block>> = Vec::new();

pub fn register_blocks() {
    register_blocks!(
        EmptyBlock,
        ResourceNodeBlue,
        ResourceNodeGreen,
        ResourceNodeBrown,
        StorageContainer,
        ExtractorBlock,
        ConveyorBlock
    );
    // register_block(Box::new(EmptyBlock));
    // register_block(Box::new(ResourceNodeBlue));
    // register_block(Box::new(ResourceNodeGreen));
    // register_block(Box::new(ResourceNodeBrown));
    // register_block(Box::new(StorageContainer::default()));
    // register_block(Box::new(ExtractorBlock::default()));
    // register_block(Box::new(ConveyorBlock::default()));
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

downcast_for!(Block);
