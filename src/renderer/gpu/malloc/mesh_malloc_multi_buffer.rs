use crate::renderer::gpu::malloc::mesh_malloc::MeshVMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use crate::renderer::gpu::{VMallocFirstFit, VirtualMalloc};
use bytemuck::{NoUninit, Pod, Zeroable};
use std::array;
use std::fmt::Debug;

#[repr(C, align(16))]
#[derive(Copy, Debug, Clone, Pod, Zeroable)]
pub struct MultiBufferMeshAllocation {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_size: u32,
    pub index_size: u32,
}

#[derive(Debug)]
pub struct MultiBufferMeshAllocationRequest {
    pub id: u128,
    pub vertex_size: usize,
    pub index_size: usize,
}

pub struct MeshVMallocMultiBuffer<const N: usize> {
    virtual_buffers: [MeshVMalloc; N],
}

impl<const N: usize> MeshVMallocMultiBuffer<N> {
    pub(crate) fn new(size: usize, offset: usize) -> Self {
        Self {
            virtual_buffers: array::from_fn(|_| MeshVMalloc::new(size, offset)),
        }
    }

    pub(crate) fn alloc(&mut self, allocation_request: MultiBufferMeshAllocationRequest) -> Result<(usize, MultiBufferMeshAllocation), MallocError> {
        let buffer_index = (allocation_request.id % N as u128) as usize;
        let allocation = self.virtual_buffers[buffer_index]
            .alloc((allocation_request.vertex_size, allocation_request.index_size))
            .ok()
            .ok_or(MallocError::OutOfMemory)?;
        Ok((
            buffer_index,
            MultiBufferMeshAllocation {
                vertex_offset: allocation.0 as u32,
                vertex_size: allocation_request.vertex_size as u32,
                index_offset: allocation.1 as u32,
                index_size: allocation_request.index_size as u32,
            },
        ))
    }

    pub(crate) fn free(&mut self, allocation: (usize, MultiBufferMeshAllocation)) -> Result<(), MallocError> {
        let buff = self.virtual_buffers.get_mut(allocation.0).unwrap();
        buff.free((
            allocation.1.vertex_offset as usize,
            allocation.1.index_offset as usize,
        ))?;
        Ok(())
    }
}
