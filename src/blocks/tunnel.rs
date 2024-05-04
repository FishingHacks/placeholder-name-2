use std::time::Instant;

use lazy_static::lazy_static;
use raylib::{
    color::Color, drawing::RaylibDraw, math::{Rectangle, Vector2}, texture::Texture2D, RaylibHandle,
    RaylibThread,
};

use crate::{
    asset,
    assets::get_rotation_vec,
    block_impl_details_with_timer,
    blocks::downcast_mut,
    game::RenderLayer,
    identifier::{GlobalString, Identifier},
    initialized_data::InitializedData,
    inventory::Inventory,
    reset_timer,
    scheduler::{schedule_task, Task},
    serialization::{Deserialize, SerializationError, Serialize},
    step_size,
    world::{ChunkBlockMetadata, Direction, Vec2i, World},
};

use super::{conveyor::CONVEYOR_ANIMATION, Block};

lazy_static! {
    pub static ref TUNNEL_NAME: GlobalString = GlobalString::from("Tunnel tier 1");
    pub static ref BLOCK_TUNNEL: Identifier =
        Identifier::from(("placeholder_name_2", "tunnel mk 1"));
}

#[derive(Clone, Debug)]
pub enum TunnelType {
    Receiving(Vec2i),
    Pushing(Vec2i),
    None,
}

impl Serialize for TunnelType {
    fn required_length(&self) -> usize {
        u8::required_length(&0)
            + match self {
                Self::None => 0,
                Self::Pushing(vec) | Self::Receiving(vec) => vec.required_length(),
            }
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Pushing(vec) => {
                0.serialize(buf);
                vec.serialize(buf);
            }
            Self::Receiving(vec) => {
                1.serialize(buf);
                vec.serialize(buf);
            }
            Self::None => 2.serialize(buf),
        }
    }
}

impl Deserialize for TunnelType {
    fn try_deserialize(buf: &mut crate::serialization::Buffer) -> Result<Self, SerializationError> {
        match u8::try_deserialize(buf)? {
            0 => Ok(Self::Pushing(Deserialize::try_deserialize(buf)?)),
            1 => Ok(Self::Receiving(Deserialize::try_deserialize(buf)?)),
            2 => Ok(Self::None),
            _ => Err(SerializationError::InvalidData),
        }
    }
}

block_impl_details_with_timer!(TunnelBlock, 500, Inventory, Direction, TunnelType);

impl Default for TunnelBlock {
    fn default() -> Self {
        Self(
            Instant::now(),
            Inventory::new(1, false),
            Default::default(),
            TunnelType::None,
        )
    }
}

impl Block for TunnelBlock {
    fn is_building(&self) -> bool {
        true
    }

    fn required_length(&self) -> usize {
        self.1.required_length() + self.2.required_length() + self.3.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        self.1.serialize(buf);
        self.2.serialize(buf);
        self.3.serialize(buf);
    }

    fn try_deserialize(
        &mut self,
        buf: &mut crate::serialization::Buffer,
    ) -> Result<(), crate::serialization::SerializationError> {
        self.1 = Inventory::try_deserialize(buf)?;
        self.2 = Deserialize::try_deserialize(buf)?;
        self.3 = Deserialize::try_deserialize(buf)?;
        Ok(())
    }

