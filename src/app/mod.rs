pub mod app_renderer;

use crate::compute::geo::Frustum;
use crate::vtypes::{Scene, Voxer, VoxerObject};
use crate::world::types::CHUNK_DIM;
use crate::world::{ClientWorld, ClientWorldConfig, ServerWorld};
use crate::{call_every, compute, vtypes};
use std::sync::Arc;
use wgpu::ComputePassDescriptor;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window};

#[derive(Default)]
pub struct AppDebug {}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub v: Voxer, // voxer engine; input, time, camera, etc
    pub server: ServerWorld,
    pub client: Option<ClientWorld<'a>>,
    pub client_config: ClientWorldConfig,
    pub scene: Scene,
    pub debug: AppDebug,
}

impl<'a> App<'a> {
    pub fn new(
        v: Voxer,
        server: ServerWorld,
        scene: Scene,
        client_config: ClientWorldConfig,
    ) -> Self {
        Self {
            window: None,
            v,
            server,
            client: None,
            client_config,
            scene,
            debug: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct WindowDescriptor {
    size: (u32, u32),
    title: String,
    cursor_grab: CursorGrabMode,
    cursor_visible: bool,
}

fn initialize_window(
    event_loop: &ActiveEventLoop,
    descriptor: &WindowDescriptor,
) -> Result<Arc<Window>, String> {
    let physical_size = PhysicalSize::new(descriptor.size.0, descriptor.size.1);
    let attributes = Window::default_attributes()
        .with_title(descriptor.title.clone())
        .with_inner_size(physical_size);
    let window = match event_loop.create_window(attributes) {
        Ok(window) => Arc::new(window),
        Err(e) => return Err(e.to_string()),
    };
    window
        .set_cursor_grab(descriptor.cursor_grab)
        .map_err(|e| e.to_string())?;
    window.set_cursor_visible(descriptor.cursor_visible);
    Ok(window)
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window_desc = WindowDescriptor {
            size: (1280, 720),
            title: "Tech".to_string(),
            cursor_grab: CursorGrabMode::Locked,
            cursor_visible: false,
        };
        let window = initialize_window(event_loop, &window_desc).unwrap();
        self.window = Some(window.clone());

        let win_size = window.inner_size();
        let aspect_ratio = win_size.width as f32 / win_size.height as f32;
        self.v.camera.set_aspect_ratio(aspect_ratio);
        self.v.camera.transform.position = glam::vec3(0.0, 10.0, 0.0);

        let client = ClientWorld::new(window, self.client_config);
        client.temp_send_req_conn();
        self.client = Some(client);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let window = match &self.window {
            Some(window) if window.id() == window_id => window,
            _ => return,
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let client = self.client.as_mut().unwrap();
                self.v.time.tick();

                call_every!(WINDOW_TITLE_UPDATE, 200, || {
                    let p = self.v.camera.transform.position;
                    let fps = self.v.time.fps().floor() as u32;
                    let title =
                        format!("FPS: {:>4} ({:>8.1},{:>8.1},{:>8.1})", fps, p.x, p.y, p.z,);
                    window.set_title(&title);
                });

                let app_renderer = &mut client.session.app_renderer;
                let mut encoder = app_renderer.renderer.create_encoder("Main Encoder");
                {
                    let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Main Compute Pass"),
                        timestamp_writes: None,
                    });
                    client
                        .session
                        .app_renderer
                        .renderer
                        .depth
                        .generate_initial_mip(
                            &client.session.app_renderer.renderer.device,
                            &mut compute_pass,
                        );
                    client
                        .session
                        .app_renderer
                        .renderer
                        .depth
                        .generate_depth_mips(
                            &client.session.app_renderer.renderer.device,
                            &mut compute_pass,
                        );
                    client.tick(&mut compute_pass);
                }
                let voxel_render_distance = self.client_config.render_distance * CHUNK_DIM;
                let render_result = client.session.app_renderer.submit_render_pass(
                    encoder,
                    &self.v.camera,
                    voxel_render_distance as u32,
                );
                render_result.unwrap_or_else(|e| println!("{:?}", e));
            }
            WindowEvent::Resized(new_size) => {
                let aspect_ratio = new_size.width as f32 / new_size.height as f32;
                self.v.camera.set_aspect_ratio(aspect_ratio);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key_code = vtypes::input::get_keycode(event.physical_key).expect("unknown key");
                match event.state {
                    ElementState::Pressed => self.v.input.write().keyboard.press(key_code),
                    ElementState::Released => self.v.input.write().keyboard.release(key_code),
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if self.window.is_none() {
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.v.input.write().mouse.accumulate_delta(delta);
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
            let sprint_mul = (1 + input.keyboard.key_down(KeyCode::ShiftLeft) as u32 * 6) as f32;

            let w = input.keyboard.key_down(KeyCode::KeyW) as i32;
            let a = input.keyboard.key_down(KeyCode::KeyA) as i32;
            let s = input.keyboard.key_down(KeyCode::KeyS) as i32;
            let d = input.keyboard.key_down(KeyCode::KeyD) as i32;
            let forward_input = w - s;
            let right_input = d - a;
            let move_vec = forward_input as f32 * self.v.camera.transform.forward()
                + right_input as f32 * self.v.camera.transform.right();
            self.v.camera.transform.position +=
                move_vec * MOVE_SPEED * sprint_mul * self.v.time.delta();
        }

        let safe_voxel_rdist = ((self.client_config.render_distance - 1) * CHUNK_DIM) as f32;
        let safe_vp = self.v.camera.view_projection_with_far(safe_voxel_rdist);
        let safe_frustum_planes = Frustum::planes(safe_vp);
        let player_position = self.v.camera.transform.position;

        let m_client = self.client.as_mut().unwrap();
        m_client.temp_set_player_position(player_position);
        m_client.temp_set_view_frustum(safe_frustum_planes);

        call_every!(CLIENT_POS_SEND, 20, || {
            m_client.temp_send_player_position()
        });

        self.server.tick();
    }
}
