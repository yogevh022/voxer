use crate::types::SceneObject;
use crate::worldgen::types::World;

#[derive(Default)]
pub struct Scene {
    pub world: World,
    pub objects: Vec<SceneObject>,
}
