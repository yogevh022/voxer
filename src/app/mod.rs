use crate::render::types::Vertex;
use crate::{RendererState, texture, types, utils};
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub state: Option<RendererState<'a>>,
    pub camera: types::Camera,
    pub scene: types::Scene,
}

impl<'a> App<'a> {
    fn test(&mut self, vert: &[Vertex]) {
        self.state
            .as_mut()
            .unwrap_or_else(|| panic!("rex"))
            .update_vertex_buffer(vert);
    }
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Tech")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
            let window = event_loop.create_window(attributes);
            let arc_window = Arc::new(window.unwrap());
            self.window = Some(arc_window.clone());

            self.camera.set_aspect_ratio(
                arc_window.inner_size().width as f32 / arc_window.inner_size().height as f32,
            );

            let atlas = texture::helpers::generate_texture_atlas();
            _ = atlas.image.save("src/texture/images/atlas.png");

            self.state = Some(pollster::block_on(RendererState::new(arc_window)));

            self.test(&utils::temp::quad_verts_for(texture::Texture::Idk, &atlas));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(window) = &self.window {
            if window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::RedrawRequested => {
                        if let Some(state) = &mut self.state {
                            if let Err(e) = state.render(&self.camera, &self.scene) {
                                println!("{:?}", e);
                            }
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        self.camera
                            .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                    } => {
                        // self.camera.target = Vec3::new(position.x as f32, position.y as f32, 0.0);
                    }
                    _ => {}
                }
            }
        }
    }
}
