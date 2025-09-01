mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use rustc_hash::FxHashMap;
use crate::world::types::{WorldServer, WorldServerConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

const SIMULATION_AND_RENDER_DISTANCE: usize = 8; // fixme temp location

fn run_app() {
    let mut server = WorldServer::new(WorldServerConfig {
        seed: 0,
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
    });
    let voxer_engine = vtypes::Voxer::default();
    let scene = vtypes::Scene {
        objects: vec![VObject::Camera(CameraController::with_sensitivity(0.01))],
    };

    server.start_generation_thread();
    let mut app = app::App::new(voxer_engine, server, scene);

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    // tracy_client::set_thread_name!("main");
    run_app();

    // debug()
}

fn debug() {
    use std::time::Instant;
    use glam::{IVec3, Vec3};

    let mut vec_vec = Vec::new();
    let mut packed_vec = Vec::new();


    let count = 20_000;
    for i in 0..count {
        vec_vec.push(IVec3::new(i, 0, 0));
        packed_vec.push(((i as u64) << 40) | ((i as u64) << 16) | i as u64);
    }
}
