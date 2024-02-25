use std::time::Instant;

use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Vector2,
};

use crate::{
    block_impl_details_with_timer,
    identifier::{GlobalString, Identifier},
    inventory::Inventory,
    items::Item,
    reset_timer,
    scheduler::{schedule_task, Task},
    simple_single_item_serializable, step_size,
    world::{ChunkBlockMetadata, Direction, Vec2i, World},
    game::RenderLayer,
};

use super::{downcast, downcast_mut, Block};

lazy_static! {
    pub static ref CONVEYOR_SPLITTER: GlobalString = GlobalString::from("Conveyor Splitter");
    pub static ref BLOCK_CONVEYOR_SPLITTER: Identifier =
        Identifier::from(("placeholder_name_2", "conveyor_splitter"));
}

block_impl_details_with_timer!(ConveyorSplitter, 200, Inventory, usize, Option<Direction>);
impl Default for ConveyorSplitter {
    fn default() -> Self {
        Self(Instant::now(), Inventory::new(1, false), 0, None)
    }
}
impl Block for ConveyorSplitter {
    simple_single_item_serializable!(1);

    fn description(&self) -> &'static str {
        "Splits incoming items evenly between all 3 outputs using round robin at a rate of 5 per second"
    }

    fn identifier(&self) -> Identifier {
        *BLOCK_CONVEYOR_SPLITTER
    }
    fn init(&mut self, _: ChunkBlockMetadata) {
        self.1.resize(1);
    }
    fn name(&self) -> GlobalString {
        *CONVEYOR_SPLITTER
    }
    fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: ChunkBlockMetadata,
        render_layer: RenderLayer,
    ) {
        if render_layer == RenderLayer::Block {
            d.draw_rectangle(x, y, w, h, Color::GOLD);
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
            d.draw_triangle(vec_1, vec_2, vec_3, Color::GREEN);
        } else if render_layer == RenderLayer::OverlayItems {
            if let Some(item) = &self.1.get_item(0) {
                let lerp = self.duration_lerp_value();

                if lerp < 0.5 {
                    let from_dir = meta.direction;
                    let step_size = step_size!(from_dir, w, h);
                    let lerp = (lerp * step_size as f32).floor() as i32;
                    let mut vec = Vec2i::new(x + 5, y + 5);
                    vec.add_directional_assign(&from_dir, -step_size / 2);
                    vec.add_directional_assign(&from_dir, lerp);
                    item.render(d, vec.x, vec.y, w - 10, h - 10);
                } else {
                    let lerp = lerp - 0.5;
                    if let Some(determined_direction) = self.3 {
                        let step_size = step_size!(determined_direction, w, h);

                        let lerp = (lerp * step_size as f32).floor() as i32;
                        let mut vec = Vec2i::new(x + 5, y + 5);
                        vec.add_directional_assign(&determined_direction, lerp);

                        item.render(d, vec.x, vec.y, w - 10, h - 10);
                    } else {
                        item.render(d, x + 5, y + 5, w - 10, h - 10);
                    }
                }
            }
        }
    }

    fn can_push(&self, side: Direction, _: &Box<dyn Item>, meta: ChunkBlockMetadata) -> bool {
        self.1.get_item(0).is_none() && self.has_capability_push(side, meta)
    }

    fn has_capability_push(&self, side: Direction, meta: ChunkBlockMetadata) -> bool {
        side == meta.direction.opposite()
    }

    fn push(
        &mut self,
        side: Direction,
        mut item: Box<dyn Item>,
        meta: ChunkBlockMetadata,
    ) -> Option<Box<dyn Item>> {
        if !self.can_push(side, &item, meta) {
            return Some(item);
        }
        let slot = self.1.get_item_mut(0);
        if slot.is_some() {
            return Some(item);
        }
        reset_timer!(self);
        if item.metadata_is_stack_size() && item.metadata() > 1 {
            let remaining = item.metadata() - 1;
            item.set_metadata(1);
            slot.replace(item.clone_item());
            item.set_metadata(remaining);
            Some(item)
        } else {
            slot.replace(item.clone_item());
            None
        }
    }

    fn get_inventory_capability<'a>(&'a mut self) -> Option<&'a mut Inventory> {
        Some(&mut self.1)
    }

    fn update(&mut self, meta: ChunkBlockMetadata) {
        if self.can_do_work() && self.3.is_some() {
            schedule_task(Task::WorldUpdateBlock(
                &|a, b| {
                    Self::update(a, b);
                },
                meta,
            ));
        } else if self.3.is_none() {
            schedule_task(Task::WorldUpdateBlock(
                &|a, b| {
                    Self::determine_direction(a, b);
                },
                meta,
            ));
        }
    }

    fn is_building(&self) -> bool {
        true
    }

    fn destroy_items(&self) -> Vec<Box<dyn Item>> {
        self.1.destroy_items()
    }
}

impl ConveyorSplitter {
    fn determine_direction(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let last_direction =
            downcast::<Self>(&**world.get_block_at_mut(meta.position.x, meta.position.y)?.0)?.2;
        let itm = world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .get_item(0);
        let itm = if let Some(itm) = itm {
            itm.clone_item()
        } else {
            return None;
        };
        let sides_to_pushto = [
            meta.direction.next(false),
            meta.direction,
            meta.direction.next(true),
        ];

        let mut last_idx = 3_usize;
        let mut side = None;
        for i in last_direction..last_direction + 3 {
            let s = sides_to_pushto[i % 3];
            let pos = meta.position.add_directional(&s, 1);
            if let Some((blk, push_meta)) = world.get_block_at(pos.x, pos.y) {
                if blk.can_push(s.opposite(), &itm, push_meta) {
                    side = Some(s);
                    last_idx = (i + 1) % 3;
                    break;
                }
            }
        }
        let side = side?;
        let me = downcast_mut::<Self>(
            &mut **world.get_block_at_mut(meta.position.x, meta.position.y)?.0,
        )?;
        if last_idx < 3 {
            me.2 = last_idx;
            me.3 = Some(side);
        }

        None
    }
    fn update(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let direction =
            downcast::<Self>(&**world.get_block_at_mut(meta.position.x, meta.position.y)?.0)?.3?;
        let mut itm = world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .take_item(0)?;

        let me = downcast_mut::<Self>(
            &mut **world.get_block_at_mut(meta.position.x, meta.position.y)?.0,
        )?;
        me.3 = None;
        world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .take_item(0);
        let pos = meta.position.add_directional(&direction, 1);
        if let Some((blk, pushto_meta)) = world.get_block_at_mut(pos.x, pos.y) {
            itm = blk.push(direction.opposite(), itm, pushto_meta)?;
        }
        world
            .get_block_at_mut(meta.position.x, meta.position.y)?
            .0
            .get_inventory_capability()?
            .add_item(itm, 0);

        None
    }
}
