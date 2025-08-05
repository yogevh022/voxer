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
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event_loop::ControlFlow;

fn run_app() {
    let atlas = texture::helpers::generate_texture_atlas();
    _ = atlas.image.save("src/texture/images/atlas.png");

    let mut world = worldgen::types::World::new(0);
    world.generate(&atlas);

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
}
