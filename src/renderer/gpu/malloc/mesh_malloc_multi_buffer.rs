use crate::renderer::gpu::VirtualMalloc;
use crate::renderer::gpu::malloc::mesh_malloc::MeshVMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use std::array;

pub struct MultiBufferAllocationRequest {
    pub id: u128,
    pub vertex_size: usize,
    pub index_size: usize,
}

pub struct MultiBufferMeshAllocation<A: VirtualMalloc> {
    pub buffer_index: usize,
    pub vertex_offset: A::Allocation,
    pub index_offset: A::Allocation,
    pub vertex_size: usize,
    pub index_size: usize,
}

pub struct MeshVMallocMultiBuffer<A: VirtualMalloc, const N: usize> {
    virtual_buffers: [MeshVMalloc<A>; N],
}

impl<A: VirtualMalloc<AllocationRequest = usize>, const N: usize> VirtualMalloc
    for MeshVMallocMultiBuffer<A, N>
{
    type Allocation = MultiBufferMeshAllocation<A>;
    type AllocationRequest = MultiBufferAllocationRequest;
    fn new(size: usize, offset: usize) -> Self {
        Self {
            virtual_buffers: array::from_fn(|_| MeshVMalloc::new(size, offset)),
        }
    }
    fn alloc(&mut self, req: Self::AllocationRequest) -> Result<Self::Allocation, MallocError> {
        let buffer_index = (req.id % N as u128) as usize;
        let allocation = self.virtual_buffers[buffer_index]
            .alloc((req.vertex_size, req.index_size))
            .ok()
            .ok_or(MallocError::OutOfMemory)?;
        Ok(MultiBufferMeshAllocation {
            buffer_index,
            vertex_offset: allocation.0,
            vertex_size: req.vertex_size,
            index_offset: allocation.1,
            index_size: req.index_size,
        })
    }

    fn free(&mut self, allocation: Self::Allocation) -> Result<(), MallocError> {
        let buff = self
            .virtual_buffers
            .get_mut(allocation.buffer_index)
            .unwrap();
        buff.free((allocation.vertex_offset, allocation.index_offset))?;
        Ok(())
    }
}
