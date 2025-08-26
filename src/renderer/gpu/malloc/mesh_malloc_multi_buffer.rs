use crate::renderer::gpu::malloc::mesh_malloc::MeshVMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use bytemuck::{Pod, Zeroable};
use std::array;
use std::fmt::Debug;
use std::io::Write;

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
    pub buffer_index: usize,
    pub vertex_size: usize,
    pub index_size: usize,
}

pub struct MeshVMallocMultiBuffer<const N: usize> {
    virtual_buffers: [MeshVMalloc; N],
}

impl<const N: usize> MeshVMallocMultiBuffer<N> {
    pub(crate) fn new(
        vertices_per_buffer: usize,
        indices_per_buffer: usize,
        offset: usize,
    ) -> Self {
        Self {
            virtual_buffers: array::from_fn(|_| {
                MeshVMalloc::new(vertices_per_buffer, indices_per_buffer, offset)
            }),
        }
    }

    pub(crate) fn alloc(
        &mut self,
        allocation_request: MultiBufferMeshAllocationRequest,
    ) -> Result<MultiBufferMeshAllocation, MallocError> {
        let allocation = self.virtual_buffers[allocation_request.buffer_index]
            .alloc((
                allocation_request.vertex_size,
                allocation_request.index_size,
            ))
            .ok()
            .ok_or(MallocError::OutOfMemory)?;
        Ok(MultiBufferMeshAllocation {
            vertex_offset: allocation.0 as u32,
            vertex_size: allocation_request.vertex_size as u32,
            index_offset: allocation.1 as u32,
            index_size: allocation_request.index_size as u32,
        })
    }

    pub(crate) fn free(
        &mut self,
        allocation: (usize, MultiBufferMeshAllocation),
    ) -> Result<(), MallocError> {
        let buff = self.virtual_buffers.get_mut(allocation.0).unwrap();
        buff.free((
            allocation.1.vertex_offset as usize,
            allocation.1.index_offset as usize,
        ))?;
        Ok(())
    }

    pub(crate) fn debug(&self) {
        const BAR_SIZE: usize = 30;
        let mut debug_display = String::new();
        for (i, b) in self.virtual_buffers.iter().enumerate() {
            let width_ratio = b.vertex_malloc.arena_size / BAR_SIZE;
            debug_display.push_str(format!("vertex {:2}: [", i).as_str());
            debug_display.push_str(&*"#".repeat(b.vertex_malloc.total_used() / width_ratio));
            debug_display
                .push_str(&*" ".repeat(BAR_SIZE - b.vertex_malloc.total_used() / width_ratio));
            debug_display.push_str("]\n");
            let width_ratio = b.index_malloc.arena_size / BAR_SIZE;
            debug_display.push_str(format!("index  {:2}: [", i).as_str());
            debug_display.push_str(&*"#".repeat(b.index_malloc.total_used() / width_ratio));
            debug_display
                .push_str(&*" ".repeat(BAR_SIZE - b.index_malloc.total_used() / width_ratio));
            debug_display.push_str("]\n\n");
        }
        print!("\x1B[2J\x1B[1;1H{}", debug_display); // the blob clears cli
        std::io::stdout().flush().unwrap();
    }
}
