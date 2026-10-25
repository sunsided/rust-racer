//! `Collision`: stops the player at roadside scenery and behind slower cars.

use std::convert::Infallible;

use super::origin_segment;
use crate::ecs::{
    ApplyCollisionSystem, CollisionSystem, SpeedComponent, StepOriginComponent,
    TrackPositionComponent, TrackState, TrafficGridState,
};
use crate::race::{MAX_SPEED, PLAYER_W, player_z};
use crate::track::{increase, overlap, sprites};

impl ApplyCollisionSystem for CollisionSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        track: &TrackState,
        grid: &TrafficGridState,
        origin: &StepOriginComponent,
        pos: &mut TrackPositionComponent,
        speed: &mut SpeedComponent,
    ) {
        let player_z = player_z();
        let segment = origin_segment(track, **origin);

        if pos.offset < -1.0 || pos.offset > 1.0 {
            for sprite in &segment.sprites {
                let sprite_w = sprite.source.w * sprites::SCALE;
                let side = if sprite.offset > 0.0 { 1.0 } else { -1.0 };
                let center = sprite.offset + sprite_w / 2.0 * side;
                if overlap(pos.offset, PLAYER_W, center, sprite_w, 1.0) {
                    // Stop in front of the sprite, at the front of the segment.
                    **speed = MAX_SPEED / 5.0;
                    pos.z = increase(segment.p1[1], -player_z, track.length());
                    break;
                }
            }
        }

        for car in grid.cars(segment.index) {
            if **speed > car.speed && overlap(pos.offset, PLAYER_W, car.offset, car.width(), 0.8) {
                // Bump into the car's rear and lose speed.
                **speed = car.speed * (car.speed / **speed);
                pos.z = increase(car.z, -player_z, track.length());
                break;
            }
        }
    }
}
