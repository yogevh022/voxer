pub mod app_renderer;

use std::collections::HashSet;
use crate::vtypes::{Scene, Voxer, VoxerObject};
use crate::world::types::{WorldClient, WorldServer};
use crate::{compute, vtypes};
use glam::{IVec3, Vec3};
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

#[derive(Default)]
pub struct AppDebug {
    pub last_chunk_pos: IVec3,
}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub v: Voxer, // voxer engine; input, time, camera, etc
    pub server: WorldServer,
    pub client: Option<WorldClient<'a>>,
    pub scene: Scene,
    pub debug: AppDebug,
}

impl App<'_> {
    pub fn new(v: Voxer, server: WorldServer, scene: Scene) -> Self {
        Self {
            window: None,
            v,
            server,
            client: None,
            scene,
            debug: Default::default(),
        }
    }
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Tech")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
            let arc_window = Arc::new(event_loop.create_window(attributes).unwrap());
            self.window = Some(arc_window.clone());

            self.v.camera.set_aspect_ratio(
                arc_window.inner_size().width as f32 / arc_window.inner_size().height as f32,
            );
            self.client = Some(WorldClient::new(arc_window));
            self.debug.last_chunk_pos = IVec3::new(100, 100, 100);
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
                    if let Some(client) = &mut self.client {
                        self.v.time.tick();

                        if self.v.time.temp_200th_frame() {
                            window.set_title(&format!("Tech - {:.2} FPS", self.v.time.fps()));
                        }

                        if let Err(e) = client.renderer.render(&self.v.camera) {
                            println!("{:?}", e);
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    self.v
                        .camera
                        .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let key_code =
                        vtypes::input::get_keycode(event.physical_key).expect("unknown key");
                    match event.state {
                        ElementState::Pressed => self.v.input.write().keyboard.press(key_code),
                        ElementState::Released => self.v.input.write().keyboard.release(key_code),
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let Some(window) = &self.window else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.v.input.write().mouse.add_delta(delta);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        for vo in self.scene.objects.iter_mut() {
            vo.update(&mut self.v);
        }
        self.update();
        self.v.input.write().mouse.set_delta((0.0, 0.0));

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl<'a> App<'a> {
    fn update(&mut self) {
        {
            let input = self.v.input.read();
            const MOVE_SPEED: f32 = 30.0;

            let forward_input = input.keyboard.key_down(KeyCode::KeyW) as i8
                - input.keyboard.key_down(KeyCode::KeyS) as i8;
            let right_input = input.keyboard.key_down(KeyCode::KeyD) as i8
                - input.keyboard.key_down(KeyCode::KeyA) as i8;
            let move_vec = forward_input as f32 * self.v.camera.transform.forward()
                + right_input as f32 * self.v.camera.transform.right();
            self.v.camera.transform.position += move_vec * MOVE_SPEED * self.v.time.delta();
        }

        let player_pos = self.v.camera.transform.position;
        // let player_pos = Vec3::default();
        self.server.set_player(0, &player_pos);

        if self.v.time.temp_200th_frame() {
            self.server.update();
            let sim_chunks = self.server.get_simulated_chunks();
            let (load_positions, unload_positions) =
                self.client.as_ref().unwrap().compare_for_delta(&sim_chunks);
            let new_chunks = self.server.get_chunks(load_positions);
            self.client
                .as_mut()
                .unwrap()
                .update_chunks_by_delta(new_chunks, unload_positions);
    
            self.client.as_mut().unwrap().sync_with_renderer();
        }

    }
}
