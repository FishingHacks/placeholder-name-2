use raylib::drawing::RaylibDrawHandle;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    blocks::{empty_block, Block},
    identifier::Identifier,
    serialization::{Buffer, Deserialize, SerializationError, SerializationTrap, Serialize},
    RenderLayer,
};

#[derive(Clone)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub w: u32,
    pub h: u32,
    pub startx: i32,
    pub starty: i32,
}

impl World {
    pub fn load_chunk(&mut self, x: i32, y: i32) {
        self.chunks.insert((x, y), Chunk::default(x, y));
    }

    pub fn get_block_at<'a>(
        &'a self,
        x: i32,
        y: i32,
    ) -> Option<(&'a Box<dyn Block>, ChunkBlockMetadata)> {
        let mut chunk_x = x / BLOCKS_PER_CHUNK_X as i32;
        let mut chunk_y = y / BLOCKS_PER_CHUNK_Y as i32;

        if (x % BLOCKS_PER_CHUNK_X as i32) < 0 {
            chunk_x -= 1;
        }
        if (y % BLOCKS_PER_CHUNK_Y as i32) < 0 {
            chunk_y -= 1;
        }
        let blk = self.chunks.get(&(chunk_x, chunk_y))?.get_block_at(x, y);
        Some((&blk.inner, blk.data))
    }

    pub fn get_block_at_mut<'a>(
        &'a mut self,
        x: i32,
        y: i32,
    ) -> Option<(&'a mut Box<dyn Block>, ChunkBlockMetadata)> {
        let mut chunk_x = x / BLOCKS_PER_CHUNK_X as i32;
        let mut chunk_y = y / BLOCKS_PER_CHUNK_Y as i32;

        if (x % BLOCKS_PER_CHUNK_X as i32) < 0 {
            chunk_x -= 1;
        }
        if (y % BLOCKS_PER_CHUNK_Y as i32) < 0 {
            chunk_y -= 1;
        }
        let blk = self
            .chunks
            .get_mut(&(chunk_x, chunk_y))?
            .get_block_at_mut(x, y);
        Some((&mut blk.inner, blk.data))
    }

    pub fn set_block_at(&mut self, x: i32, y: i32, block: Box<dyn Block>, dir: Direction) -> bool {
        let mut chunk_x = x / BLOCKS_PER_CHUNK_X as i32;
        let mut chunk_y = y / BLOCKS_PER_CHUNK_Y as i32;

        if (x % BLOCKS_PER_CHUNK_X as i32) < 0 {
            chunk_x -= 1;
        }
        if (y % BLOCKS_PER_CHUNK_Y as i32) < 0 {
            chunk_y -= 1;
        }

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y)) {
            chunk.set_block_at(x, y, block, dir);
            true
        } else {
            false
        }
    }

    pub fn new(w: u32, h: u32) -> Self {
        let off_x = -((w / 2) as i32);
        let off_y = -((h / 2) as i32);

        let mut world = Self {
            chunks: HashMap::with_capacity(w as usize * h as usize),
            startx: off_x,
            starty: off_y,
            w,
            h,
        };

        for x in 0..w as i32 {
            for y in 0..h as i32 {
                world.load_chunk(off_x + x, off_y + y)
            }
        }

        world
    }

    pub fn init(&mut self) {
        for (_, chunk) in self.chunks.iter_mut() {
            chunk.init();
        }
    }

    pub fn update(&mut self) {
        for (_, chunk) in self.chunks.iter_mut() {
            chunk.update();
        }
    }

    pub fn render(
        &mut self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        layer: RenderLayer,
    ) {
        let first_chunk_x = 0.max((x.wrapping_div(CHUNK_W as i32)) - self.startx - 1) as u32;
        let first_chunk_y = 0.max((y.wrapping_div(CHUNK_H as i32)) - self.starty - 1) as u32;

        let last_chunk_x = self
            .w
            .min(w.wrapping_div_euclid(CHUNK_W) + 3 + first_chunk_x);
        let last_chunk_y = self
            .h
            .min(h.wrapping_div_euclid(CHUNK_H) + 3 + first_chunk_y);

        for chunk_x in first_chunk_x..last_chunk_x {
            for chunk_y in first_chunk_y..last_chunk_y {
                let chunk_x = chunk_x as i32 + self.startx;
                let chunk_y = chunk_y as i32 + self.starty;
                let sc_x = chunk_x * CHUNK_W as i32 - x;
                let sc_y = chunk_y * CHUNK_H as i32 - y;

                if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk.render(d, sc_x, sc_y, CHUNK_W, CHUNK_H, BLOCK_W, BLOCK_H, layer);
                }
            }
        }
    }
}

