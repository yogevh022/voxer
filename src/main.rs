mod app;
pub mod compute;
mod input;
mod meshing;
mod render;
mod texture;
mod types;
mod utils;
mod world;

use crate::app::{AppTestData, WorkerHandles};
use crate::input::Input;
use crate::world::types::{World, WorldGenRequest, WorldGenResponse};
use glam::IVec3;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use winit::event_loop::ControlFlow;

fn run_app() {
    let atlas = Arc::new(texture::helpers::generate_texture_atlas());
    _ = atlas.image.save("src/texture/images/atlas.png");

    let world = world::types::World::new(0);

    let (worldgen_response_send, worldgen_response_recv) =
        crossbeam::channel::unbounded::<WorldGenResponse>();
    let (worldgen_request_send, worldgen_request_recv) =
        crossbeam::channel::unbounded::<WorldGenRequest>();

    std::thread::spawn(move || {
        world::types::world_generation_task(
            world.seed,
            worldgen_response_send,
            worldgen_request_recv,
        )
    });

    let worldgen_handle = world::types::WorldGenHandle {
        send: worldgen_request_send,
        receive: worldgen_response_recv,
    };

    let scene = types::Scene {
        world,
        ..Default::default()
    };

    let camera = types::Camera::default();
    let camera_controller = types::CameraController::default();

    let mut app = app::App {
        window: None,
        app_renderer: None,
        input: Arc::new(RwLock::new(Input::default())),
        time: utils::Timer::new(),
        worker_handles: WorkerHandles {
            world: worldgen_handle,
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
    //
    // let atlas = Arc::new(texture::helpers::generate_texture_atlas());
    // _ = atlas.image.save("src/texture/images/atlas.png");
    //
    // let noise = noise::OpenSimplex::new(0);
    //
    // let mut chunks = Vec::new();
    // for i in 0..200 {
    //     let c_pos = IVec3::new(i, 0, 0);
    //     let chunk = World::generate_chunk(&noise, c_pos);
    //     chunks.push(chunk);
    // }
    //
    // let start = Instant::now();
    // let mut total_verts = 0;
    // for c in chunks.iter() {
    //     let size = compute::chunk::chunk_face_count(&c);
    //     // let _ = compute::chunk::block_bits(c);
    //     total_verts += size;
    // }
    // println!("Time: {:?}", start.elapsed());
    //
    // println!(
    //     "total {}, total size: {}kb",
    //     total_verts,
    //     ((total_verts * 4 * 4 * 3) + (total_verts * 4 * 6)) / 1024
    // );
}
