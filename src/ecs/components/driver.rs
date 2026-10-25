/// What the player is doing with the car, which picks the sprite variant.
#[derive(Debug, Clone, Default)]
pub struct DriverData {
    /// `-1` steering left, `1` right, `0` straight (or standing still).
    pub steer: f32,
    /// Height change across the current segment; positive is uphill.
    pub updown: f32,
}
