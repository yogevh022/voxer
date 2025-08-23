use std::fmt::Debug;

#[derive(Debug)]
pub enum MallocError {
    OutOfMemory,
    InvalidAllocation,
}

pub trait VirtualMalloc {
    type Allocation: Copy + Clone + Debug;
    type AllocationRequest: Copy + Clone + Debug;
    fn new(size: usize, offset: usize) -> Self;
    fn alloc(&mut self, size: Self::AllocationRequest) -> Result<Self::Allocation, MallocError>;
    fn free(&mut self, alloc_index: Self::Allocation) -> Result<(), MallocError>;
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

#[test]
fn test_virtual_alloc() {
    use crate::compute;
    use crate::renderer::gpu::malloc::VMallocFirstFit;
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut malloc = VMallocFirstFit::new(compute::GIB, 8);
    let mut allocations: Vec<usize> = Vec::new();
    for i in 0..100_000 {
        if allocations.is_empty() || rng.gen_bool(0.50) {
            let allocation = malloc
                .alloc((rng.gen_range(0.1..1.0) * 50_000f64) as usize)
                .unwrap();
            allocations.push(allocation);
        } else {
            let allocation =
                allocations.remove((rng.gen_range(0.1..1.0) * allocations.len() as f64) as usize);
            malloc.free(allocation).unwrap();
        }
    }

    println!("{:?}", malloc);
}
