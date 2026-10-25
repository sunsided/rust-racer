//! `Snapshot`: publishes the player car's state for the traffic AI and the renderer.

use std::convert::Infallible;

use crate::ecs::{
    ApplySnapshotSystem, DriverComponent, PlayerViewState, SnapshotSystem, SpeedComponent,
    TrackPositionComponent,
};

impl ApplySnapshotSystem for SnapshotSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        view: &mut PlayerViewState,
        pos: &TrackPositionComponent,
        speed: &SpeedComponent,
        driver: &DriverComponent,
    ) {
        *view = PlayerViewState {
            position: pos.z,
            x: pos.offset,
            speed: **speed,
            steer: driver.steer,
            updown: driver.updown,
        };
    }
}
