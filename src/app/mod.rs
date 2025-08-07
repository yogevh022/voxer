use std::collections::HashSet;
use crate::input::Input;
use crate::meshing::generation::MeshGenHandle;
use crate::render::Renderer;
use crate::worldgen::types::{World, WorldGenHandle};
use crate::{input, types, utils};
use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

const RENDER_DISTANCE: f32 = 12.0;

#[derive(Default)]
pub struct AppTestData {
    pub last_fps: f32,
    pub last_chunk_pos: IVec3,
    pub world_gen_check: Option<Instant>,
}

pub struct WorkerHandles {
    pub worldgen: WorldGenHandle,
    pub meshgen: MeshGenHandle,
}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'a>>,
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
            let window = event_loop.create_window(attributes);
            let arc_window = Arc::new(window.unwrap());
            self.window = Some(arc_window.clone());

            self.camera.set_aspect_ratio(
                arc_window.inner_size().width as f32 / arc_window.inner_size().height as f32,
            );
            // a little silly but for now this makes sure the initial last_chunk_pos is never the same as the current one
            self.test_data.last_chunk_pos =
                utils::vec3_to_ivec3(&(self.camera.transform.position + 1f32));

            self.renderer = Some(pollster::block_on(Renderer::new(
                arc_window,
                RENDER_DISTANCE,
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
                    if let Some(state) = &mut self.renderer {
                        self.time.tick();

                        // temp fps display logic
                        if self.test_data.last_fps != self.time.fps {
                            window.set_title(&format!("Tech - {:.2} FPS", self.time.fps));
                            self.test_data.last_fps = self.time.fps;
                        }

                        if let Err(e) = state.render(&self.camera) {
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

        let renderer = self.renderer.as_mut().expect("renderer is none");

        // UNLOAD FROM RENDERER
        for expired_entry_index in renderer
            .expired_chunks(&active_chunk_positions)
            .iter()
            .rev()
        {
            renderer.remove_chunk_buffer(*expired_entry_index);
        }

        // RECEIVE FROM WORLDGEN WORKER
        if let Ok(generated_chunks) = self.worker_handles.worldgen.receive.try_recv() {
            for (c_pos, chunk) in generated_chunks {
                self.scene.world.chunks.insert(c_pos, Some(chunk));
                self.scene.world.active_generation.chunks.remove(&c_pos);
            }
        }

        // RECEIVE FROM MESHGEN WORKER
        if let Ok(generated_meshes) = self.worker_handles.meshgen.receive.try_recv() {
            for (c_pos, chunk) in generated_meshes {
                self.scene.world.chunks.insert(c_pos, Some(chunk));
                self.scene.world.active_generation.meshes.remove(&c_pos);
            }
        }

        // CHECK CHUNK STATUS *AFTER* RECEIVING FROM WORKERS
        let chunks_status = self.scene.world.chunks_status(&active_chunk_positions);

        // SEND TO WORLDGEN WORKER
        self.worker_handles
            .worldgen
            .send
            .send(
                chunks_status
                    .not_found
                    .into_iter()
                    .map(|c_pos| {
                        self.scene.world.active_generation.chunks.insert(c_pos);
                        c_pos
                    })
                    .collect(),
            )
            .unwrap();

        // SEND TO MESHGEN WORKER
        self.worker_handles
            .meshgen
            .send
            .send(
                chunks_status
                    .meshless
                    .into_iter()
                    .map(|c_pos| {
                        self.scene.world.active_generation.meshes.insert(c_pos);
                        (
                            c_pos,
                            self.scene
                                .world
                                .chunks
                                .get_mut(&c_pos)
                                .unwrap()
                                .take()
                                .unwrap(),
                        )
                    })
                    .collect(),
            )
            .unwrap();

        // fixme fix the collect atrocity
        // LOAD TO RENDERER
        for emerging_chunk in renderer.emerging_chunks(chunks_status.to_render).collect::<HashSet<_>>().iter() {
            let mesh = self.scene.world.chunks.get(emerging_chunk).unwrap()
                .as_ref()
                .unwrap()
                .mesh
                .as_ref()
                .unwrap();
            renderer.add_chunk_buffer(*emerging_chunk, mesh);
        }
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
