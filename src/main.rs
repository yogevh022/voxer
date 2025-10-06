extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use crate::world::{ServerWorld, ServerWorldConfig};
use voxer_network;
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

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
    run_app();
    // debug();
}

fn debug() {
    use crate::compute::geo::IVec3Iter;
    use crate::renderer::gpu::GPUVoxelChunk;
    use glam::IVec3;
    use rustc_hash::{FxHashMap, FxHashSet};
    use smallhash;
    use std::hint::black_box;
    use std::time::Duration;
    use std::time::Instant;
}
