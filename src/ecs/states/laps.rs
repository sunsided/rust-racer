/// Current, last and fastest lap times, in seconds.
#[derive(Debug, Clone, Copy)]
pub struct LapsState {
    pub current: f32,
    pub last: Option<f32>,
    pub fastest: f32,
    /// Whether the last lap set the fastest time.
    pub last_was_fastest: bool,
}

impl Default for LapsState {
    fn default() -> Self {
        Self {
            current: 0.0,
            last: None,
            fastest: 180.0,
            last_was_fastest: false,
        }
    }
}
