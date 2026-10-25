//! The concrete world type, its command queue, and setting up a race.

use std::convert::Infallible;

use game_toolkit::prelude::*;

use super::*;
use crate::race::{MAX_SPEED, TOTAL_CARS};
use crate::track::{SEGMENT_LENGTH, Track, random_side, sprites};

/// The racer has no game-specific world commands.
type RacerCommand = WorldCommand<Infallible>;

pub type RacerWorld = MainWorld<NoOpPhaseEvents, CommandQueue>;

/// The world's command queue: the toolkit's mpsc-backed `ChannelQueue`, implementing sillyecs'
/// queue traits. A newer game-toolkit implements them on `ChannelQueue` itself
/// (sunsided/game-toolkit#61); until it's released, this wrapper bridges the gap.
#[derive(Default)]
pub struct CommandQueue(ChannelQueue<RacerCommand>);

impl sillyecs::CommandSender for CommandQueue {
    type Command = RacerCommand;
    type Error = std::sync::mpsc::SendError<RacerCommand>;

    fn send(&self, command: RacerCommand) -> Result<(), Self::Error> {
        self.0.send(command)
    }
}

impl sillyecs::CommandReceiver for CommandQueue {
    type Command = RacerCommand;
    type Error = std::sync::mpsc::TryRecvError;

    fn recv(&self) -> Result<Option<RacerCommand>, Self::Error> {
        self.0.try_recv()
    }
}

impl WorldUserCommandHandler for RacerWorld {
    fn handle_user_command(&mut self, command: Infallible) {
        match command {}
    }
}

/// Builds the world's states for a race on `track`.
pub fn race_states(track: Track) -> MainWorldStates {
    MainWorldStates {
        traffic_grid: TrafficGridState::new(track.len()),
        track,
        ..Default::default()
    }
}

/// Puts the player at the start line and scatters the AI cars around the track.
pub fn spawn_racers(world: &mut RacerWorld, rng: &mut Rng) {
    world.spawn_player_with(
        TrackPositionComponent::new(TrackPositionData::default()),
        SpeedComponent::new(0.0),
        DriverComponent::new(DriverData::default()),
        StepOriginComponent::new(0.0),
    );

    let segments = world.states.track.len();
    for _ in 0..TOTAL_CARS {
        let offset = rng.next_f32() * 0.8 * random_side(rng);
        let z = rng.range_usize(0..segments) as f32 * SEGMENT_LENGTH;
        let sprite = sprites::CARS[rng.range_usize(0..sprites::CARS.len())];
        let spread = if sprite == sprites::SEMI { 4.0 } else { 2.0 };
        let speed = MAX_SPEED / 4.0 + rng.next_f32() * MAX_SPEED / spread;
        world.spawn_car_with(
            TrackPositionComponent::new(TrackPositionData {
                z,
                offset,
                percent: 0.0,
            }),
            SpeedComponent::new(speed),
            CarModelComponent::new(sprite),
        );
    }
}
