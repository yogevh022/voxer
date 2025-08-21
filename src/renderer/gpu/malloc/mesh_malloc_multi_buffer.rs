use crate::renderer::gpu::VirtualMalloc;
use crate::renderer::gpu::malloc::mesh_malloc::MeshVMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::{MallocError, VirtualMallocType};
use std::array;

pub struct MultiBufferAllocationRequest {
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

impl<A: VirtualMalloc, const N: usize> MeshVMallocMultiBuffer<A, N> {
    pub fn new(vertex_size: usize, index_size: usize, offset: usize) -> Self {
        Self {
            virtual_buffers: array::from_fn(|i| {
                MeshVMalloc::new(vertex_size, offset, index_size, offset)
            }),
        }
    }
    pub fn alloc(
        &mut self,
        req: MultiBufferAllocationRequest,
    ) -> Result<MultiBufferMeshAllocation<A>, MallocError> {
        for (i, buffer_malloc) in self.virtual_buffers.iter_mut().enumerate() {
            let Ok((vertex_offset, index_offset)) =
                buffer_malloc.alloc(req.vertex_size, req.index_size)
            else {
                continue;
            };
            return Ok(MultiBufferMeshAllocation {
                buffer_index: i,
                vertex_offset,
                vertex_size: req.vertex_size,
                index_offset,
                index_size: req.index_size,
            });
        }
        Err(MallocError::OutOfMemory)
    }

    pub fn free(&mut self, allocation: MultiBufferMeshAllocation<A>) -> Result<(), MallocError> {
        let buff = self
            .virtual_buffers
            .get_mut(allocation.buffer_index)
            .unwrap();
        buff.free(allocation.vertex_offset, allocation.index_offset)?;
        Ok(())
    }
}
