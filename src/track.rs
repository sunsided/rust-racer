//! Road geometry, the sprite atlas layout, and the track builder (`resetRoad` in the original).

use std::f32::consts::PI;

use game_toolkit::prelude::*;

/// Length of a single road segment, in world units.
pub const SEGMENT_LENGTH: f32 = 200.0;
/// Number of segments per light/dark rumble strip.
pub const RUMBLE_LENGTH: usize = 3;
/// Half the road's width; the road spans `-ROAD_WIDTH..=ROAD_WIDTH`.
pub const ROAD_WIDTH: f32 = 2000.0;
pub const LANES: usize = 3;

/// A rectangle in one of the texture atlases, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

const fn rect(x: f32, y: f32, w: f32, h: f32) -> AtlasRect {
    AtlasRect { x, y, w, h }
}

pub const HILLS: AtlasRect = rect(5.0, 5.0, 1280.0, 480.0);
pub const SKY: AtlasRect = rect(5.0, 495.0, 1280.0, 480.0);
pub const TREES: AtlasRect = rect(5.0, 985.0, 1280.0, 480.0);

pub mod sprites {
    use super::{AtlasRect, rect};

    pub const PALM_TREE: AtlasRect = rect(5.0, 5.0, 215.0, 540.0);
    pub const BILLBOARD08: AtlasRect = rect(230.0, 5.0, 385.0, 265.0);
    pub const TREE1: AtlasRect = rect(625.0, 5.0, 360.0, 360.0);
    pub const DEAD_TREE1: AtlasRect = rect(5.0, 555.0, 135.0, 332.0);
    pub const BILLBOARD09: AtlasRect = rect(150.0, 555.0, 328.0, 282.0);
    pub const BOULDER3: AtlasRect = rect(230.0, 280.0, 320.0, 220.0);
    pub const COLUMN: AtlasRect = rect(995.0, 5.0, 200.0, 315.0);
    pub const BILLBOARD01: AtlasRect = rect(625.0, 375.0, 300.0, 170.0);
    pub const BILLBOARD06: AtlasRect = rect(488.0, 555.0, 298.0, 190.0);
    pub const BILLBOARD05: AtlasRect = rect(5.0, 897.0, 298.0, 190.0);
    pub const BILLBOARD07: AtlasRect = rect(313.0, 897.0, 298.0, 190.0);
    pub const BOULDER2: AtlasRect = rect(621.0, 897.0, 298.0, 140.0);
    pub const TREE2: AtlasRect = rect(1205.0, 5.0, 282.0, 295.0);
    pub const BILLBOARD04: AtlasRect = rect(1205.0, 310.0, 268.0, 170.0);
    pub const DEAD_TREE2: AtlasRect = rect(1205.0, 490.0, 150.0, 260.0);
    pub const BOULDER1: AtlasRect = rect(1205.0, 760.0, 168.0, 248.0);
    pub const BUSH1: AtlasRect = rect(5.0, 1097.0, 240.0, 155.0);
    pub const CACTUS: AtlasRect = rect(929.0, 897.0, 235.0, 118.0);
    pub const BUSH2: AtlasRect = rect(255.0, 1097.0, 232.0, 152.0);
    pub const BILLBOARD03: AtlasRect = rect(5.0, 1262.0, 230.0, 220.0);
    pub const BILLBOARD02: AtlasRect = rect(245.0, 1262.0, 215.0, 220.0);
    pub const STUMP: AtlasRect = rect(995.0, 330.0, 195.0, 140.0);
    pub const SEMI: AtlasRect = rect(1365.0, 490.0, 122.0, 144.0);
    pub const TRUCK: AtlasRect = rect(1365.0, 644.0, 100.0, 78.0);
    pub const CAR03: AtlasRect = rect(1383.0, 760.0, 88.0, 55.0);
    pub const CAR02: AtlasRect = rect(1383.0, 825.0, 80.0, 59.0);
    pub const CAR04: AtlasRect = rect(1383.0, 894.0, 80.0, 57.0);
    pub const CAR01: AtlasRect = rect(1205.0, 1018.0, 80.0, 56.0);
    pub const PLAYER_UPHILL_LEFT: AtlasRect = rect(1383.0, 961.0, 80.0, 45.0);
    pub const PLAYER_UPHILL_STRAIGHT: AtlasRect = rect(1295.0, 1018.0, 80.0, 45.0);
    pub const PLAYER_UPHILL_RIGHT: AtlasRect = rect(1385.0, 1018.0, 80.0, 45.0);
    pub const PLAYER_LEFT: AtlasRect = rect(995.0, 480.0, 80.0, 41.0);
    pub const PLAYER_STRAIGHT: AtlasRect = rect(1085.0, 480.0, 80.0, 41.0);
    pub const PLAYER_RIGHT: AtlasRect = rect(995.0, 531.0, 80.0, 41.0);

