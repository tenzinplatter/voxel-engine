use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub(crate) struct TrackedHashMap<K, V> {
    inner: HashMap<K, V>,
    dirty_keys: HashSet<K>,
    first_dirty_check: bool,
}

impl<K: Eq + Hash + Clone, V> FromIterator<(K, V)> for TrackedHashMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let inner = iter.into_iter().collect();
        Self {
            inner,
            dirty_keys: HashSet::new(),
            first_dirty_check: true,
        }
    }
}

impl<K: Eq + Hash + Clone, V> TrackedHashMap<K, V> {
    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.dirty_keys.insert(key.clone());
        self.inner.insert(key, value)
    }

    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        self.dirty_keys.insert(key.clone());
        self.inner.remove(key)
    }

    pub(crate) fn dirty(&self) -> Box<dyn Iterator<Item = &K> + '_> {
        if self.first_dirty_check {
            Box::new(self.inner.keys())
        } else {
            Box::new(self.dirty_keys.iter())
        }
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub(crate) fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.inner.iter()
    }

    pub(crate) fn values(&self) -> std::collections::hash_map::Values<'_, K, V> {
        self.inner.values()
    }

    pub(crate) fn end_frame(&mut self) {
        self.dirty_keys.clear();
        self.first_dirty_check = false;
    }
}
