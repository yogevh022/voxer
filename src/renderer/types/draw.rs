use bytemuck::{Pod, Zeroable};
use wgpu::wgt::DrawIndexedIndirectArgs;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct DrawIndexedIndirectArgsDX12 {
    pub args: DrawIndexedIndirectArgs,
    pub _padding: [u32; 3],
}