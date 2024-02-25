use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::{Rectangle, Vector2},
    texture::Texture2D,
    RaylibHandle, RaylibThread,
};

use crate::{initialized_data::InitializedData, world::Direction};

#[macro_export]
macro_rules! asset {
    ($($path: expr),*) => {
        {
            let mut dir = std::env::current_dir().unwrap().join("assets");
            $(
                dir.push($path);
            )*
            format!("{}", dir.display())
        }
    };
}

const ORIGIN: Vector2 = Vector2 { x: 0.0, y: 0.0 };

impl Display for AnimatedTexture2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Animated Texture: {}x{}; {} frames; format: {}, mipmaps: {}, id: {}\n\n",
            self.width,
            self.height,
            self.frames.len(),
            self.texture.format,
            self.texture.mipmaps,
            self.texture.id
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub id: u8,
    pub length: u64,
}

impl Frame {
    pub fn new(id: u8, length: u64) -> Self {
        Self { id, length }
    }

    pub fn multiple(length_per_frame: u64, num_frames: u8) -> Vec<Self> {
        let mut vec = Vec::with_capacity(num_frames as usize);
        for i in 0..num_frames {
            vec.push(Self {
                id: i,
                length: length_per_frame,
            });
        }

        vec
    }
}

static ANIMATED_TEXTURES: InitializedData<HashMap<String, AnimatedTexture2D>> =
    InitializedData::new();

pub fn load_animated_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    path: String,
    frames: Vec<Frame>,
    width: u32,
    height: u32,
    id: Option<String>,
) -> Result<&'static AnimatedTexture2D, String> {
    ANIMATED_TEXTURES.maybe_init_default();

    let id = id.unwrap_or(
        path.split('/')
            .last()
            .ok_or("Invalid Filepath".to_string())?
            .to_string(),
    );
    if let Some(text) = get_animated_texture(&id) {
        return Ok(text);
    }
    let texture = rl.load_texture(thread, path.as_str())?;

    unsafe {
        ANIMATED_TEXTURES.get_mut().insert(
            id.clone(),
            AnimatedTexture2D::new(texture, frames, width, height),
        );
    }
    Ok((*ANIMATED_TEXTURES).get(&id).unwrap())
}

pub fn get_animated_texture(id: &String) -> Option<&'static AnimatedTexture2D> {
    (*ANIMATED_TEXTURES).get(id)
}

#[derive(Debug)]
pub struct AnimatedTexture2D {
    pub texture: Texture2D,
    pub frames: Vec<Frame>,
    pub width: u32,
    pub height: u32,
    length: u128,
    current_frame: u32,
}

pub fn update_textures() {
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards ftw").as_millis();
    for (_, texture) in unsafe { ANIMATED_TEXTURES.get_mut().iter_mut() } {
        let mut local_ms = ms % texture.length;
        let mut frame: u32 = 0;

        for f in &texture.frames {
            if local_ms > f.length as u128 {
                local_ms -= f.length as u128;
            } else {
                frame = f.id as u32;
                break;
            }
        }

        texture.current_frame = frame;
    }
}

impl AnimatedTexture2D {
    fn new(texture: Texture2D, frames: Vec<Frame>, width: u32, height: u32) -> Self {
        let length = frames
            .iter()
            .map(|f| f.length as u128)
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        Self {
            frames,
            height,
            texture,
            width,
            length,
            current_frame: 0,
        }
    }

    pub fn get_texture_rect(&self) -> Rectangle {
        Rectangle::new(
            0.0,
            (self.current_frame * self.height) as f32,
            self.width as f32,
            self.height as f32,
        )
    }

    pub fn get_frame_texture_rect(&self, current_frame: u32) -> Rectangle {
        Rectangle::new(
            0.0,
            (current_frame * self.height) as f32,
            self.width as f32,
            self.height as f32,
        )
    }

    pub fn draw(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32) {
        renderer.draw_texture_pro(
            &self.texture,
            self.get_texture_rect(),
            Rectangle::new(x as f32, y as f32, self.width as f32, self.height as f32),
            ORIGIN,
            0.0,
            Color::WHITE,
        );
    }

    pub fn draw_tinted(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, tint: Color) {
        renderer.draw_texture_pro(
            &self.texture,
            self.get_texture_rect(),
            Rectangle::new(x as f32, y as f32, self.width as f32, self.height as f32),
            ORIGIN,
            0.0,
            tint,
        );
    }

    pub fn draw_scaled(&self, renderer: &mut RaylibDrawHandle, x: i32, y: i32, scale: f32) {
        renderer.draw_texture_pro(
            &self.texture,
            self.get_texture_rect(),
            Rectangle::new(
                x as f32,
                y as f32,
                self.width as f32 * scale,
                self.height as f32 * scale,
            ),
            ORIGIN,
            0.0,
            Color::WHITE,
        );
    }

    pub fn draw_resized(
        &self,
        renderer: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        renderer.draw_texture_pro(
            &self.texture,
            self.get_texture_rect(),
            Rectangle::new(x as f32, y as f32, width as f32, height as f32),
            ORIGIN,
            0.0,
            Color::WHITE,
        );
    }

    pub fn draw_resized_rotated(
        &self,
        renderer: &mut RaylibDrawHandle,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rotation: Direction,
    ) {
        let mut dest = Rectangle::new(x as f32, y as f32, width as f32, height as f32);
        let rotation = match rotation {
            Direction::North => 0.0,
            Direction::South => {
                dest.x += width as f32;
                dest.y += height as f32;
                180.0
            }
            Direction::East => {
                dest.y += height as f32;
                270.0
            }
            Direction::West => {
                dest.x += width as f32;
                90.0
            }
        };
        renderer.draw_texture_pro(
            &self.texture,
            self.get_texture_rect(),
            dest,
            ORIGIN,
            rotation,
            Color::WHITE,
        );
    }
}
