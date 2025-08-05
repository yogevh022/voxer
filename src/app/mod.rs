use crate::input::Input;
use crate::render::RendererState;
use crate::render::types::Vertex;
use crate::{input, types, utils};
use parking_lot::RwLock;
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

#[derive(Default)]
pub struct AppTestData {
    pub last_fps: f32,
}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub renderer_state: Option<RendererState<'a>>,
    pub test_data: AppTestData,
    pub time: utils::Timer,
    pub input: Arc<RwLock<Input>>,
    pub scene: types::Scene,
    pub camera: types::Camera,
    pub camera_controller: types::CameraController,
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
                        self.time.tick();

                        // temp fps display logic
                        if self.test_data.last_fps != self.time.fps {
                            window.set_title(&format!("Tech - {:.2} FPS", self.time.fps));
                            self.test_data.last_fps = self.time.fps;
                        }

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
                        ElementState::Pressed => self.input.write().keyboard.press(key_code),
                        ElementState::Released => self.input.write().keyboard.release(key_code),
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.input.write().mouse.add_delta(delta);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.update();
        self.input.write().mouse.set_delta((0.0, 0.0));

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl<'a> App<'a> {
    fn update(&mut self) {
        let input = self.input.read();
        const MOUSE_SENSITIVITY: f64 = 0.01;
        const MOVE_SPEED: f32 = 30.0;

        self.camera_controller.look((
            input.mouse.delta[0] * MOUSE_SENSITIVITY,
            input.mouse.delta[1] * MOUSE_SENSITIVITY,
        ));
        self.camera.transform.rotation = self.camera_controller.get_rotation();

        let forward_input = input.keyboard.key_down(KeyCode::KeyW) as i8
            - input.keyboard.key_down(KeyCode::KeyS) as i8;
        let right_input = input.keyboard.key_down(KeyCode::KeyD) as i8
            - input.keyboard.key_down(KeyCode::KeyA) as i8;
        let move_vec = forward_input as f32 * self.camera.transform.forward()
            + right_input as f32 * self.camera.transform.right();
        self.camera.transform.position += move_vec * MOVE_SPEED * self.time.delta_time;
    }
}
