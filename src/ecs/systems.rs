//! The systems' implementations, one module each, and the factory that creates them.

use crate::ecs::{
    CollisionSystem, CreateSystem, DrivingSystem, LapTimerSystem, ParallaxSystem, SnapshotSystem,
    TrackState, TrafficIndexSystem, TrafficSystem,
};
use crate::race::player_z;
use crate::track::Segment;

mod collision;
mod driving;
mod lap_timer;
mod parallax;
mod snapshot;
mod traffic;
mod traffic_index;

/// Declares the (stateless) data of each system and lets [`SystemFactory`] create it.
macro_rules! stateless_systems {
    ($($system:ident($data:ident)),* $(,)?) => {
        $(
            #[derive(Debug, Default)]
            pub struct $data;

            impl CreateSystem<$system> for SystemFactory {
                fn create(&self) -> $system {
                    $system::from($data)
                }
            }
        )*
    };
}

/// Builds the system instances for the world.
pub struct SystemFactory;

stateless_systems! {
    TrafficSystem(TrafficSystemData),
    TrafficIndexSystem(TrafficIndexSystemData),
    DrivingSystem(DrivingSystemData),
    CollisionSystem(CollisionSystemData),
    LapTimerSystem(LapTimerSystemData),
    ParallaxSystem(ParallaxSystemData),
    SnapshotSystem(SnapshotSystemData),
}

/// The segment the player's car was in when the step began. The JS original evaluates the
/// whole step against it, so all player systems do too.
fn origin_segment(track: &TrackState, origin: f32) -> &Segment {
    &track.segments[track.find(origin + player_z())]
}
