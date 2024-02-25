use std::time::Instant;

use lazy_static::lazy_static;
use raylib::{drawing::RaylibDrawHandle, RaylibHandle, RaylibThread};

use crate::{
    asset,
    assets::{load_animated_texture, AnimatedTexture2D, Frame},
    block_impl_details_with_timer,
    identifier::{GlobalString, Identifier},
    initialized_data::InitializedData,
    inventory::Inventory,
    items::Item,
    reset_timer,
    scheduler::{schedule_task, Task},
    simple_single_item_direction_serializable, step_size,
    world::{ChunkBlockMetadata, Direction, Vec2i, World},
    GameConfig, game::RenderLayer,
};

use super::Block;

lazy_static! {
    pub static ref CONVEYOR_NAME: GlobalString = GlobalString::from("Conveyor Belt Tier 1");
    pub static ref BLOCK_CONVEYOR: Identifier =
        Identifier::from(("placeholder_name_2", "conveyor_mk1"));
}

block_impl_details_with_timer!(ConveyorBlock, 1000, Inventory, Direction);
impl Default for ConveyorBlock {
    fn default() -> Self {
        Self(
            Instant::now(),
            Inventory::new(1, false),
            Direction::default(),
        )
    }
}

impl Block for ConveyorBlock {
    simple_single_item_direction_serializable!(1, 2);

    fn description(&self) -> &'static str {
        "Moves 60 items per minute"
    }

    fn interact(&mut self, _: ChunkBlockMetadata, config: &mut GameConfig) {
        match self.1.take_item(0) {
            None => {}
            Some(item) => {
                if item.metadata() < 1 {
                    return;
                }
                if let Some(item) = config.inventory.try_add_item(item) {
                    self.1.get_item_mut(0).replace(item);
                }
            }
        }
    }

    fn supports_interaction(&self) -> bool {
        self.1.get_item(0).is_some()
    }

    fn custom_interact_message(&self) -> Option<String> {
        self.1
            .get_item(0)
            .as_ref()
            .map(|item| format!("Grab {} from {}", item.name(), self.name()))
    }

    fn identifier(&self) -> Identifier {
        *BLOCK_CONVEYOR
    }
    fn name(&self) -> GlobalString {
        *CONVEYOR_NAME
    }
    fn destroy_items(&self) -> Vec<Box<dyn Item>> {
        self.1.destroy_items()
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
            CONVEYOR_ANIMATION.draw_resized_rotated(d, x, y, w, h, meta.direction);
        } else if layer == RenderLayer::OverlayItems {
            if let Some(item) = &self.1.get_item(0) {
                let lerp_val = self.duration_lerp_value();
                let step_size = step_size!(self.2, w, h);
                if lerp_val < 0.5 {
                    let lerp = (lerp_val * step_size as f32).floor() as i32;
                    let mut vec = Vec2i::new(x + 5, y + 5);
                    vec.add_directional_assign(&self.2, -step_size / 2);
                    vec.add_directional_assign(&self.2, lerp);
                    item.render(d, vec.x, vec.y, w - 10, h - 10);
                } else {
                    let lerp_val = lerp_val - 0.5;
                    let lerp = (lerp_val * step_size as f32).floor() as i32;
                    let mut vec = Vec2i::new(x + 5, y + 5);
                    vec.add_directional_assign(&meta.direction, lerp);
                    item.render(d, vec.x, vec.y, w - 10, h - 10);
                }
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
        side != meta.direction
    }
    fn push(
        &mut self,
        side: Direction,
        mut item: Box<dyn Item>,
        meta: ChunkBlockMetadata,
    ) -> Option<Box<dyn Item>> {
        if side == meta.direction {
            return Some(item);
        }
        let slot = self.1.get_item_mut(0);
        if slot.is_some() {
            return Some(item);
        }
        self.2 = side.opposite();
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
        self.1.update();
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

        let push_dir = meta.direction.opposite();
        if pushto.has_capability_push(push_dir, pushto_meta)
            && pushto.can_push(push_dir, &item, meta)
        {
            item = pushto.push(push_dir, item, pushto_meta)?;
        }
        world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .add_item(item, 0);

        Some(())
    }

    pub fn load_block_files(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), String> {
        CONVEYOR_ANIMATION.init(load_animated_texture(
            rl,
            thread,
            asset!("conveyor.png"),
            Frame::multiple(50, 5),
            64,
            64,
            None,
        )?);

        Ok(())
    }
}

static CONVEYOR_ANIMATION: InitializedData<&'static AnimatedTexture2D> = InitializedData::new();