    /// World units per sprite pixel: the player sprite is 1/3rd of the (half-)road width.
    pub const SCALE: f32 = 0.3 * (1.0 / PLAYER_STRAIGHT.w);

    pub const BILLBOARDS: [AtlasRect; 9] = [
        BILLBOARD01,
        BILLBOARD02,
        BILLBOARD03,
        BILLBOARD04,
        BILLBOARD05,
        BILLBOARD06,
        BILLBOARD07,
        BILLBOARD08,
        BILLBOARD09,
    ];
    pub const PLANTS: [AtlasRect; 12] = [
        TREE1, TREE2, DEAD_TREE1, DEAD_TREE2, PALM_TREE, BUSH1, BUSH2, CACTUS, STUMP, BOULDER1,
        BOULDER2, BOULDER3,
    ];
    pub const CARS: [AtlasRect; 6] = [CAR01, CAR02, CAR03, CAR04, SEMI, TRUCK];
}

/// The colors of one road segment. `lane` is `None` where no lane markers are drawn.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub road: [f32; 4],
    pub grass: [f32; 4],
    pub rumble: [f32; 4],
    pub lane: Option<[f32; 4]>,
}

const fn hex(rgb: u32) -> [f32; 4] {
    [
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        1.0,
    ]
}

pub const SKY_COLOR: [f32; 4] = hex(0x72D7EE);
pub const FOG_COLOR: [f32; 4] = hex(0x005108);

const LIGHT: Palette = Palette {
    road: hex(0x6B6B6B),
    grass: hex(0x10AA10),
    rumble: hex(0x555555),
    lane: Some(hex(0xCCCCCC)),
};
const DARK: Palette = Palette {
    road: hex(0x696969),
    grass: hex(0x009A00),
    rumble: hex(0xBBBBBB),
    lane: None,
};
const START: Palette = Palette {
    road: hex(0xFFFFFF),
    grass: hex(0xFFFFFF),
    rumble: hex(0xFFFFFF),
    lane: None,
};
const FINISH: Palette = Palette {
    road: hex(0x000000),
    grass: hex(0x000000),
    rumble: hex(0x000000),
    lane: None,
};

/// A roadside sprite. `offset` is in road half-widths from the center (negative = left).
#[derive(Debug, Clone, Copy)]
pub struct Scenery {
    pub source: AtlasRect,
    pub offset: f32,
}

/// One slice of road between two world points `(y, z)`.
#[derive(Debug, Clone)]
pub struct Segment {
    pub index: usize,
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub curve: f32,
    pub sprites: Vec<Scenery>,
    pub color: Palette,
}

#[derive(Debug, Default)]
pub struct Track {
    pub segments: Vec<Segment>,
}

impl Track {
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Total z length of the track.
    pub fn length(&self) -> f32 {
        self.segments.len() as f32 * SEGMENT_LENGTH
    }

    /// Index of the segment containing world position `z`.
    pub fn find(&self, z: f32) -> usize {
        (z / SEGMENT_LENGTH).floor() as usize % self.segments.len()
    }

