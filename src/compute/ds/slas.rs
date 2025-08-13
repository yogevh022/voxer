use slab::Slab;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub struct Slas<T>
where
    T: Eq + Hash,
{
    slab: Slab<T>,
    index_map: HashMap<u64, usize>,
}

impl<T> Slas<T>
where
    T: Eq + Hash + std::clone::Clone,
{
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let value_hash = hasher.finish();
        self.index_map.contains_key(&value_hash)
    }

    pub fn iget(&self, idx: usize) -> Option<&T> {
        self.slab.get(idx)
    }

    pub fn insert(&mut self, value: T) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let value_hash = hasher.finish();
        if let Some(idx) = self.index_map.get(&value_hash) {
            self.slab[*idx] = value;
            return *idx;
        }
        let idx = self.slab.insert(value);
        self.index_map.insert(value_hash, idx);
        idx
    }

    pub fn remove(&mut self, value: T) -> Option<usize> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let value_hash = hasher.finish();
        self.index_map.remove(&value_hash).and_then(|idx| {
            self.slab.remove(idx);
            Some(idx)
        })
    }

    pub fn retain(&mut self, f: impl Fn(&T) -> bool) {
        self.index_map
            .retain(|_, idx| f(self.slab.get(*idx).unwrap()));
        self.slab.retain(|_, value| f(value));
    }

    pub fn extend(&mut self, values: impl IntoIterator<Item = T>) {
        for value in values.into_iter() {
            self.insert(value);
        }
    }

    pub fn extend_with_indices(&mut self, values: impl IntoIterator<Item = T>) -> Vec<(usize, &T)> {
        let indices: Vec<_> = values.into_iter().map(|value| self.insert(value)).collect();
        indices
            .into_iter()
            .map(|idx| (idx, self.slab.get(idx).unwrap()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slab.iter().map(|(_, value)| value)
    }

    pub fn symmetric_difference<'other>(&'other self, other: &'other HashSet<T>) -> Vec<&'other T> {
        self.iter()
            .filter(|v| !other.contains(v))
            .chain(other.iter().filter(|v| !self.contains(v)))
            .collect()
    }
}
