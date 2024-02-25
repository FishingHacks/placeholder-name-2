use std::time::Instant;

use lazy_static::lazy_static;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Vector2,
};

use crate::{
    block_impl_details_with_timer,
    blocks::downcast_mut,
    identifier::{GlobalString, Identifier},
    inventory::Inventory,
    reset_timer,
    scheduler::{schedule_task, Task},
    simple_single_item_serializable,
    world::{ChunkBlockMetadata, Direction, Vec2i, World},
    game::RenderLayer,
};

use super::Block;

lazy_static! {
    pub static ref EXTRACTOR_NAME: GlobalString = GlobalString::from("Extractor");
    pub static ref BLOCK_EXTRACTOR: Identifier =
        Identifier::from(("placeholder_name_2", "extractor"));
}

block_impl_details_with_timer!(ExtractorBlock, 250, Inventory);
impl Default for ExtractorBlock {
    fn default() -> Self {
        Self(Instant::now(), Inventory::new(1, false))
    }
}
impl Block for ExtractorBlock {
    simple_single_item_serializable!(1);

    fn description(&self) -> &'static str {
        "Extracts 4 Blocks per second from a machine"
    }

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
                let step_size = if matches!(meta.direction, Direction::North | Direction::South) {
                    h
                } else {
                    w
                };
                let lerp = (self.duration_lerp_value() * step_size as f32).floor() as i32 - w;
                let mut vec = Vec2i::new(x + 5, y + 5);
                vec.add_directional_assign(&meta.direction, lerp + step_size / 2);
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
    fn destroy_items(&self) -> Vec<Box<dyn crate::items::Item>> {
        self.1.destroy_items()
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
            item = blk.push(meta.direction.opposite(), item, push_meta)?;
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
