extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use glam::USizeVec3;
use crate::world::{ClientWorldConfig, ServerWorld, ServerWorldConfig, WorldConfig};
use voxer_network;
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

fn run_app() {
    const SIMULATION_AND_RENDER_DISTANCE: usize = 12;

    let server_config = ServerWorldConfig {
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
        world_config: WorldConfig {
            seed: 0,
            max_world_size: USizeVec3::new(1024, 1024, 1024),
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
    use crate::compute;
    use crate::compute::geo::IVec3Iter;
    use crate::renderer::gpu::{
        GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelChunkContent, GPUVoxelFaceData,
    };
    use glam::{IVec3, Vec3};
    use rustc_hash::{FxHashMap, FxHashSet};
    use slabmap;
    use smallhash;
    use std::hint::black_box;
    use std::time::Duration;
    use std::time::Instant;

    let lmin = Vec3::new(-8.0, -2.0, 7.0);
    let lmax = Vec3::new(8.0, 7.0, 14.0);
    let l = compute::geo::AABB::new(lmin, lmax);
    let rmin = Vec3::new(-8.0, -2.0, 8.0);
    let rmax = Vec3::new(8.0, 7.0, 15.0);
    let r = compute::geo::AABB::new(rmin, rmax);
    let (only_l, only_r) = compute::geo::AABB::sym_diff(l, r);
    println!("L: {:?}", l);
    println!("R: {:?}", r);
    println!("only L: {:?}", only_l);
    println!("only R: {:?}", only_r);
}
