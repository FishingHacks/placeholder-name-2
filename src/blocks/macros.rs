#[macro_export]
macro_rules! block_impl_details {
    ($name: ident) => {
        #[derive(Clone)]
        pub struct $name;
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                Box::new(self.clone())
            }
        }
        derive_as_any!($name);
    };
    ($name: ident, $clone_fn: block) => {
        pub struct $name;
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };
    ($name: ident, $($y:ty),*) => {
        #[derive(Clone)]
        pub struct $name($($y),*);
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn crate::blocks::Block> {
                Box::new(self.clone())
            }
        }
        crate::derive_as_any!($name);
    };
    ($name: ident, $clone_fn: expr, $($y:ty),*) => {
        pub struct $name($($y),*);
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };

    (default $name: ident) => {
        #[derive(Clone, Default)]
        pub struct $name;
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                Box::new(self.clone())
            }
        }
        derive_as_any!($name);
    };
    (default $name: ident, $clone_fn: block) => {
        #[derive(Default)]
        pub struct $name;
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };
    (default $name: ident, $($y:ty),*) => {
        #[derive(Clone, Default)]
        pub struct $name($($y),*);
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                Box::new(self.clone())
            }
        }
        derive_as_any!($name);
    };
    (default $name: ident, $clone_fn: expr, $($y:ty),*) => {
        #[derive(Default)]
        pub struct $name($($y),*);
        impl crate::blocks::BlockImplDetails for $name {
            fn clone_block(&self) -> Box<dyn Block> {
                $clone_fn(self)
            }
        }
        derive_as_any!($name);
    };
}

#[macro_export]
macro_rules! block_impl_details_with_timer {
    ($name: ident, $duration: expr) => {
        block_impl_details!($name, std::time::Instant);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $clone_fn: block) => {
        block_impl_details!($name, $clone_fn, std::time::Instant);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $($y:ty),*) => {
        $crate::block_impl_details!($name, std::time::Instant, $($y),*);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    ($name: ident, $duration: expr, $clone_fn: expr, $($y:ty),*) => {
        block_impl_details!($name, {$clone_fn}, std::time::Instant, $($y),*);
        block_impl_details_with_timer!(__ $name, $duration);
    };
    (__ $name: ident, $duration: expr) => {
        impl $name {
            fn can_do_work(&self) -> bool {
                if std::time::Instant::now().saturating_duration_since(self.0).as_millis() >= ($duration as u128) {
                    true
                } else {
                    false
                }
            }

            #[allow(dead_code)]
            fn duration_lerp_value(&self) -> f32 {
                ((std::time::Instant::now().saturating_duration_since(self.0).as_millis().min($duration as u128)) as f32 / $duration as f32).min(1.0)
            }
        }
    };
}

#[macro_export]
macro_rules! reset_timer {
    ($self: expr) => {
        $self.0 = std::time::Instant::now();
    };
}

#[macro_export]
macro_rules! register_blocks {
    ($($block: ty),*) => {
        $(
            register_block(Box::new(<$block>::default()));
        )*
    };
}

#[macro_export]
macro_rules! empty_serializable {
    () => {
        fn serialize(&self, _: &mut Vec<u8>) {}
        fn try_deserialize(&mut self, _: &mut Buffer) -> Result<(), SerializationError> {
            Ok(())
        }
        fn required_length(&self) -> usize {
            0
        }
    };
}

#[macro_export]
macro_rules! simple_single_item_serializable {
    ($index: tt) => {
        fn try_deserialize(
            &mut self,
            buf: &mut crate::serialization::Buffer,
        ) -> Result<(), crate::serialization::SerializationError> {
            use crate::serialization::Deserialize;
            let item = <Option<Box<dyn crate::items::Item>>>::try_deserialize(buf)?;
            self.$index.resize(1);
            *self.$index.get_item_mut(0) = item;
            Ok(())
        }
        fn required_length(&self) -> usize {
            use crate::serialization::Serialize;
            self.$index.get_item(0).required_length()
        }
        fn serialize(&self, buf: &mut Vec<u8>) {
            use crate::serialization::Serialize;
            self.$index.get_item(0).serialize(buf)
        }
    };
}

#[macro_export]
macro_rules! simple_single_item_direction_serializable {
    ($item: tt, $direction: tt) => {
        fn try_deserialize(
            &mut self,
            buf: &mut crate::serialization::Buffer,
        ) -> Result<(), crate::serialization::SerializationError> {
            use crate::serialization::Deserialize;
            self.$item.resize(1);
            *self.$item.get_item_mut(0) = Deserialize::try_deserialize(buf)?;
            self.$direction = Deserialize::try_deserialize(buf)?;
            Ok(())
        }
        fn required_length(&self) -> usize {
            use crate::serialization::Serialize;
            self.$item.get_item(0).required_length() + self.$direction.required_length()
        }
        fn serialize(&self, buf: &mut Vec<u8>) {
            use crate::serialization::Serialize;
            self.$item.get_item(0).serialize(buf);
            self.$direction.serialize(buf);
        }
    };
}

#[macro_export]
macro_rules! step_size {
    ($dir: expr, $w: expr, $h: expr) => {
        if matches!($dir, Direction::North | Direction::South) {
            $h
        } else {
            $w
        }
    };
}