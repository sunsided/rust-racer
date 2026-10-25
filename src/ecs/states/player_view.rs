/// The player car as of the last step, for the traffic AI and rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerViewState {
    /// Camera z; the car itself is `player_z()` further ahead.
    pub position: f32,
    pub x: f32,
    pub speed: f32,
    pub steer: f32,
    pub updown: f32,
}