    fn name(&self) -> GlobalString {
        *TUNNEL_NAME
    }
    fn identifier(&self) -> Identifier {
        *BLOCK_TUNNEL
    }
    fn description(&self) -> &'static str {
        "Moves 60 items per minute; Max length: 7 Blocks"
    }

    fn has_capability_push(&self, side: Direction, meta: crate::world::ChunkBlockMetadata) -> bool {
        matches!(self.3, TunnelType::Pushing(..)) && side == meta.direction.opposite()
    }

    fn can_push(
        &self,
        side: Direction,
        _: &Box<dyn crate::items::Item>,
        meta: crate::world::ChunkBlockMetadata,
    ) -> bool {
        self.has_capability_push(side, meta) && self.1.get_item(0).is_none()
    }

    fn push(
        &mut self,
        side: Direction,
        item: Box<dyn crate::items::Item>,
        meta: crate::world::ChunkBlockMetadata,
    ) -> Option<Box<dyn crate::items::Item>> {
        if !self.can_push(side, &item, meta) {
            Some(item)
        } else {
            reset_timer!(self);
            self.2 = meta.direction;
            self.1.get_item_mut(0).replace(item);
            None
        }
    }

    fn render(
        &self,
        d: &mut raylib::prelude::RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        meta: crate::world::ChunkBlockMetadata,
        render_layer: crate::game::RenderLayer,
    ) {
        if render_layer == RenderLayer::Block {
            CONVEYOR_ANIMATION.draw_resized_rotated(d, x, y, w, h, meta.direction);
        } else if render_layer == RenderLayer::OverlayItems {
            match self.3 {
                TunnelType::None | TunnelType::Pushing(..) => {
                    if let Some(item) = &self.1.get_item(0) {
                        let lerp_val = self.duration_lerp_value() / 2.0;
                        let step_size = step_size!(self.2, w, h);

                        let lerp = (lerp_val * step_size as f32).floor() as i32;
                        let mut vec = Vec2i::new(x + 5, y + 5);
                        vec.add_directional_assign(&self.2, -step_size / 2);
                        vec.add_directional_assign(&self.2, lerp);
                        item.render(d, vec.x, vec.y, w - 10, h - 10);
                    }
                }

                TunnelType::Receiving(..) => {
                    if let Some(item) = &self.1.get_item(0) {
                        let lerp_val = self.duration_lerp_value() / 2.0;
                        let step_size = step_size!(self.2, w, h);

                        let lerp = (lerp_val * step_size as f32).floor() as i32;
                        let mut vec = Vec2i::new(x + 5, y + 5);
                        vec.add_directional_assign(&meta.direction, lerp);
                        item.render(d, vec.x, vec.y, w - 10, h - 10);
                    }
                }
            }
            let dir = match self.3 {
                TunnelType::None | TunnelType::Pushing(..) => meta.direction,
                TunnelType::Receiving(..) => meta.direction.opposite(),
            };
            let (rot, vec) = get_rotation_vec(dir, Vec2i::new(x, y), w, h);
            d.draw_texture_ex(&*TUNNEL_OVERLAY, vec.as_vec2f(), rot, 1.0, Color::WHITE);
        } else if render_layer == RenderLayer::Preview {
            CONVEYOR_ANIMATION.draw_resized_rotated(d, x, y, w, h, meta.direction);

            let dir = match self.3 {
                TunnelType::None | TunnelType::Pushing(..) => meta.direction,
                TunnelType::Receiving(..) => meta.direction.opposite(),
            };
            let (rot, vec) = get_rotation_vec(dir, Vec2i::new(x, y), w, h);
            d.draw_texture_pro(
                &*TUNNEL_OVERLAY,
                Rectangle::new(0.0, 0.0, 64.0, 64.0),
                Rectangle::new(vec.x as f32, vec.y as f32, w as f32, h as f32),
                Vector2::zero(),
                rot,
                Color::WHITE,
            )
        }
    }

    fn supports_interaction(&self) -> bool {
        true
    }

    fn custom_interact_message(&self) -> Option<String> {
        Some("Press F to switch the orientation of the tunnels".to_string())
    }

    fn interact(&mut self, meta: ChunkBlockMetadata, _: &mut crate::game::GameConfig) {
        schedule_task(Task::WorldUpdateBlock(&Self::reverse_orientation, meta))
    }

    fn init(&mut self, _: ChunkBlockMetadata) {
        self.1.resize(1);
    }

    fn on_after_dismantle(&mut self, _: ChunkBlockMetadata, world: &mut World) {
        let vec = match self.3 {
            TunnelType::None => return,
            TunnelType::Pushing(vec) | TunnelType::Receiving(vec) => vec,
        };
        world
            .get_block_at_mut(vec.x, vec.y)
            .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))
            .map(|other| other.3 = TunnelType::None);
    }

    fn destroy_items(&self) -> Vec<Box<dyn crate::items::Item>> {
        self.1.destroy_items()
    }

    fn on_before_place(&mut self, meta: ChunkBlockMetadata, world: &mut crate::world::World) {
        let mut blk_pos: Option<Vec2i> = None;
        for i in -7..=7 {
            if i == 0 {
                continue;
            }
            let new_pos = meta.position.add_directional(&meta.direction, i);
            if world
                .get_block_at(new_pos.x, new_pos.y)
                .map(|(blk, blk_meta)| {
                    blk.identifier() == *BLOCK_TUNNEL && blk_meta.direction == meta.direction
                })
                .unwrap_or(false)
            {
                blk_pos = Some(new_pos);
                break;
            }
        }
        if let Some(blk_pos) = blk_pos {
            let blk = match world.get_block_at_mut(blk_pos.x, blk_pos.y) {
                Some(v) => v.0,
                None => return,
            };
            let blk = downcast_mut::<Self>(&mut **blk);
            if let Some(blk) = blk {
                blk.3 = TunnelType::Receiving(meta.position);
                self.3 = TunnelType::Pushing(blk_pos);
            }
        } else {
            self.3 = TunnelType::None;
        }
    }

    fn update(&mut self, meta: ChunkBlockMetadata) {
        if !self.can_do_work() {
            return;
        }
        if matches!(self.3, TunnelType::Pushing(..) | TunnelType::Receiving(..)) {
            self.1.update();
            schedule_task(Task::WorldUpdateBlock(&Self::update, meta))
        }
    }
}

