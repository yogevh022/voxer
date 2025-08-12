mod app_renderer;

use crate::app::app_renderer::AppRenderer;
use crate::input::Input;
use crate::world::types::{World, WorldGenHandle};
use crate::{input, types, utils};
use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

const RENDER_DISTANCE: f32 = 10.0;

#[derive(Default)]
pub struct AppTestData {
    pub last_fps: f32,
    pub last_chunk_pos: IVec3,
    pub world_gen_check: Option<Instant>,
}

pub struct WorkerHandles {
    pub world: WorldGenHandle,
}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub app_renderer: Option<AppRenderer<'a>>,
    pub test_data: AppTestData,
    pub worker_handles: WorkerHandles,
    pub time: utils::Timer,
    pub input: Arc<RwLock<Input>>,
    pub scene: types::Scene,
    pub camera: types::Camera,
    pub camera_controller: types::CameraController,
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Tech")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
            let arc_window = Arc::new(event_loop.create_window(attributes).unwrap());
            self.window = Some(arc_window.clone());

            self.camera.set_aspect_ratio(
                arc_window.inner_size().width as f32 / arc_window.inner_size().height as f32,
            );
            // a little silly but for now this makes sure the initial last_chunk_pos is never the same as the current one
            self.test_data.last_chunk_pos =
                utils::vec3_to_ivec3(&(self.camera.transform.position + 1f32));

            let app_renderer = app_renderer::make_app_renderer(arc_window, RENDER_DISTANCE);
            self.app_renderer = Some(app_renderer);
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
                    if let Some(app_renderer) = &mut self.app_renderer {
                        self.time.tick();

                        // temp fps display logic
                        if self.test_data.last_fps != self.time.fps {
                            window.set_title(&format!("Tech - {:.2} FPS", self.time.fps));
                            self.test_data.last_fps = self.time.fps;
                        }
                        // app_renderer
                        //     .unload_chunks(self.scene.world.chunk_manager.dequeue_pending_unload());
                        app_renderer.load_chunks(
                            self.scene.world.chunk_manager.loaded_chunks.len(),
                            &self.scene.world.chunk_manager.dequeue_pending_load(),
                        );
                        if let Err(e) = app_renderer.render(&self.camera) {
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
    fn update_active_chunks(&mut self) {
        let pos = self.camera.transform.position;
        let chunk_pos = World::world_to_chunk_pos(&pos);

        let chunk_pos_f32 = Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);
        let active_chunk_positions =
            utils::geo::discrete_sphere_pts(chunk_pos_f32, RENDER_DISTANCE);

        let active_chunk_positions = HashSet::from([IVec3::new(0, 0, 0)]);

        let app_renderer = self.app_renderer.as_mut().expect("app_renderer is none");

        // UNLOAD FROM RENDERER
        let to_unload = self
            .scene
            .world
            .chunk_manager
            .loaded_chunks_at_positions(&active_chunk_positions);
        self.scene
            .world
            .chunk_manager
            .enqueue_pending_unload(to_unload);

        // RECEIVE FROM WORLDGEN WORKER
        if let Ok(generated_chunks) = self.worker_handles.world.receive.try_recv() {
            self.scene
                .world
                .chunk_manager
                .insert_chunks(generated_chunks);
        }

        // SEND TO WORLDGEN WORKER
        let to_gen = self
            .scene
            .world
            .chunk_manager
            .ungenerated_chunks_at_positions(&active_chunk_positions);
        self.scene
            .world
            .chunk_manager
            .enqueue_pending_generation(&to_gen);

        self.worker_handles.world.send.send(to_gen).unwrap();

        // LOAD TO RENDERER
        let to_load = self
            .scene
            .world
            .chunk_manager
            .unloaded_chunks_at_positions(&active_chunk_positions);
        self.scene.world.chunk_manager.enqueue_pending_load(to_load);

        // SEND TO MESHGEN WORKER
        // self.worker_handles
        //     .meshgen
        //     .send
        //     .send(
        //         chunks_status
        //             .meshless
        //             .into_iter()
        //             .map(|c_pos| {
        //                 self.scene.world.active_generation.meshes.insert(c_pos);
        //                 (
        //                     c_pos,
        //                     self.scene
        //                         .world
        //                         .chunks
        //                         .get_mut(&c_pos)
        //                         .unwrap()
        //                         .take()
        //                         .unwrap(),
        //                 )
        //             })
        //             .collect(),
        //     )
        //     .unwrap();
    }

    fn update(&mut self) {
        {
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

        if self
            .test_data
            .world_gen_check
            .map_or(true, |t| t.elapsed().as_secs_f32() > 0.2)
        {
            self.test_data.world_gen_check = Some(Instant::now());
            self.update_active_chunks();
        }

        // let chunk_pos = World::world_to_chunk_pos(&self.camera.transform.position);
        // if self.test_data.last_chunk_pos != chunk_pos {
        //     self.test_data.last_chunk_pos = chunk_pos;
        //     self.update_active_chunks();
        // }
    }
}