impl Serialize for World {
    fn required_length(&self) -> usize {
        // self.chunks.required_length()
        self.chunks
            .values()
            .map(|chunk| chunk.required_length())
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + self.w.required_length()
            + self.h.required_length()
            + self.startx.required_length()
            + self.starty.required_length()
            + SerializationTrap::World.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::World.serialize(buf);
        self.startx.serialize(buf);
        self.starty.serialize(buf);
        self.w.serialize(buf);
        self.h.serialize(buf);
        // self.chunks.serialize(buf);
        assert_eq!(self.w as usize * self.h as usize, self.chunks.len());
        let mut vals = self
            .chunks
            .iter()
            .map(|(&(a, b), chunk)| {
                (
                    (a + self.startx.abs()) as usize
                        + (b + self.startx.abs()) as usize * self.w as usize,
                    chunk,
                )
            })
            .collect::<Vec<(usize, &Chunk)>>();
        vals.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, chunk) in vals {
            chunk.serialize(buf);
        }
    }
}

impl Deserialize for World {
    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::World.deserialize(buf);
        let startx = i32::deserialize(buf);
        let starty = i32::deserialize(buf);
        let w = u32::deserialize(buf);
        let h = u32::deserialize(buf);

        let num_chunks = w as usize * h as usize;
        let mut chunks = HashMap::with_capacity(num_chunks);

        for i in 0..(w as usize * h as usize) {
            let x = (i % w as usize) as i32 + startx;
            let y = (i / w as usize) as i32 + starty;

            chunks.insert((x, y), Chunk::deserialize(buf));
        }

        Self {
            chunks,
            startx,
            starty,
            w,
            h,
        }
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::World.try_deserialize(buf)?;
        let startx = i32::try_deserialize(buf)?;
        let starty = i32::try_deserialize(buf)?;
        let w = u32::try_deserialize(buf)?;
        let h = u32::try_deserialize(buf)?;

        let num_chunks = w as usize * h as usize;
        let mut chunks = HashMap::with_capacity(num_chunks);

        for i in 0..(w as usize * h as usize) {
            let x = (i % w as usize) as i32 + startx;
            let y = (i / w as usize) as i32 + starty;

            chunks.insert((x, y), Chunk::try_deserialize(buf)?);
        }

        Ok(Self {
            chunks,
            startx,
            starty,
            w,
            h,
        })
    }
}

pub const BLOCK_W: u32 = 64;
pub const BLOCK_H: u32 = 64;
pub const BLOCKS_PER_CHUNK_X: u32 = 32;
pub const BLOCKS_PER_CHUNK_Y: u32 = 32;
pub const CHUNK_W: u32 = BLOCK_W * BLOCKS_PER_CHUNK_X;
pub const CHUNK_H: u32 = BLOCK_H * BLOCKS_PER_CHUNK_Y;

/// chunks: 32x32 area
#[derive(Clone)]
#[allow(dead_code)]
pub struct Chunk {
    pub blocks: Vec<ChunkBlock>,
    chunk_x: i32,
    chunk_y: i32,
}

