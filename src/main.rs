extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use crate::world::generation::WorldConfig;
use crate::world::{ClientWorldConfig, ServerWorld, ServerWorldConfig};
use voxer_network;
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

fn run_app() {
    const SIMULATION_AND_RENDER_DISTANCE: usize = 16;

    let server_config = ServerWorldConfig {
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
        world_config: WorldConfig {
            seed: 0,
            noise_scale: 0.03,
        },
    };

    let client_config = ClientWorldConfig {
        render_distance: SIMULATION_AND_RENDER_DISTANCE,
    };

    let mut server = ServerWorld::new(server_config);
    let voxer_engine = vtypes::Voxer::default();
    let scene = vtypes::Scene {
        objects: vec![VObject::Camera(CameraController::with_sensitivity(0.01))],
    };

    server.start_session();
    let mut app = app::App::new(voxer_engine, server, scene, client_config);

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    run_app();
    // debug();
}

fn debug() {
    use crate::compute::geo::IVec3Iter;
    use crate::renderer::gpu::{GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelFaceData, GPUVoxelChunkContent};
    use glam::IVec3;
    use rustc_hash::{FxHashMap, FxHashSet};
    use smallhash;
    use slabmap;
    use std::hint::black_box;
    use std::time::Duration;
    use std::time::Instant;

    dbg!(
        size_of::<GPUVoxelChunk>(),
        size_of::<GPUVoxelChunkAdjContent>(),
        size_of::<GPUVoxelChunkContent>(),
        size_of::<GPUVoxelFaceData>(),
    );
}
