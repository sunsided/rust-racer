//! `Driving`: applies the player's controls, the curves' centrifugal force and off-road drag.

use std::convert::Infallible;

use super::origin_segment;
use crate::ecs::{
    ApplyDrivingSystem, ControlsState, DriverComponent, DrivingSystem, SpeedComponent,
    StepOriginComponent, TrackPositionComponent, TrackState,
};
use crate::race::{
    ACCEL, BREAKING, CENTRIFUGAL, DECEL, MAX_SPEED, OFF_ROAD_DECEL, OFF_ROAD_LIMIT, STEP,
};
use crate::track::increase;

impl ApplyDrivingSystem for DrivingSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        track: &TrackState,
        controls: &ControlsState,
        pos: &mut TrackPositionComponent,
        speed: &mut SpeedComponent,
        driver: &mut DriverComponent,
        origin: &mut StepOriginComponent,
    ) {
        let dt = STEP;
        **origin = pos.z;
        let segment = origin_segment(track, pos.z);
        let speed_percent = **speed / MAX_SPEED;
        // At top speed, crossing the road from left to right takes one second.
        let dx = dt * 2.0 * speed_percent;

        pos.z = increase(pos.z, dt * **speed, track.length());

        if controls.left {
            pos.offset -= dx;
        } else if controls.right {
            pos.offset += dx;
        }
        pos.offset -= dx * speed_percent * segment.curve * CENTRIFUGAL;

        **speed += dt
            * if controls.faster {
                ACCEL
            } else if controls.slower {
                BREAKING
            } else {
                DECEL
            };
        let off_road = pos.offset < -1.0 || pos.offset > 1.0;
        if off_road && **speed > OFF_ROAD_LIMIT {
            **speed += dt * OFF_ROAD_DECEL;
        }

        // Don't ever let the car go too far out of bounds, or exceed the top speed.
        pos.offset = pos.offset.clamp(-3.0, 3.0);
        **speed = speed.clamp(0.0, MAX_SPEED);

        let steer = if controls.left {
            -1.0
        } else if controls.right {
            1.0
        } else {
            0.0
        };
        driver.steer = if **speed > 0.0 { steer } else { 0.0 };
        driver.updown = segment.p2[0] - segment.p1[0];
    }
}