impl Chunk {
    fn default(chunk_x: i32, chunk_y: i32) -> Self {
        let mut vec: Vec<ChunkBlock> =
            Vec::with_capacity(BLOCKS_PER_CHUNK_X as usize * BLOCKS_PER_CHUNK_Y as usize);

        for y in 0..BLOCKS_PER_CHUNK_Y {
            for x in 0..BLOCKS_PER_CHUNK_X {
                let blk = ChunkBlock::new(
                    empty_block().clone_block(),
                    x as i32 + chunk_x * BLOCKS_PER_CHUNK_X as i32,
                    y as i32 + chunk_y * BLOCKS_PER_CHUNK_Y as i32,
                    Direction::North,
                );

                vec.push(blk);
            }
        }
        Self {
            blocks: vec,
            chunk_x,
            chunk_y,
        }
    }

    pub fn set_block_at(&mut self, x: i32, y: i32, new_block: Box<dyn Block>, dir: Direction) {
        let blk = ChunkBlock::new(new_block, x, y, dir);

        let mut off_x = x % BLOCKS_PER_CHUNK_X as i32;
        let mut off_y = y % BLOCKS_PER_CHUNK_Y as i32;

        if off_x < 0 {
            off_x += BLOCKS_PER_CHUNK_X as i32;
        }
        if off_y < 0 {
            off_y += BLOCKS_PER_CHUNK_X as i32;
        }

        self.blocks[off_y as usize * BLOCKS_PER_CHUNK_X as usize + off_x as usize] = blk;
        self.blocks[off_y as usize * BLOCKS_PER_CHUNK_X as usize + off_x as usize].init();
    }

    pub fn get_block_at<'a>(&'a self, x: i32, y: i32) -> &'a ChunkBlock {
        let mut off_x = x % BLOCKS_PER_CHUNK_X as i32;
        let mut off_y = y % BLOCKS_PER_CHUNK_Y as i32;

        if off_x < 0 {
            off_x += BLOCKS_PER_CHUNK_X as i32;
        }
        if off_y < 0 {
            off_y += BLOCKS_PER_CHUNK_X as i32;
        }

        &self.blocks[off_y as usize * BLOCKS_PER_CHUNK_X as usize + off_x as usize]
    }

    pub fn get_block_at_mut<'a>(&'a mut self, x: i32, y: i32) -> &'a mut ChunkBlock {
        let mut off_x = x % BLOCKS_PER_CHUNK_X as i32;
        let mut off_y = y % BLOCKS_PER_CHUNK_Y as i32;

        if off_x < 0 {
            off_x += BLOCKS_PER_CHUNK_X as i32;
        }
        if off_y < 0 {
            off_y += BLOCKS_PER_CHUNK_X as i32;
        }

        &mut self.blocks[off_y as usize * BLOCKS_PER_CHUNK_X as usize + off_x as usize]
    }

    pub fn init(&mut self) {
        for blk in &mut self.blocks {
            blk.init();
        }
    }

    pub fn update(&mut self) {
        for blk in &mut self.blocks {
            blk.update();
        }
    }

    pub fn render(
        &mut self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        block_w: u32,
        block_h: u32,
        layer: RenderLayer,
    ) -> (u32, u32) {
        let blocks_x = w.div_ceil(block_w).min(BLOCKS_PER_CHUNK_X);
        let blocks_y = h.div_ceil(block_h).min(BLOCKS_PER_CHUNK_Y);

        for blk_y in 0..blocks_y {
            for blk_x in 0..blocks_x {
                self.blocks[blk_y as usize * BLOCKS_PER_CHUNK_X as usize + blk_x as usize].render(
                    d,
                    x + (blk_x * block_w) as i32,
                    y + (blk_y * block_h) as i32,
                    block_w as i32,
                    block_h as i32,
                    layer,
                );
            }
        }

        (w.min(blocks_x * block_w), h.min(blocks_y * block_h))
    }
}

