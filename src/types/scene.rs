use crate::types::SceneObject;
use crate::world::types::World;

#[derive(Default)]
pub struct Scene {
    pub world: World,
    pub objects: Vec<SceneObject>,
}
