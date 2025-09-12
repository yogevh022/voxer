extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use glam::IVec3;
use crate::world::{ServerWorld, ServerWorldConfig};
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;
use voxer_network;
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
    // tracy_client::set_thread_name!("main");
    run_app();

    // debug();
    // debug2();
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

fn debug() {
    use crate::compute::geo;
    use crate::compute::geo::Frustum;
    use crate::vtypes::Camera;
    use crate::world::types::CHUNK_DIM;
    use glam::IVec3;
    use rustc_hash::FxHashMap;
    use std::time::Instant;


    let mut hmap: FxHashMap<IVec3, usize> = FxHashMap::default();

    let rend = 64;
    let count = 1000;
    let cam = Camera::default();
    let cvp = cam.chunk_view_projection(rend as f32);
    let vf = Frustum::planes(cvp);

    let new_start = Instant::now();

    for i in 0..count {
        let mut frustum_aabb = Frustum::aabb(&vf);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();
        frustum_aabb.discrete_points(|chunk_position| {
            const CHUNK_RADIUS_APPROX: f32 = 0.5 * 1.7320508f32 * CHUNK_DIM as f32; // sqrt(3) const
            let chunk_world_position = geo::chunk_to_world_pos(chunk_position);
            if !Frustum::sphere_within_frustum(chunk_world_position, CHUNK_RADIUS_APPROX, &vf) {
                hmap.insert(chunk_position, i);
            }
        });
    }
    let new_end = new_start.elapsed();


    let old_start = Instant::now();
    for i in 0..count {
        let mut frustum_aabb = Frustum::aabb(&vf);
        frustum_aabb.min = (frustum_aabb.min / CHUNK_DIM as f32).floor();
        frustum_aabb.max = (frustum_aabb.max / CHUNK_DIM as f32).ceil();
        frustum_aabb.discrete_points(|chunk_position| {
            const CHUNK_RADIUS_APPROX: f32 = 0.5 * 1.7320508f32 * CHUNK_DIM as f32; // sqrt(3) const
            let chunk_world_position = geo::chunk_to_world_pos(chunk_position);
            if !Frustum::aabb_within_frustum(
                chunk_world_position,
                chunk_world_position + CHUNK_DIM as f32,
                &vf,
            ) {
                hmap.insert(chunk_position, i);
            }
        });
    }
    let old_end = old_start.elapsed();

    println!("old {:?}", old_end);
    println!("new {:?}", new_end);
}
