/// Held driving controls, sampled once per frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct ControlsState {
    pub left: bool,
    pub right: bool,
    pub faster: bool,
    pub slower: bool,
}