impl Serialize for Chunk {
    fn required_length(&self) -> usize {
        SerializationTrap::Chunk.required_length()
            + self.chunk_x.required_length()
            + self.chunk_y.required_length()
            + self
                .blocks
                .iter()
                .map(|blk| blk.inner.required_length() + blk.data.direction.required_length())
                .reduce(|a, b| a + b)
                .unwrap_or_default()
            + usize::required_length(&0)
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Chunk.serialize(buf);
        self.chunk_x.serialize(buf);
        self.chunk_y.serialize(buf);
        self.blocks.len().serialize(buf);
        for b in &self.blocks {
            b.data.direction.serialize(buf);
            b.inner.serialize(buf);
        }
    }
}

impl Deserialize for Chunk {
    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::Chunk.deserialize(buf);
        let chunk_x = i32::deserialize(buf);
        let chunk_y = i32::deserialize(buf);
        let num_blocks = usize::deserialize(buf);
        let mut blocks: Vec<ChunkBlock> = Vec::with_capacity(num_blocks);

        for y in 0..BLOCKS_PER_CHUNK_Y {
            for x in 0..BLOCKS_PER_CHUNK_X {
                let direction = Direction::deserialize(buf);
                let inner = <Box<dyn Block>>::deserialize(buf);
                let blk = ChunkBlock::new(
                    inner,
                    x as i32 + chunk_x * BLOCKS_PER_CHUNK_X as i32,
                    y as i32 + chunk_y * BLOCKS_PER_CHUNK_Y as i32,
                    direction,
                );

                blocks.push(blk);
            }
        }
        Self {
            blocks,
            chunk_x,
            chunk_y,
        }
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Chunk.try_deserialize(buf)?;
        let chunk_x = i32::try_deserialize(buf)?;
        let chunk_y = i32::try_deserialize(buf)?;
        let num_blocks = usize::try_deserialize(buf)?;
        let mut blocks: Vec<ChunkBlock> = Vec::with_capacity(num_blocks);

        for y in 0..BLOCKS_PER_CHUNK_Y {
            for x in 0..BLOCKS_PER_CHUNK_X {
                let direction = Direction::try_deserialize(buf)?;
                let inner = <Box<dyn Block>>::try_deserialize(buf)?;
                let blk = ChunkBlock::new(
                    inner,
                    x as i32 + chunk_x * BLOCKS_PER_CHUNK_X as i32,
                    y as i32 + chunk_y * BLOCKS_PER_CHUNK_Y as i32,
                    direction,
                );

                blocks.push(blk);
            }
        }
        Ok(Self {
            blocks,
            chunk_x,
            chunk_y,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}

impl Serialize for Direction {
    fn required_length(&self) -> usize {
        1
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        (*self as u8).serialize(buf)
    }
}

impl Deserialize for Direction {
    fn deserialize(buf: &mut Buffer) -> Self {
        Self::from(u8::deserialize(buf))
    }
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        Ok(Self::from(u8::try_deserialize(buf)?))
    }
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Self::North,
            1 => Self::East,
            2 => Self::South,
            3 => Self::West,
            _ => Self::North,
        }
    }
}

impl Add for Direction {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self as u8 + rhs as u8).into()
    }
}

