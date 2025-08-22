use crate::renderer::gpu::VirtualMalloc;
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use crate::renderer::{Index, Vertex};

pub struct MeshVMalloc<A: VirtualMalloc> {
    pub vertex_malloc: A,
    pub index_malloc: A,
}

impl<A: VirtualMalloc<AllocationRequest = usize>> VirtualMalloc for MeshVMalloc<A> {
    type Allocation = (A::Allocation, A::Allocation);
    type AllocationRequest = (usize, usize);
    fn new(size: usize, offset: usize) -> Self {
        Self {
            vertex_malloc: A::new(size / Vertex::size(), offset),
            index_malloc: A::new(size / size_of::<Index>(), offset),
        }
    }

    fn alloc(&mut self, size: Self::AllocationRequest) -> Result<Self::Allocation, MallocError> {
        let v_alloc = self.vertex_malloc.alloc(size.0)?;
        match self.index_malloc.alloc(size.1) {
            Err(e) => {
                self.vertex_malloc.free(v_alloc)?;
                Err(e)
            }
            Ok(i_alloc) => Ok((v_alloc, i_alloc)),
        }
    }

    fn free(&mut self, offset: Self::Allocation) -> Result<(), MallocError> {
        self.vertex_malloc.free(offset.0)?;
        self.index_malloc.free(offset.1)?;
        Ok(())
    }
}
