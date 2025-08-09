mod app;
mod compute;
mod input;
mod meshing;
mod render;
mod texture;
mod types;
mod utils;
mod worldgen;

use crate::app::{AppTestData, WorkerHandles};
use crate::input::Input;
use crate::meshing::generation::{MeshGenRequest, MeshGenResponse};
use crate::worldgen::types::{World, WorldGenRequest, WorldGenResponse};
use glam::IVec3;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use winit::event_loop::ControlFlow;

fn run_app() {
    let atlas = Arc::new(texture::helpers::generate_texture_atlas());
    _ = atlas.image.save("src/texture/images/atlas.png");

    let world = worldgen::types::World::new(0);

    let (worldgen_response_send, worldgen_response_recv) =
        crossbeam::channel::unbounded::<WorldGenResponse>();
    let (worldgen_request_send, worldgen_request_recv) =
        crossbeam::channel::unbounded::<WorldGenRequest>();
    let (meshgen_response_send, meshgen_response_recv) =
        crossbeam::channel::unbounded::<MeshGenResponse>();
    let (meshgen_request_send, meshgen_request_recv) =
        crossbeam::channel::unbounded::<MeshGenRequest>();

    std::thread::spawn(move || {
        worldgen::types::world_generation_task(
            world.seed,
            worldgen_response_send,
            worldgen_request_recv,
        )
    });
    std::thread::spawn(move || {
        meshing::generation::world_mesh_generation_task(
            atlas,
            meshgen_response_send,
            meshgen_request_recv,
        )
    });

    let worldgen_handle = worldgen::types::WorldGenHandle {
        send: worldgen_request_send,
        receive: worldgen_response_recv,
    };

    let meshgen_handle = meshing::generation::MeshGenHandle {
        send: meshgen_request_send,
        receive: meshgen_response_recv,
    };

    let scene = types::Scene {
        world,
        ..Default::default()
    };

    let camera = types::Camera::default();
    let camera_controller = types::CameraController::default();

    let mut app = app::App {
        window: None,
        renderer: None,
        input: Arc::new(RwLock::new(Input::default())),
        time: utils::Timer::new(),
        worker_handles: WorkerHandles {
            worldgen: worldgen_handle,
            meshgen: meshgen_handle,
        },
        scene,
        camera,
        camera_controller,
        test_data: AppTestData::default(),
    };

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    run_app();

    // let atlas = Arc::new(texture::helpers::generate_texture_atlas());
    // _ = atlas.image.save("src/texture/images/atlas.png");
    //
    // let noise = noise::OpenSimplex::new(0);
    //
    // let mut chunks = Vec::new();
    // for i in 0..200 {
    //     let c_pos = IVec3::new(i as i32, 0, 0);
    //     let chunk = World::generate_chunk(&noise, c_pos);
    //     chunks.push(chunk);
    // }
    //
    // let start = Instant::now();
    // let mut total_verts = 0;
    // for c in chunks.iter() {
    //     let mesh = meshing::chunk::generate_mesh(c, &atlas);
    //     total_verts += mesh.vertices.len();
    // }
    // println!("Time: {:?}", start.elapsed());
    //
    // println!("Total Verts: {}", total_verts);
}
