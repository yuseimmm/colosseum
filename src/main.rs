use crate::{
    app::SimulationState,
    command::{Command, CommandExecutor, CommandListener},
};
use kiss3d::{camera::ArcBall, light::Light, window::Window};
use rapier3d::prelude::*;
use std::sync::Arc;
use zenoh::Config;
mod app;
mod command;
mod simulation_object;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Initialize the log (if the environment variable RUST_LOG does not exist, log output will be "info")
    zenoh::init_log_from_env_or("info");
    println!("Initializing camera...");
    // Camera
    let mut camera = ArcBall::new(
        kiss3d::nalgebra::Point3::new(-1.0, 0.5, -2.0),
        kiss3d::nalgebra::Point3::new(0.0, 0.0, 0.0),
    );
    println!("Initializing window...");
    // Window
    let mut window = Window::new("colosseum");
    window.set_light(Light::StickToCamera);

    println!("Initializing command sys...");
    // Command Sys
    let command_executor = CommandExecutor::new();
    let mut command_listener = CommandListener::new("robot/command".to_string(), &command_executor);

    // State. This will store the state of our physical sumilation.
    let sumilation_state = SimulationState::new();

    //Ground
    println!("Creating a ground...");

    let mut ground_scene_node = window.add_quad(5.0, 5.0, 50, 50);
    ground_scene_node.set_lines_color(Some(kiss3d::nalgebra::Point3::new(0.0, 1.0, 0.0)));
    ground_scene_node.set_lines_width(1.0);
    ground_scene_node.set_local_rotation(kiss3d::nalgebra::Unit::from_quaternion(
        kiss3d::nalgebra::Quaternion::new(
            (2.0 as f32).sqrt() / 2.0,
            (2.0 as f32).sqrt() / 2.0,
            0.0,
            0.0,
        ),
    ));
    ground_scene_node.set_surface_rendering_activation(false);

    sumilation_state
        .lock()
        .unwrap()
        .add_simulation_object_from_parts(
            ground_scene_node,
            ColliderBuilder::cuboid(5.0, 1.0, 5.0)
                .translation(vector![0.0, -1.0, 0.0])
                .build(),
            None,
        );
    // Ball
    println!("Creating a ball...");

    let mut ball_scene_node = window.add_capsule(0.05, 0.0);
    ball_scene_node.set_color(1.0, 0.0, 0.0);

    let mut ball = sumilation_state
        .lock()
        .unwrap()
        .add_simulation_object_from_parts(
            ball_scene_node,
            ColliderBuilder::ball(0.05).restitution(0.7).build(),
            Some(
                RigidBodyBuilder::dynamic()
                    .translation(vector![0.0, 3.0, 0.0])
                    .build(),
            ),
        );

    println!("Opening session...");
    let session = zenoh::open(Config::default()).await.unwrap();

    // Command Handler
    {
        let ball_handler = ball.rigid_body.clone().unwrap();
        let _sumilation_state = Arc::clone(&sumilation_state);
        command_executor.add_command(
            "ADD_FORCE_TO_BALL".to_string(),
            Command::new(move |arg| {
                let rigid_body_set = &mut _sumilation_state.lock().unwrap().rigid_body_set;
                let ball_rigid_body = rigid_body_set.get_mut(ball_handler).unwrap();

                ball_rigid_body.apply_impulse(Vector::from_vec(arg), true);

                None
            }),
        );
    }

    command_listener.start(&session).await;

    // Simulation Loop
    while window.render_with_camera(&mut camera) {
        sumilation_state.step_phisics_pipeline();
        ball.synchronize_graphics(&sumilation_state);
    }
}
