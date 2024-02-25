use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

#[derive(Default)]
pub struct InitializedData<T: Sync>(Cell<Option<T>>);

impl<T: Clone + Sync> Clone for InitializedData<T> {
    fn clone(&self) -> Self {
        match unsafe { &*self.0.as_ptr() } {
            Some(v) => Self(Cell::new(Some(v.clone()))),
            None => Self(Cell::new(None)),
        }
    }
}

impl<T: Sync> InitializedData<T> {
    pub const fn new() -> Self {
        Self(Cell::new(None))
    }

    pub fn init(&self, value: T) {
        self.0.set(Some(value))
    }

    pub fn is_init(&self) -> bool {
        unsafe { (&*self.0.as_ptr()).is_some() }
    }

    pub fn get<'a>(&'a self) -> &'a T {
        match unsafe { &*self.0.as_ptr() } {
            Some(v) => v,
            None => panic!("Tried accessing InitializedData before initialization"),
        }
    }

    pub unsafe fn get_mut<'a>(&'a self) -> &'a mut T {
        match unsafe { &mut *self.0.as_ptr() } {
            Some(v) => v,
            None => panic!("Tried accessing InitializedData before initialization"),
        }
    }
}

impl<T: Sync> Deref for InitializedData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: Sync> DerefMut for InitializedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0.get_mut() {
            Some(v) => v,
            None => panic!("Tried accessing InitializedData before initialization"),
        }
    }
}

unsafe impl<T: Sync> Sync for InitializedData<T> {}

impl <T: Default + Sync> InitializedData<T> {
    pub fn maybe_init_default(&self) {
        if !self.is_init() {
            self.init(Default::default());
        }
    }
}