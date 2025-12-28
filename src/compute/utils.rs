use rustc_hash::FxHashMap;

#[inline(always)]
pub fn fxmap_with_capacity<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, Default::default())
}


#[inline(always)]
pub fn free_ptr<'a, 'b, T>(mut_ref: &'a mut T) -> &'b mut T
where
    'b: 'a,
{
    unsafe { &mut *(mut_ref as *mut T) }
}
