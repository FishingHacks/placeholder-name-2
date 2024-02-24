use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    ops::Add,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    blocks::{empty_block, get_block_by_id, Block, BLOCK_EMPTY},
    identifier::Identifier,
    inventory::Inventory,
    items::{get_item_by_id, Item},
    world::World,
    GameConfig,
};

pub struct Buffer(Vec<u8>, usize);

impl Buffer {
    pub fn new(vec: Vec<u8>) -> Self {
        Self(vec, 0)
    }

    pub fn len(&self) -> usize {
        self.0.len().saturating_sub(self.1)
    }

    pub fn read_elements<'a>(&'a mut self, num: usize) -> &'a [u8] {
        self.1 += num;
        if self.1 >= self.0.len() {
            panic!("read more elements than possible ohnyu");
        }
        &self.0[self.1 - num..self.1]
    }

    pub fn try_read_elements<'a>(&'a mut self, num: usize) -> Result<&'a [u8], SerializationError> {
        self.1 += num;
        if self.1 >= self.0.len() {
            Err(SerializationError::NotEnoughSpace)
        } else {
            Ok(&self.0[self.1 - num..self.1])
        }
    }

    pub fn read_element(&mut self) -> u8 {
        if self.len() < 1 {
            panic!("read more elements than possible ohnyu");
        }
        self.1 += 1;
        self.0[self.1 - 1]
    }
    pub fn try_read_element(&mut self) -> Result<u8, SerializationError> {
        if self.len() < 1 {
            Err(SerializationError::NotEnoughSpace)
        } else {
            self.1 += 1;
            Ok(self.0[self.1 - 1])
        }
    }
}

pub trait Serialize: Sized {
    fn serialize(&self, buf: &mut Vec<u8>);
    fn required_length(&self) -> usize;
}

pub trait Deserialize: Sized {
    fn deserialize(buf: &mut Buffer) -> Self {
        Self::try_deserialize(buf).expect("Failed to deserialize")
    }
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError>;
}

#[derive(Debug)]
pub enum SerializationError {
    NotEnoughSpace,
    Other,
    Io(std::io::Error),
    InvalidData,
    SerializeTrap {
        found: SerializationTrap,
        expected: SerializationTrap,
    },
}

macro_rules! num_serializable {
    ($name: ty) => {
        impl Serialize for $name {
            fn serialize(&self, buf: &mut Vec<u8>) {
                buf.reserve(std::mem::size_of::<$name>());
                buf.extend(self.to_le_bytes())
            }
            fn required_length(&self) -> usize { std::mem::size_of::<$name>() }
        }

        impl Deserialize for $name {
            fn deserialize(buf: &mut Buffer) -> Self { Self::try_deserialize(buf).expect(concat!("Failed to deserialize ", stringify!($name))) }
            fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
                let bytes: [u8; std::mem::size_of::<$name>()] = buf.try_read_elements(std::mem::size_of::<$name>())?.try_into().unwrap();
                Ok(<$name>::from_le_bytes(bytes))

            }
        }
    };
    ($($name: ty),+) => {
        $(
            num_serializable!($name);
        )+
    }
}

num_serializable!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl Serialize for bool {
    fn serialize(&self, buf: &mut Vec<u8>) {
        if *self {
            buf.push(1)
        } else {
            buf.push(0)
        }
    }
    fn required_length(&self) -> usize {
        1
    }
}
impl Deserialize for bool {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        if buf.try_read_elements(1)?[0] != 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Vec.serialize(buf);
        self.len().serialize(buf);
        self.iter().for_each(|v| v.serialize(buf));
    }

    fn required_length(&self) -> usize {
        self.iter()
            .map(|v| v.required_length())
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + usize::required_length(&0)
            + SerializationTrap::required_length()
    }
}

impl<T: Serialize> Serialize for &[T] {
    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Vec.serialize(buf);
        self.len().serialize(buf);
        self.iter().for_each(|v| v.serialize(buf));
    }

    fn required_length(&self) -> usize {
        self.iter()
            .map(|v| v.required_length())
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + usize::required_length(&0)
            + SerializationTrap::required_length()
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Vec.try_deserialize(buf)?;
        let len = usize::try_deserialize(buf)?;
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::try_deserialize(buf)?);
        }

        Ok(vec)
    }

    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::Vec.deserialize(buf);
        let len = usize::deserialize(buf);
        let mut vec: Vec<T> = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::deserialize(buf));
        }

        vec
    }
}

