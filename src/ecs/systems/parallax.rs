//! `Parallax`: scrolls the background layers as the player drives through curves.

use std::convert::Infallible;

use super::origin_segment;
use crate::ecs::{
    ApplyParallaxSystem, BackdropState, ParallaxSystem, StepOriginComponent,
    TrackPositionComponent, TrackState,
};
use crate::race::{HILL_SPEED, SKY_SPEED, TREE_SPEED};
use crate::track::{SEGMENT_LENGTH, increase};

impl ApplyParallaxSystem for ParallaxSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        track: &TrackState,
        backdrop: &mut BackdropState,
        pos: &TrackPositionComponent,
        origin: &StepOriginComponent,
    ) {
        let segment = origin_segment(track, **origin);
        let scroll = segment.curve * (pos.z - **origin) / SEGMENT_LENGTH;
        backdrop.sky = increase(backdrop.sky, SKY_SPEED * scroll, 1.0);
        backdrop.hills = increase(backdrop.hills, HILL_SPEED * scroll, 1.0);
        backdrop.trees = increase(backdrop.trees, TREE_SPEED * scroll, 1.0);
    }
}
