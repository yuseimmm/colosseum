use kiss3d::{
    nalgebra::{Isometry3, Quaternion, Translation3, Unit},
    scene::SceneNode,
};
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::app::SimulationState;
pub struct SimulationObject {
    scene_node: kiss3d::scene::SceneNode,
    pub rigid_body: Option<RigidBodyHandle>,
    pub colider: ColliderHandle,
}

impl SimulationObject {
    pub fn new(
        scene_node: SceneNode,
        colider: ColliderHandle,
        rigid_body: Option<RigidBodyHandle>,
    ) -> Self {
        SimulationObject {
            scene_node,
            rigid_body,
            colider,
        }
    }
    pub fn synchronize_graphics(&mut self, state: &SimulationState) {
        if let Some(rigid_body_handle) = &self.rigid_body {
            let rigid_body = &state.lock().unwrap().rigid_body_set[*rigid_body_handle];
            let p = rigid_body.position();
            self.scene_node
                .set_local_transformation(Isometry3::from_parts(
                    Translation3::new(p.translation.x, p.translation.y, p.translation.z),
                    Unit::from_quaternion(Quaternion::new(
                        p.rotation.w,
                        p.rotation.i,
                        p.rotation.j,
                        p.rotation.k,
                    )),
                ));
        }
    }
}
