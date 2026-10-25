/// Where an entity is on the track.
#[derive(Debug, Clone, Default)]
pub struct TrackPositionData {
    pub z: f32,
    /// Lateral offset in road half-widths; `-1..=1` is on the road.
    pub offset: f32,
    /// How far into its segment the entity is, for interpolating while rendering.
    pub percent: f32,
}
