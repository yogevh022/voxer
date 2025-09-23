mod chunk_compute;
mod chunk_manager;
mod chunk_render;

use crate::renderer::DrawIndirectArgsDX12;
pub use chunk_manager::ChunkManager;
use std::collections::HashMap;

type BufferDrawArgs = HashMap<usize, DrawIndirectArgsDX12>;
