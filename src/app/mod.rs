pub mod app_renderer;

use crate::vtypes::{Scene, Voxer, VoxerObject};
use crate::world::{ClientWorld, ClientWorldConfig, ServerWorld};
use crate::{SIMULATION_AND_RENDER_DISTANCE, compute, vtypes, call_every};
use glam::IVec3;
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
    pub server: ServerWorld,
    pub client: Option<ClientWorld<'a>>,
    pub scene: Scene,
    pub debug: AppDebug,
}

impl<'a> App<'a> {
    pub fn new(v: Voxer, server: ServerWorld, scene: Scene) -> Self {
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
            let client_config = ClientWorldConfig {
                render_distance: SIMULATION_AND_RENDER_DISTANCE,
            };
            self.client = Some(ClientWorld::new(arc_window, client_config));
            self.client.as_mut().unwrap().temp_send_req_conn();
            
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
                            window.set_title(&format!(
                                "FPS: {:.2} POS: x: {:.2}, y: {:.2}, z: {:.2}",
                                self.v.time.fps(),
                                self.v.camera.transform.position.x,
                                self.v.camera.transform.position.y,
                                self.v.camera.transform.position.z,
                            ));
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
        {
            let mut input = self.v.input.write();
            input.mouse.set_delta((0.0, 0.0));
            input.keyboard.reset_atomics();
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl<'a> App<'a> {
    fn update(&mut self) {
        {
            let input = self.v.input.read();
            const MOVE_SPEED: f32 = 10.0;
            let fast_mul: f32 = if input.keyboard.key_down(KeyCode::ShiftLeft) {
                4.0
            } else {
                1.0
            };

            let forward_input = input.keyboard.key_down(KeyCode::KeyW) as i8
                - input.keyboard.key_down(KeyCode::KeyS) as i8;
            let right_input = input.keyboard.key_down(KeyCode::KeyD) as i8
                - input.keyboard.key_down(KeyCode::KeyA) as i8;
            let move_vec = forward_input as f32 * self.v.camera.transform.forward()
                + right_input as f32 * self.v.camera.transform.right();
            self.v.camera.transform.position +=
                move_vec * MOVE_SPEED * fast_mul * self.v.time.delta();
        }

        let m_client = self.client.as_mut().unwrap();
        let player_position = self.v.camera.transform.position;
        let frustum_planes = compute::geo::Frustum::planes(self.v.camera.get_view_projection());
        m_client.set_player_position(player_position);
        m_client.set_view_frustum(frustum_planes);
        m_client.tick();

        call_every!(CLIENT_POS_SEND, 20, || { m_client.temp_send_player_position() });

        call_every!(SERVER_TICK, 20, || { self.server.tick() });

    }
}
