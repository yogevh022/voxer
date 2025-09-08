mod firstfit_malloc;
mod multibuffer_malloc;
mod virtual_malloc;

pub use firstfit_malloc::VMallocFirstFit;
pub use multibuffer_malloc::{
    VMallocMultiBuffer, MultiBufferAllocationRequest, MultiBufferAllocation
};
pub use virtual_malloc::VirtualMalloc;

#[derive(Debug, Clone, Copy)]
struct VirtualMemSlot {
    pub size: usize,
    pub prev_free: usize,    // available adjacent space previous to this slot
}