use crate::worldgen::types::Chunk;
use glam::IVec3;

#[derive(Debug)]
pub struct ChunkTasks<'a> {
    pub renderer_load: Vec<&'a IVec3>,
    pub generate_chunk: Vec<&'a IVec3>,
    pub generate_mesh: Vec<&'a IVec3>,
}

pub(crate) enum ChunkTaskKind {
    RendererLoad,
    GenerateChunk,
    GenerateMesh,
}

pub(crate) fn task_kind_for(chunk_opt: Option<&Chunk>) -> Option<ChunkTaskKind> {
    match chunk_opt {
        Some(chunk) => {
            match &chunk.mesh {
                Some(mesh) => {
                    if !mesh.vertices.is_empty() {
                        return Some(ChunkTaskKind::RendererLoad);
                    }
                    None // mesh exists but is air
                }
                None => Some(ChunkTaskKind::GenerateMesh), // mesh does not exist
            }
        }
        None => Some(ChunkTaskKind::GenerateChunk),
    }
}
