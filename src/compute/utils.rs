use rustc_hash::FxHashMap;

pub fn fxmap_with_capacity<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, Default::default())
}