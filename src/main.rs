mod app;
mod input;
mod mat;
mod render;
mod texture;
mod types;
mod utils;
mod worldgen;

use crate::app::AppTestData;
use crate::input::Input;
use crate::render::types::Model;
use crate::texture::TextureAtlas;
use crate::types::{SceneObject, Transform};
use crate::worldgen::types::CHUNK_SIZE;
use glam::Vec3;
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event_loop::ControlFlow;

fn run_app() {
    let atlas = texture::helpers::generate_texture_atlas();
    _ = atlas.image.save("src/texture/images/atlas.png");

    let scene = types::Scene {
        objects: get_chunks(&atlas),
    };

    let camera = types::Camera::default();
    let camera_controller = types::CameraController::default();

    let mut app = app::App {
        window: None,
        renderer_state: None,
        input: Arc::new(RwLock::new(Input::default())),
        time: utils::Timer::new(),
        scene,
        camera,
        camera_controller,
        test_data: AppTestData::default(),
    };

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn get_chunks(texture_atlas: &TextureAtlas) -> Vec<SceneObject> {
    let mut sos = Vec::new();
    let ns = Arc::new(noise::OpenSimplex::new(0));
    for x in 0..1 {
        for z in 0..1 {
            let chunk_position = Vec3::new(x as f32, 0f32, z as f32);
            let chunk = worldgen::generate_chunk(&ns, chunk_position);
            let chunk_mesh = chunk.generate_mesh(texture_atlas);
            sos.push(SceneObject {
                transform: Transform::from_vec3(Vec3::new(
                    chunk_position.x * CHUNK_SIZE as f32,
                    chunk_position.y * CHUNK_SIZE as f32,
                    chunk_position.z * CHUNK_SIZE as f32,
                )),
                model: Model { mesh: chunk_mesh },
            });
        }
    }
    sos
}

fn main() {
    run_app();
}
