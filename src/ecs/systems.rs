//! The systems' implementations, one module each. All systems are stateless, so sillyecs
//! generates their data and construction.

use crate::ecs::TrackState;
use crate::race::player_z;
use crate::track::Segment;

mod collision;
mod driving;
mod lap_timer;
mod parallax;
mod snapshot;
mod traffic;
mod traffic_index;

/// The segment the player's car was in when the step began. The JS original evaluates the
/// whole step against it, so all player systems do too.
fn origin_segment(track: &TrackState, origin: f32) -> &Segment {
    &track.segments[track.find(origin + player_z())]
}
