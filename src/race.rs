//! Driving, camera and traffic constants shared by the simulation and the renderer.

use crate::track::{SEGMENT_LENGTH, sprites};

/// Simulation step of the `FixedUpdate` phase (see `ecs.yaml`).
pub const STEP: f32 = 1.0 / 60.0;
/// Top speed: never move more than one segment per step, which keeps collisions simple.
pub const MAX_SPEED: f32 = SEGMENT_LENGTH / STEP;
pub const ACCEL: f32 = MAX_SPEED / 5.0;
pub const BREAKING: f32 = -MAX_SPEED;
/// Natural deceleration when neither accelerating nor braking.
pub const DECEL: f32 = -MAX_SPEED / 5.0;
pub const OFF_ROAD_DECEL: f32 = -MAX_SPEED / 2.0;
/// Off-road deceleration stops below this speed.
pub const OFF_ROAD_LIMIT: f32 = MAX_SPEED / 4.0;
/// How hard curves push the car outwards.
pub const CENTRIFUGAL: f32 = 0.3;
pub const TOTAL_CARS: usize = 200;

pub const FIELD_OF_VIEW: f32 = 100.0;
pub const CAMERA_HEIGHT: f32 = 1000.0;
/// Number of segments to draw, and how far ahead traffic bothers to steer.
pub const DRAW_DISTANCE: usize = 300;
pub const FOG_DENSITY: f32 = 5.0;

/// Background layer scroll speeds when going around curves.
pub const SKY_SPEED: f32 = 0.001;
pub const HILL_SPEED: f32 = 0.002;
pub const TREE_SPEED: f32 = 0.003;

/// Distance of the camera to the projection plane.
pub fn camera_depth() -> f32 {
    1.0 / (FIELD_OF_VIEW / 2.0).to_radians().tan()
}

/// Distance of the player's car in front of the camera.
pub fn player_z() -> f32 {
    CAMERA_HEIGHT * camera_depth()
}

/// Width of the player's car in road half-widths.
pub const PLAYER_W: f32 = sprites::PLAYER_STRAIGHT.w * sprites::SCALE;
