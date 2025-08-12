pub mod app_renderer;

use crate::vtypes::{Scene, Voxer, VoxerObject};
use crate::world::types::{WorldClient, WorldServer};
use crate::{compute, vtypes};
use glam::Vec3;
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct App<'a> {
    pub window: Option<Arc<Window>>,
    pub v: Voxer, // voxer engine; input, time, camera, etc
    pub server: WorldServer,
    pub client: Option<WorldClient<'a>>,
    pub scene: Scene,
}

impl App<'_> {
    pub fn new(v: Voxer, server: WorldServer, scene: Scene) -> Self {
        Self {
            window: None,
            v,
            server,
            client: None,
            scene,
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
                            client.sync_with_renderer();
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
    fn update_active_chunks(&mut self) {
        let pos = self.v.camera.transform.position;
        let chunk_pos = compute::geo::world_to_chunk_pos(&pos);

        let chunk_pos_f32 = Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32);
        let active_chunk_positions = compute::geo::discrete_sphere_pts(&chunk_pos_f32, 0.0);

        // let active_chunk_positions = HashSet::from([IVec3::new(0, 0, 0)]);

        let app_renderer = &mut self.client.as_mut().expect("client is none").renderer;

        // UNLOAD FROM RENDERER
        // let to_unload = self
        //     .scene
        //     .world
        //     .chunk_manager
        //     .loaded_chunks_at_positions(&active_chunk_positions);
        // self.scene
        //     .world
        //     .chunk_manager
        //     .enqueue_pending_unload(to_unload);

        // RECEIVE FROM WORLDGEN WORKER
        // if let Ok(generated_chunks) = self.worker_handles.world.receive.try_recv() {
        //     self.scene
        //         .world
        //         .chunk_manager
        //         .insert_chunks(generated_chunks);
        // }

        // SEND TO WORLDGEN WORKER
        // let to_gen = self
        //     .scene
        //     .world
        //     .chunk_manager
        //     .ungenerated_chunks_at_positions(&active_chunk_positions);
        // self.scene
        //     .world
        //     .chunk_manager
        //     .enqueue_pending_generation(&to_gen);
        //
        // self.worker_handles.world.send.send(to_gen).unwrap();

        // LOAD TO RENDERER
        // let to_load = self
        //     .scene
        //     .world
        //     .chunk_manager
        //     .unloaded_chunks_at_positions(&active_chunk_positions);
        // self.scene.world.chunk_manager.enqueue_pending_load(to_load);

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

        self.server.set_player(0, &self.v.camera.transform.position);

        if self.v.time.temp_200th_frame() {
            self.server.update();
            let sim_chunks = self.server.get_simulated_chunks();
            let (load_positions, unload_positions) =
                self.client.as_ref().unwrap().compare_for_delta(&sim_chunks);
            let new_chunks = self.server.get_chunks(load_positions);
            self.client
                .as_mut()
                .unwrap()
                .update_by_delta(new_chunks, unload_positions);
        }

        // if self
        //     .test_data
        //     .world_gen_check
        //     .map_or(true, |t| t.elapsed().as_secs_f32() > 0.2)
        // {
        //     self.test_data.world_gen_check = Some(Instant::now());
        //     self.update_active_chunks();
        // }

        // let chunk_pos = World::world_to_chunk_pos(&self.camera.transform.position);
        // if self.test_data.last_chunk_pos != chunk_pos {
        //     self.test_data.last_chunk_pos = chunk_pos;
        //     self.update_active_chunks();
        // }
    }
}
