extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use std::hint::black_box;
use glam::{IVec3, Vec3};
use crate::world::{ServerWorld, ServerWorldConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;
use voxer_network;
use crate::compute::geo::AABB;
use crate::world::generation::generate_chunk;

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
    // debug3();
    // debug2();
}

fn debug3() {

}

fn debug2() {
    use crate::compute::chunk;

    let rend = 16;

    let mut chunks = Vec::new();
    let gen_config = world::generation::WorldConfig {
        seed: 0,
        noise_scale: 0.05,
        simulation_distance: rend,
    };

    let start = std::time::Instant::now();
    for i in 0..200 {
        let chunk = generate_chunk(gen_config, IVec3::new(i, 0, 0));
        chunks.push(chunk);
    }
    let gen_end = start.elapsed();

    let start = std::time::Instant::now();
    for c in chunks.iter() {
        let fc = chunk::face_count(&c.blocks, &c.adjacent_blocks);
    }
    let end = start.elapsed();

    println!("gen: {:?}, face: {:?}", gen_end, end);
}
