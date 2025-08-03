use crate::mat::model_to_world_matrix;
use crate::render::types::Model;
use crate::types::Transform;
use glam::Mat4;

pub struct SceneObject {
    pub transform: Transform,
    pub model: Model,
}

impl SceneObject {
    pub fn model_matrix(&mut self) -> Mat4 {
        // todo cache model matrix
        model_to_world_matrix(
            self.transform.position,
            self.transform.rotation,
            self.transform.scale,
        )
    }
}
