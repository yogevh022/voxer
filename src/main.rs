mod app;
mod constants;
mod input;
mod mat;
mod render;
mod texture;
mod types;
mod utils;

use crate::input::Input;
use glam::Vec3;
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event_loop::ControlFlow;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let atlas = texture::helpers::generate_texture_atlas();
    _ = atlas.image.save("src/texture/images/atlas.png");

    let mut curr_index = 0u16; // temp, track indices

    let scene = types::Scene {
        objects: vec![
            utils::temp::scene_plane(
                &mut curr_index,
                &atlas,
                texture::Texture::Murica,
                types::Transform::from_vec3(Vec3::new(0.0, 0.0, -5.0)),
            ),
            utils::temp::scene_plane(
                &mut curr_index,
                &atlas,
                texture::Texture::Idk,
                types::Transform::from_vec3(Vec3::new(2.0, 0.0, -5.0)),
            ),
            utils::temp::scene_plane(
                &mut curr_index,
                &atlas,
                texture::Texture::Green,
                types::Transform::from_vec3(Vec3::new(-2.0, 0.0, -5.0)),
            ),
        ],
    };
    let camera = types::Camera {
        target: Vec3::new(0.0, 0.0, -1.0),
        ..Default::default()
    };

    let mut app = app::App {
        window: None,
        renderer_state: None,
        input: Arc::new(RwLock::new(Input::default())),
        scene,
        camera,
    };

    event_loop.run_app(&mut app).unwrap();
}