impl TunnelBlock {
    fn reverse_orientation(meta: ChunkBlockMetadata, world: &mut World) {
        if let Some(self_blk) = world
            .get_block_at_mut(meta.position.x, meta.position.y)
            .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))
        {
            let vec = match self_blk.3 {
                TunnelType::None => return,
                TunnelType::Pushing(vec) => {
                    self_blk.3 = TunnelType::Receiving(vec);
                    vec
                }
                TunnelType::Receiving(vec) => {
                    self_blk.3 = TunnelType::Pushing(vec);
                    vec
                }
            };

            if let Some(other_blk) = world
                .get_block_at_mut(vec.x, vec.y)
                .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))
            {
                match other_blk.3 {
                    TunnelType::None => return,
                    TunnelType::Pushing(vec) => other_blk.3 = TunnelType::Receiving(vec),
                    TunnelType::Receiving(vec) => other_blk.3 = TunnelType::Pushing(vec),
                }
            }
        }
    }

    fn update_push(meta: ChunkBlockMetadata, world: &mut World) -> Option<()> {
        let self_blk = world
            .get_block_at_mut(meta.position.x, meta.position.y)
            .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))?;

        if !self_blk.can_do_work() {
            return None;
        }

        match self_blk.3 {
            TunnelType::None => {}
            TunnelType::Pushing(vec) => {
                let item = self_blk.1.take_item(0)?;
                if let Some(other) = world
                    .get_block_at_mut(vec.x, vec.y)
                    .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))
                {
                    if other.1.get_item(0).is_none() {
                        reset_timer!(other);
                        other.1.get_item_mut(0).replace(item);
                        return None;
                    }
                } else {
                    let self_blk = world
                        .get_block_at_mut(meta.position.x, meta.position.y)
                        .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))?;
                    self_blk.3 = TunnelType::None;
                    self_blk.1.get_item_mut(0).replace(item);
                    return None;
                }
                world
                    .get_block_at_mut(meta.position.x, meta.position.y)
                    .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))?
                    .1
                    .get_item_mut(0)
                    .replace(item);
            }
            TunnelType::Receiving(..) => {
                let mut item = self_blk.1.take_item(0)?;
                let vec = meta.position.add_directional(&meta.direction, 1);

                if let Some((other_blk, other_meta)) = world.get_block_at_mut(vec.x, vec.y) {
                    item = other_blk.push(meta.direction.opposite(), item, other_meta)?;
                }

                world
                    .get_block_at_mut(meta.position.x, meta.position.y)
                    .and_then(|(blk, _)| downcast_mut::<Self>(&mut **blk))?
                    .1
                    .get_item_mut(0)
                    .replace(item);
            }
        }

        None
    }

    fn update(meta: ChunkBlockMetadata, world: &mut World) {
        Self::update_push(meta, world);
    }

    pub fn load_block_files(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), String> {
        TUNNEL_OVERLAY.init(rl.load_texture(thread, asset!("tunnel_overlay.png").as_str())?);

        Ok(())
    }
}

pub static TUNNEL_OVERLAY: InitializedData<Texture2D> = InitializedData::new();
