use slab::Slab;
use rustc_hash::FxHashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Slap<K, V>
where
    K: Hash + Eq,
{
    slab: Slab<V>,
    map: FxHashMap<K, usize>,
}

impl<K, V> Slap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            map: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> usize {
        let index = self.slab.insert(value);
        self.map
            .insert(key, index)
            .map(|old_index| self.slab.remove(old_index));
        index
    }

    pub fn remove(&mut self, key: &K) -> Option<(usize, V)> {
        self.map
            .remove(key)
            .and_then(|index| Some((index, self.slab.remove(index))))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map
            .iter()
            .map(|(key, &index)| (key, self.slab.get(index).unwrap()))
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).and_then(|&index| self.slab.get(index))
    }
    
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn slab(&self) -> &Slab<V> {
        &self.slab
    }

    pub fn hmap(&self) -> &FxHashMap<K, usize> {
        &self.map
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
