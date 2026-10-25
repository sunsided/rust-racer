//! `TrafficIndex`: buckets the AI cars into their track segments.

use std::convert::Infallible;

use crate::ecs::{
    ApplyTrafficIndexSystem, CarModelComponent, SpeedComponent, TrackPositionComponent, TrackState,
    TrafficCar, TrafficGridState, TrafficIndexSystem,
};

impl ApplyTrafficIndexSystem for TrafficIndexSystem {
    type Error = Infallible;

    fn on_begin_phase(
        &mut self,
        _track: &TrackState,
        grid: &mut TrafficGridState,
    ) -> Result<(), Self::Error> {
        grid.clear();
        Ok(())
    }

    fn apply_many(
        &mut self,
        track: &TrackState,
        grid: &mut TrafficGridState,
        track_positions: &[TrackPositionComponent],
        speeds: &[SpeedComponent],
        car_models: &[CarModelComponent],
    ) {
        let cars = track_positions.iter().zip(speeds).zip(car_models);
        for ((pos, speed), model) in cars {
            let car = TrafficCar {
                z: pos.z,
                offset: pos.offset,
                percent: pos.percent,
                speed: **speed,
                sprite: **model,
            };
            grid.insert(track.find(pos.z), car);
        }
    }
}
