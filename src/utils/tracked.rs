use std::ops::{Deref, DerefMut};

pub struct Tracked<T> {
    value: T,
    dirty: bool,
}

impl<T> Default for Tracked<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: T::default(),
            dirty: true,
        }
    }
}

impl<T> DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.value
    }
}

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Tracked<T> {
    pub fn new(value: T) -> Self {
        Self { value, dirty: true }
    }

    pub fn set(&mut self, new: T) {
        self.dirty = true;
        self.value = new;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
