//! The shared `Race` ECS state, the component payloads, and the driving constants.

use crate::track::{AtlasRect, SEGMENT_LENGTH, Track, sprites};

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

/// Where an entity is on the track.
#[derive(Debug, Clone, Default)]
pub struct TrackPositionData {
    pub z: f32,
    /// Lateral offset in road half-widths; `-1..=1` is on the road.
    pub offset: f32,
    /// How far into its segment the entity is, for interpolating while rendering.
    pub percent: f32,
}

pub type SpeedData = f32;

/// The sprite an AI car is drawn with (also determines its width).
pub type CarModelData = AtlasRect;

/// What the player is doing with the car, which picks the sprite variant.
#[derive(Debug, Clone, Default)]
pub struct DriverData {
    /// `-1` steering left, `1` right, `0` straight (or standing still).
    pub steer: f32,
    /// Height change across the current segment; positive is uphill.
    pub updown: f32,
}

/// Held driving controls, sampled once per frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct DriverInput {
    pub left: bool,
    pub right: bool,
    pub faster: bool,
    pub slower: bool,
}

/// The player car as last simulated, for traffic AI and rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerSnapshot {
    /// Camera z; the car itself is `player_z()` further ahead.
    pub position: f32,
    pub x: f32,
    pub speed: f32,
    pub steer: f32,
    pub updown: f32,
}

/// An AI car as indexed into its track segment.
#[derive(Debug, Clone, Copy)]
pub struct TrafficCar {
    pub z: f32,
    pub offset: f32,
    pub percent: f32,
    pub speed: f32,
    pub sprite: AtlasRect,
}

impl TrafficCar {
    pub fn width(&self) -> f32 {
        self.sprite.w * sprites::SCALE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LapTimes {
    pub current: f32,
    pub last: Option<f32>,
    pub fastest: f32,
    /// Whether the last lap set the fastest time.
    pub last_was_fastest: bool,
}

impl Default for LapTimes {
    fn default() -> Self {
        Self {
            current: 0.0,
            last: None,
            fastest: 180.0,
            last_was_fastest: false,
        }
    }
}

/// Everything the systems share: the track, the player's input and snapshot, traffic
/// indexed per segment, background scroll, and lap timing.
#[derive(Debug, Default)]
pub struct RaceState {
    pub track: Track,
    pub input: DriverInput,
    pub player: PlayerSnapshot,
    /// The AI cars in each segment, rebuilt every step by the `TrafficIndex` system.
    pub traffic: Vec<Vec<TrafficCar>>,
    /// Scroll offsets (`0..1`) of the sky, hill and tree layers.
    pub sky_offset: f32,
    pub hill_offset: f32,
    pub tree_offset: f32,
    pub laps: LapTimes,
}

impl RaceState {
    pub fn new(track: Track) -> Self {
        Self {
            traffic: vec![Vec::new(); track.len()],
            track,
            ..Default::default()
        }
    }
}
