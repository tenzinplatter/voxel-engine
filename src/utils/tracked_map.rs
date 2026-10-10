use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub struct TrackedHashMap<K, V> {
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.dirty_keys.insert(key.clone());
        self.inner.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.dirty_keys.insert(key.clone());
        self.inner.remove(key)
    }

    pub fn dirty(&self) -> Box<dyn Iterator<Item = &K> + '_> {
        if self.first_dirty_check {
            Box::new(self.inner.keys())
        } else {
            Box::new(self.dirty_keys.iter())
        }
    }

    /// Peek without resetting
    pub fn is_dirty(&self) -> bool {
        !self.dirty_keys.is_empty() || self.first_dirty_check
    }

    // Delegate read-only methods directly
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.dirty_keys.insert(key.clone());
        self.inner.get_mut(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, K, V> {
        self.dirty_all();
        self.inner.iter_mut()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, K, V> {
        self.inner.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, K, V> {
        self.inner.values()
    }

    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, V> {
        self.dirty_all();
        self.inner.values_mut()
    }

    pub fn end_frame(&mut self) {
        self.dirty_keys.clear();
        self.first_dirty_check = false;
    }

    fn dirty_all(&mut self) {
        self.dirty_keys.extend(self.inner.keys().cloned());
    }
}

impl<K: Eq + Hash + Clone, V> Default for TrackedHashMap<K, V> {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            dirty_keys: HashSet::new(),
            first_dirty_check: true,
        }
    }
}

// Implement IntoIterator for convenient for-loops
impl<'a, K: Eq + Hash, V> IntoIterator for &'a TrackedHashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a mut TrackedHashMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
