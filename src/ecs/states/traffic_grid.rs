use crate::track::{AtlasRect, sprites};

/// An AI car as indexed into its track segment.
#[derive(Debug, Clone, Copy)]
pub struct TrafficCar {
    pub z: f32,
    pub offset: f32,
    pub percent: f32,
    pub speed: f32,
    pub sprite: AtlasRect,
}

impl TrafficCar {
    pub fn width(&self) -> f32 {
        self.sprite.w * sprites::SCALE
    }
}

/// The AI cars in each track segment, rebuilt every step by the `TrafficIndex` system.
#[derive(Debug, Default)]
pub struct TrafficGridState {
    segments: Vec<Vec<TrafficCar>>,
}

impl TrafficGridState {
    pub fn new(segments: usize) -> Self {
        Self {
            segments: vec![Vec::new(); segments],
        }
    }

    pub fn cars(&self, segment: usize) -> &[TrafficCar] {
        &self.segments[segment]
    }

    pub fn clear(&mut self) {
        for cars in &mut self.segments {
            cars.clear();
        }
    }

    pub fn insert(&mut self, segment: usize, car: TrafficCar) {
        self.segments[segment].push(car);
    }
}
