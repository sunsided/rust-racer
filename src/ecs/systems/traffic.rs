//! `Traffic`: steers the AI cars around slower traffic and the player, and moves them.

use std::convert::Infallible;

use crate::ecs::{
    ApplyTrafficSystem, CarModelComponent, PlayerViewState, SpeedComponent, TrackPositionComponent,
    TrackState, TrafficGridState, TrafficSystem,
};
use crate::race::{DRAW_DISTANCE, MAX_SPEED, PLAYER_W, STEP, player_z};
use crate::track::{SEGMENT_LENGTH, increase, overlap, percent_remaining, sprites};

impl ApplyTrafficSystem for TrafficSystem {
    type Error = Infallible;

    fn apply_many(
        &mut self,
        track: &TrackState,
        grid: &TrafficGridState,
        player: &PlayerViewState,
        speeds: &[SpeedComponent],
        car_models: &[CarModelComponent],
        track_positions: &mut [TrackPositionComponent],
    ) {
        let track_length = track.length();
        let ahead = Ahead {
            track,
            grid,
            player,
            player_segment: track.find(player.position + player_z()),
        };
        for ((speed, model), pos) in speeds.iter().zip(car_models).zip(track_positions) {
            let speed = **speed;
            let car_segment = track.find(pos.z);
            pos.offset += ahead.steer_around(car_segment, speed, pos.offset, model.w);
            pos.z = increase(pos.z, STEP * speed, track_length);
            pos.percent = percent_remaining(pos.z, SEGMENT_LENGTH);
        }
    }
}

/// What an AI car can see of the road ahead.
struct Ahead<'a> {
    track: &'a TrackState,
    grid: &'a TrafficGridState,
    player: &'a PlayerViewState,
    player_segment: usize,
}

impl Ahead<'_> {
    /// How far a car at `offset` swerves this step to get around slower cars (or the player)
    /// in the next few segments, or to get back onto the road.
    fn steer_around(&self, car_segment: usize, speed: f32, offset: f32, sprite_w: f32) -> f32 {
        const LOOKAHEAD: usize = 20;

        // Don't bother steering around other cars when out of sight of the player.
        if car_segment as isize - self.player_segment as isize > DRAW_DISTANCE as isize {
            return 0.0;
        }

        let player = self.player;
        let car_w = sprite_w * sprites::SCALE;
        let segments = self.track.len();
        for i in 1..LOOKAHEAD {
            let segment = (car_segment + i) % segments;

            // The closer the obstacle (smaller `i`) and the bigger the speed difference, the
            // harder the swerve.
            if segment == self.player_segment
                && speed > player.speed
                && overlap(player.x, PLAYER_W, offset, car_w, 1.2)
            {
                let dir = swerve_direction(offset, player.x);
                return dir / i as f32 * (speed - player.speed) / MAX_SPEED;
            }

            for other in self.grid.cars(segment) {
                if speed > other.speed && overlap(offset, car_w, other.offset, other.width(), 1.2) {
                    let dir = swerve_direction(offset, other.offset);
                    return dir / i as f32 * (speed - other.speed) / MAX_SPEED;
                }
            }
        }

        // No cars ahead, but somehow ended up off road: steer back on.
        if offset < -0.9 {
            0.1
        } else if offset > 0.9 {
            -0.1
        } else {
            0.0
        }
    }
}

/// Pass an obstacle on the inside when it is near an edge, else on whichever side we are.
fn swerve_direction(offset: f32, obstacle: f32) -> f32 {
    if obstacle > 0.5 {
        -1.0
    } else if obstacle < -0.5 || offset > obstacle {
        1.0
    } else {
        -1.0
    }
}
