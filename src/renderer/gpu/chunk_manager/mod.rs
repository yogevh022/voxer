mod chunk_compute;
mod chunk_manager;
mod chunk_render;

pub use chunk_manager::ChunkManager;
use std::collections::HashMap;
use wgpu::wgt::DrawIndirectArgs;

type BufferDrawArgs = HashMap<usize, DrawIndirectArgs>;
