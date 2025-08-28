use crate::app::app_renderer;
use crate::app::app_renderer::AppRenderer;
use crate::compute::geo;
use glam::{IVec3, Vec3};
use std::sync::Arc;
use winit::window::Window;

pub struct WorldClientConfig {
    pub render_distance: usize,
}

pub struct WorldClient<'window> {
    pub config: WorldClientConfig,
    pub renderer: AppRenderer<'window, 2, 1>,
    player_position: Vec3,
}

impl<'window> WorldClient<'window> {
    pub fn new(window: Arc<Window>, config: WorldClientConfig) -> Self {
        Self {
            renderer: app_renderer::make_app_renderer(window, config.render_distance as f32),
            config,
            player_position: Vec3::ZERO,
        }
    }

    pub fn set_player_position(&mut self, position: Vec3) {
        self.player_position = position;
    }

    pub fn map_visible_chunk_positions<F>(&self, mut func: F) -> Vec<IVec3>
    where
        F: FnMut(IVec3) -> bool,
    {
        let mut positions = Vec::new();
        geo::Sphere::discrete_points(
            geo::world_to_chunk_pos(self.player_position),
            self.config.render_distance as isize,
            |chunk_position| {
                // todo if within frustum ..
                if func(chunk_position) {
                    positions.push(chunk_position);
                };
            },
        );
        positions
    }
}
