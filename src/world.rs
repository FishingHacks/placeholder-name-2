use raylib::drawing::RaylibDrawHandle;
use std::{collections::HashMap, fmt::Display};

use crate::{
    blocks::{empty_block, Block},
    identifier::Identifier,
};

pub struct World {
    pub loaded_chunks: HashMap<(i32, i32), Chunk>,
    pub w: u32,
    pub h: u32,
    pub startx: i32,
    pub starty: i32,
}

impl World {
    pub fn load_chunk(&mut self, x: i32, y: i32) {
        self.loaded_chunks.insert((x, y), Chunk::default(x, y));
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
        let blk = self
            .loaded_chunks
            .get(&(chunk_x, chunk_y))?
            .get_block_at(x, y);
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
            .loaded_chunks
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

        if let Some(chunk) = self.loaded_chunks.get_mut(&(chunk_x, chunk_y)) {
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
            loaded_chunks: HashMap::with_capacity(w as usize * h as usize),
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
        for (_, chunk) in self.loaded_chunks.iter_mut() {
            chunk.init();
        }
    }

    pub fn update(&self) {
        for (_, chunk) in self.loaded_chunks.iter() {
            chunk.update();
        }
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, w: u32, h: u32) {
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

                if let Some(chunk) = self.loaded_chunks.get_mut(&(chunk_x, chunk_y)) {
                    chunk.render(d, sc_x, sc_y, CHUNK_W, CHUNK_H, BLOCK_W, BLOCK_H);
                }
            }
        }
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
pub struct Chunk {
    pub blocks: Vec<ChunkBlock>,
    chunk_x: i32,
    chunk_y: i32,
}

impl Chunk {
    fn default(chunk_x: i32, chunk_y: i32) -> Self {
        let mut vec: Vec<ChunkBlock> =
            Vec::with_capacity(BLOCKS_PER_CHUNK_X as usize * BLOCKS_PER_CHUNK_Y as usize);

        for x in 0..BLOCKS_PER_CHUNK_X {
            for y in 0..BLOCKS_PER_CHUNK_Y {
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

    pub fn update(&self) {
        for blk in &self.blocks {
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
                );
            }
        }

        (w.min(blocks_x * block_w), h.min(blocks_y * block_h))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
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
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkBlockMetadata {
    pub position: (i32, i32),
    pub direction: Direction,
}

impl From<Direction> for ChunkBlockMetadata {
    fn from(direction: Direction) -> Self {
        Self {
            direction,
            position: (0, 0),
        }
    }
}

pub struct ChunkBlock {
    inner: Box<dyn Block>,
    data: ChunkBlockMetadata,
}

impl ChunkBlock {
    pub fn new(inner: Box<dyn Block>, pos_x: i32, pos_y: i32, direction: Direction) -> Self {
        Self {
            inner,
            data: ChunkBlockMetadata {
                direction,
                position: (pos_x, pos_y),
            },
        }
    }
    pub fn init(&mut self) {
        self.inner.init(self.data);
    }
    pub fn render(&self, d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32) {
        self.inner.render(d, x, y, w, h, self.data)
    }
    pub fn identifier(&self) -> Identifier {
        self.inner.identifier()
    }
    pub fn update(&self) {
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
            "Block {:?} at {}:{}",
            self.inner.identifier(),
            self.data.position.0,
            self.data.position.1
        ))
    }
}
