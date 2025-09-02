mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use crate::app::network::{NetworkClient, NetworkServer, Networking};
use crate::world::types::{WorldServer, WorldServerConfig};
use std::net::SocketAddr;
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
    // run_app();

    debug_server();
}

fn debug_server() {
    let mut net_server = NetworkServer::<1024>::new();
    net_server.send(&"192.168.50.165:12345".to_string());
}

fn debug_client() {
    let mut net_client = NetworkClient::<1024>::new();
    net_client.listen(&"192.168.50.215:12345".to_string());
}
