use crate::simulation_object::SimulationObject;
use kiss3d::scene::SceneNode;
use rapier3d::prelude::*;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

/// This will store the state of our physical sumilation.
///
/// NOTE: `SimulationState` is internally protected by a `Mutex`.
/// If a locked `SimulationState` is still alive within the current scope,
/// calling lock again in the same scope may cause the application to freeze.
pub struct SimulationState {
    inner: Arc<Mutex<SimulationStateInner>>,
}

impl Deref for SimulationState {
    type Target = Arc<Mutex<SimulationStateInner>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SimulationState {
    pub fn new() -> Self {
        SimulationState {
            inner: Arc::new(Mutex::new(SimulationStateInner::new())),
        }
    }
    pub fn step_phisics_pipeline(&self) {
        self.inner.lock().unwrap().step_phisics_pipeline();
    }
}
pub struct SimulationStateInner {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    gravity: nalgebra::Matrix<
        f32,
        nalgebra::Const<3>,
        nalgebra::Const<1>,
        nalgebra::ArrayStorage<f32, 3, 1>,
    >,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl SimulationStateInner {
    pub fn new() -> Self {
        SimulationStateInner {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }
    pub fn step_phisics_pipeline(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }
    pub fn add_simulation_object_from_parts(
        &mut self,
        scene_node: SceneNode,
        colider: Collider,
        rigidbody: Option<RigidBody>,
    ) -> SimulationObject {
        match rigidbody {
            None => {
                let colider_handle = self.collider_set.insert(colider);
                SimulationObject::new(scene_node, colider_handle, None)
            }
            Some(rigidbody) => {
                let rigid_body_handle = self.rigid_body_set.insert(rigidbody);
                let colider_handle = self.collider_set.insert_with_parent(
                    colider,
                    rigid_body_handle,
                    &mut self.rigid_body_set,
                );

                SimulationObject::new(scene_node, colider_handle, Some(rigid_body_handle))
            }
        }
    }
}
