mod app;
pub mod compute;
mod renderer;
mod vtypes;
mod world;

use crate::world::types::{WorldServer, WorldServerConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

const SIMULATION_AND_RENDER_DISTANCE: usize = 6; // fixme temp location

fn run_app() {
    let mut server = WorldServer::new(WorldServerConfig {
        seed: 0,
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
    });
    let voxer_engine = vtypes::Voxer::default();
    let scene = vtypes::Scene {
        objects: vec![VObject::Camera(CameraController::default())],
    };

    server.start_generation_thread();
    let mut app = app::App::new(voxer_engine, server, scene);

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
