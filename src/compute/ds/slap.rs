use slab::Slab;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Slap<K, V>
where
    K: Hash + Eq,
{
    slab: Slab<V>,
    map: HashMap<K, usize>,
}

impl<K, V> Slap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> usize {
        let index = self.slab.insert(value);
        self.map.insert(key, index);
        index
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map
            .remove(key)
            .and_then(|index| Some(self.slab.remove(index)))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map
            .iter()
            .map(|(key, &index)| (key, self.slab.get(index).unwrap()))
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).and_then(|&index| self.slab.get(index))
    }

    pub fn iget(&self, key: usize) -> Option<&V> {
        self.slab.get(key)
    }

    pub fn index_of(&self, key: &K) -> Option<&usize> {
        self.map.get(key)
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }
}
