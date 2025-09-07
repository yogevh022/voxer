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