extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use crate::world::generation::{WorldConfig, generate_chunk};
use crate::world::{ServerWorld, ServerWorldConfig};
use glam::IVec3;
use rustc_hash::FxHashMap;
use std::hint::black_box;
use voxer_network;
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;
use crate::compute::array::Array3D;
use crate::world::types::Chunk;

const SIMULATION_AND_RENDER_DISTANCE: usize = 8; // fixme temp location

fn run_app() {
    let mut server = ServerWorld::new(ServerWorldConfig {
        seed: 0,
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
    });
    let voxer_engine = vtypes::Voxer::default();
    let scene = vtypes::Scene {
        objects: vec![VObject::Camera(CameraController::with_sensitivity(0.01))],
    };

    server.start_session();
    let mut app = app::App::new(voxer_engine, server, scene);

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    // run_app();
    debug();
}

fn debug() {
    use crate::renderer::gpu::GPUVoxelChunk;
    use glam::IVec3;
    use smallhash;
    use std::hint::black_box;
    use std::time::Instant;

    const CHUNK_COUNT: i32 = 3;

    let world_config = WorldConfig {
        seed: 0,
        noise_scale: 0.3,
        simulation_distance: 16,
    };
    let sample_pos = IVec3::new(1, 1, 1);
    let mut chunks_map: FxHashMap<IVec3, Chunk> = FxHashMap::default();
    let exc = true;
    if exc {
        let chunk_pos = sample_pos;
        let chunk = generate_chunk(world_config, chunk_pos);
        chunks_map.insert(chunk_pos, chunk);
    } else {
        for x in 0..CHUNK_COUNT {
            for y in 0..CHUNK_COUNT {
                for z in 0..CHUNK_COUNT {
                    let chunk_pos = IVec3::new(x, y, z);
                    let chunk = generate_chunk(world_config, chunk_pos);
                    chunks_map.insert(chunk_pos, chunk);
                }
            }
        }
    }


    let adj_blocks = Array3D(compute::chunk::get_adj_blocks(sample_pos, &chunks_map));
    chunks_map.get_mut(&sample_pos).map(|chunk| {
        let fc = Some(compute::chunk::face_count(&chunk.blocks, &adj_blocks));
        dbg!(fc);
    });
}
