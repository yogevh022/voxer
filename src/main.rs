extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod voxer_network;
mod vtypes;
mod world;

use std::net::SocketAddr;
use bytemuck::{Pod, Zeroable};
use crate::world::{WorldServer, WorldServerConfig};
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

    server.start();
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
impl voxer_network::NetworkMessageConfig for Test {
    const TAG: voxer_network::NetworkMessageTag = 1;
}

fn debug_server() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3100));
    let net = voxer_network::UdpChannel::<1024>::bind(addr);
    let test = Test {
        hello: 45569,
        world: 34468964,
    };
    let r = net.send_to(test, &String::from("192.168.50.165:3100"));
    dbg!(r);
}

fn debug_client() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3100));
    let mut net = voxer_network::UdpChannel::<1024>::bind(addr);
    loop {
        let new_messages = net.recv_complete();
        if !new_messages.is_empty() {
            for message in new_messages {
                let test_struct: &Test = bytemuck::from_bytes(&message.data[..message.data.len() - 1]);
                dbg!(test_struct);
            }
            break;
        } else {
            println!("eep");
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }
}