    /// Builds the full lap: the road shape, roadside scenery, and start/finish lines.
    /// `player_z` is the player's distance in front of the camera.
    pub fn build(rng: &mut Rng, player_z: f32) -> Self {
        let mut t = Self::default();

        t.add_straight(LENGTH_SHORT);
        t.add_low_rolling_hills(LENGTH_SHORT, HILL_LOW);
        t.add_s_curves();
        t.add_curve(LENGTH_MEDIUM, CURVE_MEDIUM, HILL_LOW);
        t.add_bumps();
        t.add_low_rolling_hills(LENGTH_SHORT, HILL_LOW);
        t.add_curve(LENGTH_LONG * 2, CURVE_MEDIUM, HILL_MEDIUM);
        t.add_straight(LENGTH_MEDIUM);
        t.add_hill(LENGTH_MEDIUM, HILL_HIGH);
        t.add_s_curves();
        t.add_curve(LENGTH_LONG, -CURVE_MEDIUM, HILL_NONE);
        t.add_hill(LENGTH_LONG, HILL_HIGH);
        t.add_curve(LENGTH_LONG, CURVE_MEDIUM, -HILL_LOW);
        t.add_bumps();
        t.add_hill(LENGTH_LONG, -HILL_MEDIUM);
        t.add_straight(LENGTH_MEDIUM);
        t.add_s_curves();
        t.add_downhill_to_end(200);

        t.add_scenery(rng);

        let start = t.find(player_z);
        t.segments[start + 2].color = START;
        t.segments[start + 3].color = START;
        let n = t.segments.len();
        for segment in &mut t.segments[n - RUMBLE_LENGTH..] {
            segment.color = FINISH;
        }
        t
    }

    fn last_y(&self) -> f32 {
        self.segments.last().map_or(0.0, |s| s.p2[0])
    }

    fn add_segment(&mut self, curve: f32, y: f32) {
        let n = self.segments.len();
        self.segments.push(Segment {
            index: n,
            p1: [self.last_y(), n as f32 * SEGMENT_LENGTH],
            p2: [y, (n + 1) as f32 * SEGMENT_LENGTH],
            curve,
            sprites: Vec::new(),
            color: if (n / RUMBLE_LENGTH) % 2 == 1 {
                DARK
            } else {
                LIGHT
            },
        });
    }

    /// Adds `enter + hold + leave` segments easing into `curve` and up/down by `hills`
    /// segment lengths.
    fn add_road(&mut self, enter: usize, hold: usize, leave: usize, curve: f32, hills: f32) {
        let start_y = self.last_y();
        let end_y = start_y + hills.trunc() * SEGMENT_LENGTH;
        let total = (enter + hold + leave) as f32;
        for n in 0..enter {
            let p = n as f32 / enter as f32;
            let y = ease_in_out(start_y, end_y, n as f32 / total);
            self.add_segment(ease_in(0.0, curve, p), y);
        }
        for n in 0..hold {
            let y = ease_in_out(start_y, end_y, (enter + n) as f32 / total);
            self.add_segment(curve, y);
        }
        for n in 0..leave {
            let p = n as f32 / leave as f32;
            let y = ease_in_out(start_y, end_y, (enter + hold + n) as f32 / total);
            self.add_segment(ease_in_out(curve, 0.0, p), y);
        }
    }

    fn add_straight(&mut self, num: usize) {
        self.add_road(num, num, num, 0.0, 0.0);
    }

    fn add_hill(&mut self, num: usize, height: f32) {
        self.add_road(num, num, num, 0.0, height);
    }

    fn add_curve(&mut self, num: usize, curve: f32, height: f32) {
        self.add_road(num, num, num, curve, height);
    }

    fn add_low_rolling_hills(&mut self, num: usize, height: f32) {
        self.add_road(num, num, num, 0.0, height / 2.0);
        self.add_road(num, num, num, 0.0, -height);
        self.add_road(num, num, num, CURVE_EASY, height);
        self.add_road(num, num, num, 0.0, 0.0);
        self.add_road(num, num, num, -CURVE_EASY, height / 2.0);
        self.add_road(num, num, num, 0.0, 0.0);
    }

    fn add_s_curves(&mut self) {
        let m = LENGTH_MEDIUM;
        self.add_road(m, m, m, -CURVE_EASY, HILL_NONE);
        self.add_road(m, m, m, CURVE_MEDIUM, HILL_MEDIUM);
        self.add_road(m, m, m, CURVE_EASY, -HILL_LOW);
        self.add_road(m, m, m, -CURVE_EASY, HILL_MEDIUM);
        self.add_road(m, m, m, -CURVE_MEDIUM, -HILL_MEDIUM);
    }

    fn add_bumps(&mut self) {
        for height in [5.0, -2.0, -5.0, 8.0, 5.0, -7.0, 5.0, -2.0] {
            self.add_road(10, 10, 10, 0.0, height);
        }
    }

    fn add_downhill_to_end(&mut self, num: usize) {
        let hills = -self.last_y() / SEGMENT_LENGTH;
        self.add_road(num, num, num, -CURVE_EASY, hills);
    }

