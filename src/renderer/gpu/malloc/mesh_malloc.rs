use crate::renderer::gpu::{VMallocFirstFit, VirtualMalloc};
use crate::renderer::gpu::malloc::virtual_malloc::MallocError;
use crate::renderer::{Index, Vertex};

pub struct MeshVMalloc {
    pub vertex_malloc: VMallocFirstFit,
    pub index_malloc: VMallocFirstFit,
}

impl MeshVMalloc {
    pub(crate) fn new(size: usize, offset: usize) -> Self {
        Self {
            vertex_malloc: VMallocFirstFit::new(size / Vertex::size(), offset),
            index_malloc: VMallocFirstFit::new(size / size_of::<Index>(), offset),
        }
    }

    pub(crate) fn alloc(&mut self, requested_allocation: (usize, usize)) -> Result<(usize, usize), MallocError> {
        let v_alloc = self.vertex_malloc.alloc(requested_allocation.0)?;
        match self.index_malloc.alloc(requested_allocation.1) {
            Err(e) => {
                self.vertex_malloc.free(v_alloc)?;
                Err(e)
            }
            Ok(i_alloc) => Ok((v_alloc, i_alloc)),
        }
    }

    pub(crate) fn free(&mut self, allocation: (usize, usize)) -> Result<(), MallocError> {
        self.vertex_malloc.free(allocation.0)?;
        self.index_malloc.free(allocation.1)?;
        Ok(())
    }
}
