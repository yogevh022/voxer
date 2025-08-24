use std::fmt::Debug;

#[derive(Debug)]
pub enum MallocError {
    OutOfMemory,
    InvalidAllocation,
}

pub trait VirtualMalloc<T, const L: usize>
where
    T: Copy + Clone + Debug,
{
    fn new(size: usize, offset: usize) -> Self;
    fn alloc(&mut self, request: [T; L]) -> Result<[T; L], MallocError>;
    fn free(&mut self, allocation: [T; L]) -> Result<(), MallocError>;
    fn total_free(&self) -> usize {
        unimplemented!()
    }
    fn free_count(&self) -> usize {
        unimplemented!()
    }
    fn total_used(&self) -> usize {
        unimplemented!()
    }
    fn used_count(&self) -> usize {
        unimplemented!()
    }

    fn debug(&self) {
        unimplemented!()
    }
}

// #[test]
// fn test_virtual_alloc() {
//     use crate::compute;
//     use crate::renderer::gpu::malloc::VMallocFirstFit;
//     use rand::Rng;
// 
//     let mut rng = rand::thread_rng();
//     let mut malloc = VMallocFirstFit::new(compute::GIB, 8);
//     let mut allocations: Vec<usize> = Vec::new();
//     for i in 0..100_000 {
//         if allocations.is_empty() || rng.gen_bool(0.50) {
//             let allocation = malloc
//                 .alloc((0u128, (rng.gen_range(0.1..1.0) * 50_000f64) as usize))
//                 .unwrap();
//             allocations.push(allocation);
//         } else {
//             let allocation =
//                 allocations.remove((rng.gen_range(0.1..1.0) * allocations.len() as f64) as usize);
//             malloc.free(allocation).unwrap();
//         }
//     }
// 
//     println!("{:?}", malloc);
// }