impl Direction {
    pub fn next(&self, right: bool) -> Self {
        match self {
            Self::North if right => Self::East,
            Self::East if right => Self::South,
            Self::South if right => Self::West,
            Self::West if right => Self::North,
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Serialize for Vec2i {
    fn required_length(&self) -> usize {
        self.x.required_length()
            + self.y.required_length()
            + usize::required_length(&0)
            + SerializationTrap::Vec.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Vec.serialize(buf);
        usize::serialize(&2, buf);
        self.x.serialize(buf);
        self.y.serialize(buf);
    }
}

impl Deserialize for Vec2i {
    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::Vec.deserialize(buf);
        let len = usize::deserialize(buf);
        if len != 2 {
            panic!("Vec2i: Expected a vector length of 2");
        }
        let x = i32::deserialize(buf);
        let y = i32::deserialize(buf);
        Self { x, y }
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Vec.try_deserialize(buf)?;
        let len = usize::try_deserialize(buf)?;
        if len != 2 {
            return Err(SerializationError::InvalidData);
        }
        let x = i32::try_deserialize(buf)?;
        let y = i32::try_deserialize(buf)?;
        Ok(Self { x, y })
    }
}

impl Add for Vec2i {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2i {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2i {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2i {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Display for Vec2i {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, {}", self.x, self.y))
    }
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add_directional(&self, direction: &Direction, steps: i32) -> Vec2i {
        match direction {
            Direction::North => *self - Self::new(0, steps),
            Direction::South => *self + Self::new(0, steps),
            Direction::East => *self - Self::new(steps, 0),
            Direction::West => *self + Self::new(steps, 0),
        }
    }

    pub fn add_directional_assign(&mut self, direction: &Direction, steps: i32) {
        match direction {
            Direction::North => self.y -= steps,
            Direction::South => self.y += steps,
            Direction::East => self.x -= steps,
            Direction::West => self.x += steps,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkBlockMetadata {
    pub position: Vec2i,
    pub direction: Direction,
}

impl From<Direction> for ChunkBlockMetadata {
    fn from(direction: Direction) -> Self {
        Self {
            direction,
            position: Vec2i::default(),
        }
    }
}

impl Serialize for ChunkBlockMetadata {
    fn required_length(&self) -> usize {
        self.position.required_length() + self.direction.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        self.position.serialize(buf);
        self.direction.serialize(buf);
    }
}

impl Deserialize for ChunkBlockMetadata {
    fn deserialize(buf: &mut Buffer) -> Self {
        let position = Vec2i::deserialize(buf);
        let direction = Direction::deserialize(buf);

        Self {
            position,
            direction,
        }
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        let position = Vec2i::try_deserialize(buf)?;
        let direction = Direction::try_deserialize(buf)?;

        Ok(Self {
            position,
            direction,
        })
    }
}

pub struct ChunkBlock {
    inner: Box<dyn Block>,
    data: ChunkBlockMetadata,
}

impl Serialize for ChunkBlock {
    fn required_length(&self) -> usize {
        self.inner.required_length() + self.data.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        self.data.serialize(buf);
        self.inner.serialize(buf);
    }
}

impl Deserialize for ChunkBlock {
    fn deserialize(buf: &mut Buffer) -> Self {
        let data = ChunkBlockMetadata::deserialize(buf);
        let inner = <Box<dyn Block>>::deserialize(buf);
        Self { data, inner }
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        let data = ChunkBlockMetadata::try_deserialize(buf)?;
        let inner = <Box<dyn Block>>::try_deserialize(buf)?;
        Ok(Self { data, inner })
    }
}

impl ChunkBlock {
    pub fn new(inner: Box<dyn Block>, pos_x: i32, pos_y: i32, direction: Direction) -> Self {
        Self {
            inner,
            data: ChunkBlockMetadata {
                direction,
                position: Vec2i::new(pos_x, pos_y),
            },
        }
    }
    pub fn init(&mut self) {
        self.inner.init(self.data);
    }
    pub fn render(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        layer: RenderLayer,
    ) {
        self.inner.render(d, x, y, w, h, self.data, layer)
    }
    pub fn identifier(&self) -> Identifier {
        self.inner.identifier()
    }
    pub fn update(&mut self) {
        self.inner.update(self.data);
    }
}

impl Clone for ChunkBlock {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            inner: self.inner.clone_block(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.data = source.data;
        self.inner = source.inner.clone_block();
    }
}

impl Display for ChunkBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Block {:?} at {}",
            self.inner.identifier(),
            self.data.position
        ))
    }
}
