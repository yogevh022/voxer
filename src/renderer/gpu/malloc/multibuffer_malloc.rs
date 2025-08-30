use super::VirtualMalloc;
use super::virtual_malloc::{MallocError, SimpleAllocation, SimpleAllocationRequest};
use std::array;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct MultiBufferAllocationRequest {
    pub buffer_index: usize,
    pub size: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct MultiBufferAllocation {
    pub buffer_index: usize,
    pub offset: usize,
}

pub struct VMallocMultiBuffer<M: VirtualMalloc, const N: usize> {
    virtual_buffers: [M; N],
}

impl<M: VirtualMalloc, const N: usize> VirtualMalloc for VMallocMultiBuffer<M, N>
where
    M: VirtualMalloc<AllocationRequest = SimpleAllocationRequest, Allocation = SimpleAllocation>,
{
    type AllocationRequest = MultiBufferAllocationRequest;
    type Allocation = MultiBufferAllocation;
    fn new(buffer_size: usize, buffer_offset: usize) -> Self {
        Self {
            virtual_buffers: array::from_fn(|_| M::new(buffer_size, buffer_offset)),
        }
    }

    fn alloc(
        &mut self,
        allocation_request: MultiBufferAllocationRequest,
    ) -> Result<Self::Allocation, MallocError> {
        let inner_request = M::AllocationRequest {
            size: allocation_request.size,
        };
        let inner_allocation =
            self.virtual_buffers[allocation_request.buffer_index].alloc(inner_request)?;
        Ok(Self::Allocation {
            buffer_index: allocation_request.buffer_index,
            offset: inner_allocation.offset,
        })
    }

    fn free(&mut self, allocation: Self::Allocation) -> Result<(), MallocError> {
        let buff = self
            .virtual_buffers
            .get_mut(allocation.buffer_index)
            .unwrap();

        let inner_allocation = SimpleAllocation {
            offset: allocation.offset,
        };
        buff.free(inner_allocation)?;
        Ok(())
    }

    fn clear(&mut self) {
        for mesh_malloc in self.virtual_buffers.iter_mut() {
            mesh_malloc.clear();
        }
    }
}

impl<M: VirtualMalloc, const N: usize> VMallocMultiBuffer<M, N> {
    pub fn buffer_size(&self) -> usize {
        self.virtual_buffers.first().unwrap().total_size()
    }
}

impl<M: VirtualMalloc, const N: usize> Display for VMallocMultiBuffer<M, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const BAR_SIZE: usize = 30;
        let mut debug_display = String::new();
        for (i, b) in self.virtual_buffers.iter().enumerate() {
            let width_ratio = b.total_size() / BAR_SIZE;
            debug_display.push_str(format!("buffer {:2}: [", i).as_str());
            debug_display.push_str(&*"#".repeat(b.used_size() / width_ratio));
            debug_display.push_str(&*" ".repeat(BAR_SIZE - b.used_size() / width_ratio));
            debug_display.push_str(&format!(
                "] ab: {:2}, ub: {:2}\n\n",
                b.available_count(),
                b.used_count()
            ));
        }
        write!(f, "{}", debug_display)
    }
}
