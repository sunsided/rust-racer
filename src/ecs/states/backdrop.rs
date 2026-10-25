/// Scroll offsets (`0..1`) of the sky, hill and tree background layers.
#[derive(Debug, Clone, Copy, Default)]
pub struct BackdropState {
    pub sky: f32,
    pub hills: f32,
    pub trees: f32,
}
