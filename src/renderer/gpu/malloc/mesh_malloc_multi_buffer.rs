use crate::renderer::gpu::malloc::mesh_malloc::MeshVMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use crate::renderer::gpu::{VMallocFirstFit, VirtualMalloc};
use bytemuck::{NoUninit, Pod, Zeroable};
use std::array;
use std::fmt::Debug;

#[derive(Copy, Debug, Clone)]
pub struct MultiBufferAllocationRequest {
    pub id: u128,
    pub vertex_size: usize,
    pub index_size: usize,
}

#[repr(C)]
#[derive(Copy, Debug, Clone, Pod, Zeroable)]
pub struct MultiBufferMeshAllocation {
    pub vertex_offset: <VMallocFirstFit as VirtualMalloc>::Allocation,
    pub index_offset: <VMallocFirstFit as VirtualMalloc>::Allocation,
    pub vertex_size: usize,
    pub index_size: usize,
}

pub struct MeshVMallocMultiBuffer<const N: usize> {
    virtual_buffers: [MeshVMalloc<VMallocFirstFit>; N],
}

impl<const N: usize> VirtualMalloc for MeshVMallocMultiBuffer<N> {
    type Allocation = (usize, MultiBufferMeshAllocation);
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
        Ok((
            buffer_index,
            MultiBufferMeshAllocation {
                vertex_offset: allocation.0,
                vertex_size: req.vertex_size,
                index_offset: allocation.1,
                index_size: req.index_size,
            },
        ))
    }

    fn free(&mut self, allocation: Self::Allocation) -> Result<(), MallocError> {
        let buff = self.virtual_buffers.get_mut(allocation.0).unwrap();
        buff.free((allocation.1.vertex_offset, allocation.1.index_offset))?;
        Ok(())
    }
}