    fn add_sprite(&mut self, n: usize, source: AtlasRect, offset: f32) {
        self.segments[n].sprites.push(Scenery { source, offset });
    }

    fn add_scenery(&mut self, rng: &mut Rng) {
        use sprites::*;

        let billboards = [
            BILLBOARD07,
            BILLBOARD06,
            BILLBOARD08,
            BILLBOARD09,
            BILLBOARD01,
            BILLBOARD02,
            BILLBOARD03,
            BILLBOARD04,
            BILLBOARD05,
        ];
        for (i, billboard) in billboards.into_iter().enumerate() {
            self.add_sprite(20 + 20 * i, billboard, -1.0);
        }

        let end = self.segments.len() - 25;
        for n in [240, end] {
            self.add_sprite(n, BILLBOARD07, -1.2);
            self.add_sprite(n, BILLBOARD06, 1.2);
        }

        let mut n = 10;
        while n < 200 {
            self.add_sprite(n, PALM_TREE, 0.5 + rng.next_f32() * 0.5);
            self.add_sprite(n, PALM_TREE, 1.0 + rng.next_f32() * 2.0);
            n += 4 + n / 100;
        }

        for n in (250..1000).step_by(5) {
            self.add_sprite(n, COLUMN, 1.1);
            let i = n + rng.range_usize(0..6);
            self.add_sprite(i, TREE1, -1.0 - rng.next_f32() * 2.0);
            let i = n + rng.range_usize(0..6);
            self.add_sprite(i, TREE2, -1.0 - rng.next_f32() * 2.0);
        }

        for n in (200..self.segments.len()).step_by(3) {
            let plant = PLANTS[rng.range_usize(0..PLANTS.len())];
            self.add_sprite(n, plant, random_side(rng) * (2.0 + rng.next_f32() * 5.0));
        }

        for n in (1000..self.segments.len() - 50).step_by(100) {
            let side = random_side(rng);
            let billboard = BILLBOARDS[rng.range_usize(0..BILLBOARDS.len())];
            let i = n + rng.range_usize(0..51);
            self.add_sprite(i, billboard, -side);
            for _ in 0..20 {
                let plant = PLANTS[rng.range_usize(0..PLANTS.len())];
                let i = n + rng.range_usize(0..51);
                self.add_sprite(i, plant, side * (1.5 + rng.next_f32()));
            }
        }
    }
}

const LENGTH_SHORT: usize = 25;
const LENGTH_MEDIUM: usize = 50;
const LENGTH_LONG: usize = 100;
const HILL_NONE: f32 = 0.0;
const HILL_LOW: f32 = 20.0;
const HILL_MEDIUM: f32 = 40.0;
const HILL_HIGH: f32 = 60.0;
const CURVE_EASY: f32 = 2.0;
const CURVE_MEDIUM: f32 = 4.0;

/// `-1.0` or `1.0`, with equal odds.
pub fn random_side(rng: &mut Rng) -> f32 {
    if rng.next_f32() < 0.5 { -1.0 } else { 1.0 }
}

pub fn interpolate(a: f32, b: f32, percent: f32) -> f32 {
    a + (b - a) * percent
}

fn ease_in(a: f32, b: f32, percent: f32) -> f32 {
    a + (b - a) * percent * percent
}

fn ease_in_out(a: f32, b: f32, percent: f32) -> f32 {
    a + (b - a) * (-(percent * PI).cos() / 2.0 + 0.5)
}

/// `start + increment`, wrapped into `0..max`.
pub fn increase(start: f32, increment: f32, max: f32) -> f32 {
    let mut result = start + increment;
    while result >= max {
        result -= max;
    }
    while result < 0.0 {
        result += max;
    }
    result
}

/// How far `n` is into its current `total`-sized chunk, as `0.0..1.0`.
pub fn percent_remaining(n: f32, total: f32) -> f32 {
    (n % total) / total
}

/// Whether the spans centered at `x1` and `x2` overlap, with widths scaled by `percent`.
pub fn overlap(x1: f32, w1: f32, x2: f32, w2: f32, percent: f32) -> bool {
    let half = percent / 2.0;
    let (min1, max1) = (x1 - w1 * half, x1 + w1 * half);
    let (min2, max2) = (x2 - w2 * half, x2 + w2 * half);
    !(max1 < min2 || min1 > max2)
}
