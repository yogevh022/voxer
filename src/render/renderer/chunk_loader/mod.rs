use crate::render::Renderer;
use crate::utils;
use crate::worldgen::types::World;
use glam::{IVec3, Vec3};

fn get_relevant_chunk_positions(chunk_pos_f32: Vec3, radius: f32, world: &World) -> Vec<IVec3> {
    // filters out distant chunks, non existent chunks, and empty chunks
    utils::geo::discrete_points_within_sphere(chunk_pos_f32, radius)
        .into_iter()
        .filter(|chunk_pos| {
            !world
                .chunks
                .get(chunk_pos)
                .map(|chunk| chunk.mesh.vertices.is_empty())
                .unwrap_or(true)
        })
        .collect()
}

pub fn update_loaded_chunks(
    chunk_pos_f32: Vec3,
    radius: f32,
    world: &mut World,
    renderer: &mut Renderer,
) {
    let relevant_chunk_positions: Vec<IVec3> =
        get_relevant_chunk_positions(chunk_pos_f32, radius, world);

    let mut chunk_indexes_to_unload: Vec<usize> = world
        .loaded_chunks
        .iter()
        .enumerate()
        .filter(|(_, c_pos)| !relevant_chunk_positions.contains(*c_pos))
        .map(|(c_idx, _)| c_idx)
        .collect();

    let chunk_positions_to_load: Vec<&IVec3> = relevant_chunk_positions
        .iter()
        .filter(|c_pos| !world.loaded_chunks.contains(&c_pos))
        .collect();

    if !chunk_indexes_to_unload.is_empty() {
        chunk_indexes_to_unload.sort();
        for c_idx in chunk_indexes_to_unload.into_iter().rev() {
            // fixme find a better way to sync world loaded chunks with renderer loaded chunk buffers
            world.loaded_chunks.remove(c_idx);
            renderer.remove_buffer(c_idx);
        }
    }

    if !chunk_positions_to_load.is_empty() {
        for c_pos in chunk_positions_to_load.into_iter() {
            world.loaded_chunks.push(*c_pos);
            renderer.add_buffer(&world.chunks[c_pos].mesh);
        }
    }
}
