use crate::input::Input;
use crate::render::Renderer;
use crate::render::types::Mesh;
use crate::texture::TextureAtlas;
use crate::worldgen::types::{Chunk, World};
use crate::{input, meshing, types, utils};
use glam::{IVec3, Vec3};
use parking_lot::RwLock;
use rayon::iter::ParallelIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

#[derive(Default)]
pub struct AppTestData {
    pub last_fps: f32,
    pub last_chunk_pos: IVec3,
    pub atlas: Option<TextureAtlas>,
}

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer<'a>>,
    pub test_data: AppTestData,
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

            self.renderer = Some(pollster::block_on(Renderer::new(arc_window)));
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
        const RENDER_DISTANCE: f32 = 6.0;
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

        let pos = self.camera.transform.position;
        let chunk_pos = utils::world::world_to_chunk_pos(&pos);
        if self.test_data.last_chunk_pos != chunk_pos {
            self.test_data.last_chunk_pos = chunk_pos;
            let chunk_pos_f32 =
                Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);
            let renderer = self.renderer.as_mut().expect("renderer is none");
            let active_chunk_positions =
                utils::geo::discrete_points_within_sphere(chunk_pos_f32, RENDER_DISTANCE);
            let chunk_tasks = self.scene.world.chunks_tasks(&active_chunk_positions);

            // generating chunks
            let g_chunks = &self.scene.world.generating_chunks;
            let noise = &self.scene.world.noise;
            let pending_chunk_generation = {
                let generating_chunks_lock = self.scene.world.generating_chunks.read();
                chunk_tasks
                    .generate_chunk
                    .into_iter()
                    .filter(|c| !generating_chunks_lock.contains(*c))
                    .collect::<Vec<_>>()
            };
            let chunks: Vec<(IVec3, Chunk)> = pending_chunk_generation
                .par_iter()
                .map(|c_pos| {
                    g_chunks.write().insert(**c_pos);
                    (**c_pos, World::generate_chunk(&noise, **c_pos))
                })
                .collect();

            let mut g_chunks_lock = g_chunks.write();
            for (c_pos, chunk) in chunks {
                g_chunks_lock.remove(&c_pos);
                self.scene.world.chunks.insert(c_pos, chunk);
            }

            // generating meshes
            {
                let g_meshes = &self.scene.world.generating_meshes;

                let pending_mesh_generation = {
                    let generating_meshes_lock = g_meshes.read();
                    chunk_tasks
                        .generate_mesh
                        .into_iter()
                        .filter(|c| !generating_meshes_lock.contains(*c))
                        .collect::<Vec<_>>()
                };

                let mut meshes: Vec<(&IVec3, Mesh)> =
                    Vec::with_capacity(pending_mesh_generation.len());
                pending_mesh_generation
                    .par_iter()
                    .map(|c_pos| {
                        g_meshes.write().insert(**c_pos);
                        (
                            *c_pos,
                            meshing::chunk::generate_mesh(
                                &self.scene.world.chunks[*c_pos],
                                self.test_data.atlas.as_ref().unwrap(),
                            ),
                        )
                    })
                    .collect_into_vec(&mut meshes);
                let mut g_meshes_lock = g_meshes.write();
                for (c_pos, mesh) in meshes {
                    g_meshes_lock.remove(c_pos);
                    let chunk = self.scene.world.chunks.get_mut(c_pos).unwrap();
                    chunk.mesh = Some(mesh);
                }
            }

            // unload unactive from renderer
            for expired_entry_index in renderer
                .expired_chunks(&active_chunk_positions)
                .iter()
                .rev()
            {
                renderer.remove_chunk_buffer(*expired_entry_index);
            }

            // load active to renderer
            for chunk_pos in renderer
                .emerging_chunks(chunk_tasks.renderer_load)
                .into_iter()
            {
                let c_mesh = &self.scene.world.chunks[chunk_pos].mesh.as_ref().unwrap();
                renderer.add_chunk_buffer(*chunk_pos, c_mesh)
            }
        }
    }
}
