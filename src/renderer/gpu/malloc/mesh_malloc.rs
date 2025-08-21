use crate::renderer::gpu::VirtualMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::{MallocError, VirtualMallocType};

pub struct MeshVMalloc<A: VirtualMalloc> {
    pub vertex_malloc: A,
    pub index_malloc: A,
}

impl<A: VirtualMalloc> VirtualMallocType for MeshVMalloc<A> {
    type Allocation = (A::Allocation, A::Allocation);
}

impl<A: VirtualMalloc> MeshVMalloc<A> {
    pub fn new(
        vertex_size: usize,
        vertex_offset: usize,
        index_size: usize,
        index_offset: usize,
    ) -> Self {
        Self {
            vertex_malloc: A::new(vertex_size, vertex_offset),
            index_malloc: A::new(index_size, index_offset),
        }
    }

    pub fn alloc(
        &mut self,
        vertex_size: usize,
        index_size: usize,
    ) -> Result<<MeshVMalloc<A> as VirtualMallocType>::Allocation, MallocError> {
        let v_alloc = self.vertex_malloc.alloc(vertex_size)?;
        match self.index_malloc.alloc(index_size) {
            Err(e) => {
                self.vertex_malloc.free(v_alloc)?;
                Err(e)
            }
            Ok(i_alloc) => Ok((v_alloc, i_alloc)),
        }
    }

    pub fn free(
        &mut self,
        vertex_offset: A::Allocation,
        index_offset: A::Allocation,
    ) -> Result<(), MallocError> {
        self.vertex_malloc.free(vertex_offset)?;
        self.index_malloc.free(index_offset)?;
        Ok(())
    }
}
