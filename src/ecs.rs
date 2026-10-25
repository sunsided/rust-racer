//! The sillyecs world - generated from `ecs.yaml` by `build.rs` - with its components, states
//! and systems. In order, each 60 Hz `FixedUpdate` step:
//!
//! - `Traffic` steers the AI cars around each other and the player, and moves them.
//! - `TrafficIndex` buckets the cars into their track segments in the `TrafficGrid`.
//! - `Driving` applies the player's controls, centrifugal force and off-road drag.
//! - `Collision` stops the player at roadside scenery and behind slower cars.
//! - `LapTimer`, `Parallax` and `Snapshot` then time laps, scroll the background, and publish
//!   the player car for the next step's traffic AI and the renderer.

mod components;
mod states;
mod systems;
mod world;

sillyecs::include_ecs!(
    /// The code sillyecs generates from `ecs.yaml` (see `build.rs`).
    mod generated
);

pub use components::*;
pub use generated::*;
pub use states::*;
pub use world::{RacerWorld, race_states, spawn_racers};
