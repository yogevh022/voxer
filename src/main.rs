extern crate core;

mod app;
pub mod compute;
mod macros;
mod renderer;
mod vtypes;
mod world;

use glam::USizeVec3;
use crate::world::{ClientWorldConfig, ServerWorld, ServerWorldConfig, WorldConfig};
use voxer_network;
use vtypes::{CameraController, VObject};
use winit::event_loop::ControlFlow;
use crate::compute::geo::AABB;

fn run_app() {
    const SIMULATION_AND_RENDER_DISTANCE: usize = 16;

    let server_config = ServerWorldConfig {
        simulation_distance: SIMULATION_AND_RENDER_DISTANCE,
        world_config: WorldConfig {
            seed: 0,
            max_world_size: USizeVec3::new(1024, 1024, 1024),
            noise_scale: 0.03,
        },
    };

    let client_config = ClientWorldConfig {
        render_distance: SIMULATION_AND_RENDER_DISTANCE,
    };

    let mut server = ServerWorld::new(server_config);
    let voxer_engine = vtypes::Voxer::default();
    let scene = vtypes::Scene {
        objects: vec![VObject::Camera(CameraController::with_sensitivity(0.01))],
    };

    server.start_session();
    let mut app = app::App::new(voxer_engine, server, scene, client_config);
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    run_app();
    // debug();
}

fn debug() {
    use crate::compute;
    use crate::compute::geo::IVec3Iter;
    use crate::renderer::gpu::{
        GPUVoxelChunk, GPUVoxelChunkAdjContent, GPUVoxelChunkContent, GPUVoxelFaceData,
    };
    use glam::{IVec3, Vec3};
    use rustc_hash::{FxHashMap, FxHashSet};
    use slabmap;
    use smallhash;
    use std::hint::black_box;
    use std::time::Duration;
    use std::time::Instant;

    // let a = AABB::new(Vec3::new(-1.4, 2.4, 2.2), Vec3::new(0.0, 1.4, 44.0));
    let x = AABB::new(Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let xd = AABB::new(Vec3::new(1.0, 1.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let x_diff = x.diff(xd);

    let y = AABB::new(Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let yd = AABB::new(Vec3::new(0.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let y_diff = y.diff(yd);

    let z = AABB::new(Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let zd = AABB::new(Vec3::new(0.0, 1.0, 3.0), Vec3::new(3.0, 3.0, 3.0));
    let z_diff = z.diff(zd);

    let xy = AABB::new(Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let xyd = AABB::new(Vec3::new(1.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let xy_diff = xy.diff(xyd);

    let t = AABB::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(4.0, 4.0, 4.0));
    let td = AABB::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let t_diff = t.diff(td);

    println!("xdiff: {:?}", x_diff);
    println!("ydiff: {:?}", y_diff);
    println!("zdiff: {:?}", z_diff);
    println!("xydiff: {:?}", xy_diff);
    println!("tdiff: {:?}", t_diff);
}
