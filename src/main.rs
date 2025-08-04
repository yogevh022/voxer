mod app;
mod input;
mod mat;
mod render;
mod texture;
mod types;
mod utils;
mod worldgen;

use crate::input::Input;
use crate::render::types::Model;
use crate::types::SceneObject;
use glam::Vec3;
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event_loop::ControlFlow;

fn run_app(scene: types::Scene) {
    let atlas = texture::helpers::generate_texture_atlas();
    _ = atlas.image.save("src/texture/images/atlas.png");

    let camera = types::Camera::default();
    let camera_controller = types::CameraController {
        sensitivity: 0.002f32,
        ..Default::default()
    };

    let mut app = app::App {
        window: None,
        renderer_state: None,
        input: Arc::new(RwLock::new(Input::default())),
        scene,
        camera,
        camera_controller,
    };

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn get_chunk_so() -> SceneObject {
    let n = Arc::new(noise::Perlin::new(0));
    let chunk = worldgen::generate_chunk(n);
    let z_mesh = worldgen::get_z_mesh(&chunk);
    SceneObject {
        transform: Default::default(),
        model: Model { mesh: z_mesh },
    }
}

fn main() {
    let scene = types::Scene {
        objects: vec![get_chunk_so()],
    };
    run_app(scene);
}
