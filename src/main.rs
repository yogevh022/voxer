mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use crate::world::types::{WorldServer, WorldServerConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;

const SIMULATION_AND_RENDER_DISTANCE: usize = 4; // fixme temp location

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

    // debug_chunk_gen();
}

fn debug() {
    use std::time::Instant;
    use glam::{IVec3, Vec3};

    let ipoint = IVec3::ZERO;
    let mut ps = Vec::new();
    let start = Instant::now();
    compute::geo::Sphere::discrete_points(ipoint, 2, |p| {
        ps.push(p);
    });
    println!("{}; {:?}", ps.len(), start.elapsed());
    println!("{:?}", ps);
}

fn debug_chunk_gen() {
    use crate::world::generation::WorldGenConfig;
    use crate::world::generation::generate_chunk;
    use glam::IVec3;
    use std::time::Instant;

    let noise = noise::OpenSimplex::new(0);

    let worldgen_config = WorldGenConfig {
        seed: 0,
        noise_scale: 0.05,
    };
    let mut chunks = Vec::new();
    for i in 0..1 {
        let c_pos = IVec3::new(2, 0, 0);
        let chunk = generate_chunk(worldgen_config, c_pos);
        // let chunk = Chunk {
        //     last_visited: None,
        //     blocks: ChunkBlocks::checkerboard(
        //         Block {
        //             value: 1u16 << 15u16,
        //         },
        //         Block { value: 0u16 },
        //     ),
        // };
        // let chunk = Chunk {
        //     last_visited: None,
        //     blocks: ChunkBlocks::splat(Block {
        //         value: 1u16 << 15u16,
        //     }),
        // };
        chunks.push(chunk);
    }

    let start = Instant::now();
    let mut total_verts = 0;
    for c in chunks.iter() {
        let size = compute::chunk::face_count(&c.blocks);
        total_verts += size;
    }
    println!("Time: {:?}", start.elapsed());

    println!(
        "total {}, total size: {}kb",
        total_verts,
        ((total_verts * 4 * 4 * 3) + (total_verts * 4 * 6)) / 1024
    );
}
