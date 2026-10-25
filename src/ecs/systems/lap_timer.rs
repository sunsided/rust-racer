//! `LapTimer`: times laps as the player crosses the start line.

use std::convert::Infallible;

use crate::ecs::{
    ApplyLapTimerSystem, LapTimerSystem, LapsState, StepOriginComponent, TrackPositionComponent,
};
use crate::race::{STEP, player_z};

impl ApplyLapTimerSystem for LapTimerSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        laps: &mut LapsState,
        pos: &TrackPositionComponent,
        origin: &StepOriginComponent,
    ) {
        let player_z = player_z();
        if pos.z <= player_z {
            return;
        }
        let crossed_start = **origin < player_z;
        if laps.current > 0.0 && crossed_start {
            laps.last_was_fastest = laps.current <= laps.fastest;
            if laps.last_was_fastest {
                laps.fastest = laps.current;
            }
            laps.last = Some(laps.current);
            laps.current = 0.0;
        } else {
            laps.current += STEP;
        }
    }
}
