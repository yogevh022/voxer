use crate::input::Input;
use crate::render::RendererState;
use crate::render::types::Vertex;
use crate::{input, texture, types, utils};
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub renderer_state: Option<RendererState<'a>>,
    pub input: Arc<RwLock<Input>>,
    pub scene: types::Scene,
    pub camera: types::Camera,
}

impl<'a> App<'a> {
    fn test(&mut self, vert: &[Vertex]) {
        self.renderer_state
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

            self.renderer_state = Some(pollster::block_on(RendererState::new(
                arc_window,
                &self.scene.objects,
            )));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };
        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::RedrawRequested => {
                    if let Some(state) = &mut self.renderer_state {
                        if let Err(e) = state.render(&self.camera, &mut self.scene) {
                            println!("{:?}", e);
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    self.camera
                        .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let key_code =
                        input::keycode::get_keycode(event.physical_key).expect("unknown key");
                    match event.state {
                        ElementState::Pressed => self.input.write().press(key_code),
                        ElementState::Released => self.input.write().release(key_code),
                    }
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };
        window.request_redraw();
    }
}