impl<K: Serialize + Hash + Eq, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::HashMap.serialize(buf);
        self.len().serialize(buf);
        for (k, v) in self {
            k.serialize(buf);
            v.serialize(buf);
        }
    }

    fn required_length(&self) -> usize {
        self.iter()
            .map(|(k, v)| k.required_length() + v.required_length())
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + usize::required_length(&0)
            + SerializationTrap::required_length()
    }
}

impl<K: Deserialize + Hash + Eq, V: Deserialize> Deserialize for HashMap<K, V> {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::HashMap.try_deserialize(buf)?;
        let num_elements = usize::try_deserialize(buf)?;
        let mut hashmap: HashMap<K, V> = HashMap::with_capacity(num_elements);

        for _ in 0..num_elements {
            let key = K::try_deserialize(buf)?;
            let value = V::try_deserialize(buf)?;
            hashmap.insert(key, value);
        }

        Ok(hashmap)
    }
}

impl Serialize for String {
    fn required_length(&self) -> usize {
        self.as_bytes().required_length() + SerializationTrap::required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::String.serialize(buf);
        self.as_bytes().serialize(buf)
    }
}

impl Deserialize for String {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::String.try_deserialize(buf)?;
        let bytes = Vec::<u8>::try_deserialize(buf)?;
        match String::from_utf8(bytes) {
            Ok(str) => Ok(str),
            Err(..) => Err(SerializationError::InvalidData),
        }
    }
}

