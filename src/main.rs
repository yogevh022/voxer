extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use bytemuck::{Pod, Zeroable};
use crate::world::{ServerWorld, ServerWorldConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;
use voxer_network;

const SIMULATION_AND_RENDER_DISTANCE: usize = 16; // fixme temp location

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
    // tracy_client::set_thread_name!("main");
    run_app();

    // debug_server();
    // debug_client();
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Test {
    hello: u64,
    world: u64,
}
