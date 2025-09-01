use std::fmt::Debug;

#[derive(Debug)]
pub enum MallocError {
    OutOfMemory,
    InvalidAllocation,
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleAllocationRequest {
    pub size: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleAllocation {
    pub offset: usize,
}

pub trait VirtualMalloc {
    type AllocationRequest: Copy + Clone + Debug;
    type Allocation: Copy + Clone + Debug;
    fn new(arena_size: usize, arena_offset: usize) -> Self;
    fn alloc(
        &mut self,
        alloc_request: Self::AllocationRequest,
    ) -> Result<Self::Allocation, MallocError>;
    fn free(
        &mut self,
        allocation: Self::Allocation,
    ) -> Result<(), MallocError>;
    fn clear(&mut self);
    fn total_size(&self) -> usize {
        unimplemented!()
    }
    fn available_size(&self) -> usize {
        unimplemented!()
    }
    fn available_count(&self) -> usize {
        unimplemented!()
    }
    fn used_size(&self) -> usize {
        unimplemented!()
    }
    fn used_count(&self) -> usize {
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
