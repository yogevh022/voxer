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

#[repr(C, align(16))]
#[derive(Copy, Debug, Clone, Pod, Zeroable)]
pub struct MultiBufferMeshAllocation {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_size: u32,
    pub index_size: u32,
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
                vertex_offset: allocation.0 as u32,
                vertex_size: req.vertex_size as u32,
                index_offset: allocation.1 as u32,
                index_size: req.index_size as u32,
            },
        ))
    }

    fn free(&mut self, allocation: Self::Allocation) -> Result<(), MallocError> {
        let buff = self.virtual_buffers.get_mut(allocation.0).unwrap();
        buff.free((allocation.1.vertex_offset as usize, allocation.1.index_offset as usize))?;
        Ok(())
    }
}