impl Serialize for &str {
    fn required_length(&self) -> usize {
        self.as_bytes().required_length() + SerializationTrap::required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::String.serialize(buf);
        self.as_bytes().serialize(buf)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn required_length(&self) -> usize {
        SerializationTrap::required_length()
            + bool::required_length(&false)
            + match self {
                Some(v) => v.required_length(),
                None => 0,
            }
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Option.serialize(buf);
        self.is_some().serialize(buf);
        match self {
            Some(v) => v.serialize(buf),
            None => {}
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Option.try_deserialize(buf)?;
        if bool::try_deserialize(buf)? {
            Ok(Some(T::try_deserialize(buf)?))
        } else {
            Ok(None)
        }
    }

    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::Option.deserialize(buf);
        if bool::deserialize(buf) {
            Some(T::deserialize(buf))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SerializationTrap {
    String,
    HashMap,
    Vec,
    Option,
    Item,
    Block,
    Chunk,
    World,
    Time,

    Unknown = 0xff,
}

impl Serialize for SerializationTrap {
    fn required_length(&self) -> usize {
        1
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl SerializationTrap {
    pub fn deserialize(&self, buf: &mut Buffer) {
        let read = buf.read_element();
        if read != *self as u8 {
            panic!(
                "SerializationTrap trapped!: found: {:?}, expected: {:?}",
                Self::from_u8(read),
                self
            );
        }
    }

    pub fn required_length() -> usize {
        1
    }

    pub fn try_deserialize(&self, buf: &mut Buffer) -> Result<(), SerializationError> {
        let read = buf.read_element();
        if read != *self as u8 {
            Err(SerializationError::SerializeTrap {
                found: Self::from_u8(read),
                expected: *self,
            })
        } else {
            Ok(())
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::String,
            1 => Self::HashMap,
            2 => Self::Vec,
            3 => Self::Option,
            4 => Self::Item,
            5 => Self::Block,
            6 => Self::Chunk,
            7 => Self::World,
            8 => Self::Time,
            _ => Self::Unknown,
        }
    }
}

impl Serialize for Box<dyn Item> {
    fn required_length(&self) -> usize {
        self.identifier().required_length()
            + u32::required_length(&0)
            + Item::required_length(&**self)
            + SerializationTrap::required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.reserve(self.required_length());
        SerializationTrap::Item.serialize(buf);
        self.identifier().serialize(buf);
        self.metadata().serialize(buf);
        Item::serialize(&**self, buf);
    }
}

impl Deserialize for Box<dyn Item> {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Item.try_deserialize(buf)?;
        let mut item = match get_item_by_id(Identifier::try_deserialize(buf)?) {
            None => return Err(SerializationError::InvalidData),
            Some(item) => item.clone_item(),
        };
        item.set_metadata(u32::try_deserialize(buf)?);
        Item::try_deserialize(&mut *item, buf)?;
        Ok(item)
    }
}

impl Serialize for Box<dyn Block> {
    fn required_length(&self) -> usize {
        if self.identifier() != *BLOCK_EMPTY {
            SerializationTrap::required_length()
                + bool::required_length(&false)
                + self.identifier().required_length()
                + Block::required_length(&**self)
        } else {
            SerializationTrap::required_length() + bool::required_length(&false)
        }
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Block.serialize(buf);
        (self.identifier() == *BLOCK_EMPTY).serialize(buf);
        if self.identifier() != *BLOCK_EMPTY {
            self.identifier().serialize(buf);
            Block::serialize(&**self, buf);
        }
    }
}

impl Deserialize for Box<dyn Block> {
    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Block.try_deserialize(buf)?;
        let is_empty = bool::deserialize(buf);
        if is_empty {
            Ok(empty_block().clone_block())
        } else {
            let ident = Identifier::try_deserialize(buf)?;
            let mut blk = match get_block_by_id(ident) {
                Some(v) => v.clone_block(),
                None => return Err(SerializationError::InvalidData),
            };
            Block::try_deserialize(&mut *blk, buf)?;
            Ok(blk)
        }
    }
}

impl<T: Serialize, K: Serialize> Serialize for (T, K) {
    fn required_length(&self) -> usize {
        self.0.required_length() + self.1.required_length()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        self.0.serialize(buf);
        self.1.serialize(buf);
    }
}

impl<T: Deserialize, K: Deserialize> Deserialize for (T, K) {
    fn deserialize(buf: &mut Buffer) -> Self {
        let a = T::deserialize(buf);
        let b = K::deserialize(buf);
        (a, b)
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        let a = T::try_deserialize(buf)?;
        let b = K::try_deserialize(buf)?;
        Ok((a, b))
    }
}

impl<T: Serialize, const N: usize> Serialize for [T; N] {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.as_ref().serialize(buf);
    }

    fn required_length(&self) -> usize {
        self.as_ref().required_length()
    }
}

impl<T: Deserialize, const N: usize> Deserialize for [T; N] {
    fn deserialize(buf: &mut Buffer) -> Self {
        <[T; N]>::try_from(<Vec<T>>::deserialize(buf))
            .ok()
            .expect("Not enough items to deserialize [T; N]!")
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        <[T; N]>::try_from(<Vec<T>>::try_deserialize(buf)?)
            .map_err(|_| SerializationError::InvalidData)
    }
}

impl Serialize for SystemTime {
    fn required_length(&self) -> usize {
        SerializationTrap::required_length() + u64::required_length(&0)
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        SerializationTrap::Time.serialize(buf);
        self.duration_since(UNIX_EPOCH)
            .expect("Time went backwards ftw")
            .as_secs()
            .serialize(buf);
    }
}

impl Deserialize for SystemTime {
    fn deserialize(buf: &mut Buffer) -> Self {
        SerializationTrap::Time.deserialize(buf);
        UNIX_EPOCH.add(Duration::new(u64::deserialize(buf), 0))
    }

    fn try_deserialize(buf: &mut Buffer) -> Result<Self, SerializationError> {
        SerializationTrap::Time.try_deserialize(buf)?;
        Ok(UNIX_EPOCH.add(Duration::new(u64::try_deserialize(buf)?, 0)))
    }
}

pub trait Serializable: Serialize + Deserialize {}
impl<T: Serialize + Deserialize> Serializable for T {}

const SIGNATURE: &[u8] = b"PN2S_SAV";

pub fn save_game(world: &World, cfg: &GameConfig, file: String) -> std::io::Result<usize> {
    let mut buf: Vec<u8> = Vec::with_capacity(4096);

    // PN2S_SAV: signature
    buf.extend(SIGNATURE);
    // save time
    SystemTime::now().serialize(&mut buf);

    // save world
    world.serialize(&mut buf);

    // save player inventory
    cfg.inventory.serialize(&mut buf);

    let len = buf.len();
    println!("Save Size: {} bytes", len);
    std::fs::write(file, buf)?;
    Ok(len)
}

pub fn load_game(file: String) -> Result<(World, GameConfig, SystemTime), SerializationError> {
    let mut buf = std::fs::read(file)
        .map(|bytes| Buffer::new(bytes))
        .map_err(|e| SerializationError::Io(e))?;
    if buf.len() < 8 {
        return Err(SerializationError::InvalidData);
    }
    if buf.read_elements(8) != SIGNATURE {
        return Err(SerializationError::InvalidData);
    }

    // save time
    let time = SystemTime::try_deserialize(&mut buf)?;

    // world
    let world = World::try_deserialize(&mut buf)?;

    // config
    let mut config: GameConfig = GameConfig::default();
    config.inventory = Inventory::deserialize(&mut buf);

    if buf.len() < 1 {
        return Err(SerializationError::InvalidData);
    }

    Ok((world, config, time))
}
