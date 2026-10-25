# The road is a list of segments

The whole track is a `Vec<Segment>`, and a segment is a slice of road 200 world units long,
described by the two points at its near and far edge:

```rust
pub const SEGMENT_LENGTH: f32 = 200.0;
pub const ROAD_WIDTH: f32 = 2000.0; // half the width: the road spans -2000..=2000

pub struct Segment {
    pub index: usize,
    pub p1: [f32; 2], // (y, z) of the near edge
    pub p2: [f32; 2], // (y, z) of the far edge
    pub curve: f32,
    pub sprites: Vec<Scenery>,
    pub color: Palette,
}
```

([`src/track.rs`](https://github.com/sunsided/rust-racer/blob/main/src/track.rs))

Notice what's missing: there is no x. Every segment sits on the same center line, and `z` is
simply `index * SEGMENT_LENGTH`. The only things that vary from slice to slice are the height
`y` of its two edges (for hills), a `curve` value (which the renderer turns into a bend; see
[curves](curves.md)), the sprites standing next to it, and its colors. Jake keeps `p1` and `p2`
separate even though each segment's `p2` equals the next one's `p1`, so every segment can be
projected on its own; I kept that, too.

The track loops. Finding the segment under a world position is a division and a modulo, and
everything that moves along the track wraps its `z` back into `0..track.length()` with a little
helper called `increase`:

```rust
pub fn find(&self, z: f32) -> usize {
    (z / SEGMENT_LENGTH).floor() as usize % self.segments.len()
}

pub fn increase(start: f32, increment: f32, max: f32) -> f32 {
    let mut result = start + increment;
    while result >= max { result -= max; }
    while result < 0.0 { result += max; }
    result
}
```

The finished track has 6,705 segments, i.e. 1,341,000 world units per lap.

## Rumble strips and the palette

Each segment carries a `Palette` of four colors (road, grass, rumble strip, and an optional lane
marker color), and segments alternate between a light and a dark palette every
`RUMBLE_LENGTH = 3` segments:

```rust
color: if (n / RUMBLE_LENGTH) % 2 == 1 { DARK } else { LIGHT },
```

This alternation is what sells the motion. A flat grey road moving towards you looks exactly
like a flat grey road standing still; the stripes are the only thing your eye can track.
Alternating on every single segment is too fast, though: Jake points out that it turns into a
strobe effect at speed, and groups of three look right. Lane markers are only drawn on light segments (`DARK` has
`lane: None`), which gives the dashed center lines for free.

Two more palettes mark the start and finish: two white segments a little ahead of the player's
starting position, and the last three segments of the track in black.

## Building a track from easing curves

Nobody wants to type 6,705 segments, so the track is assembled from a few building blocks. The
primitive is `add_road`, which adds a stretch of road in three phases: easing into a curve, holding
it, and easing back out, while moving the height from where the road currently is to some
number of segment lengths up or down:

```rust
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
```

The curve and the height ease independently: the curve goes in, holds, and comes out over the
three phases, while the height follows one smooth cosine across the whole stretch. The easing
functions are the classic ones:

\\[
\mathrm{easeIn}(a, b, p) = a + (b - a)\\,p^2
\qquad
\mathrm{easeInOut}(a, b, p) = a + (b - a)\left(\tfrac{1}{2} - \tfrac{1}{2}\cos(\pi p)\right)
\]

Without easing, a curve would snap on at full strength from one segment to the next, which you
feel as a jolt in the steering (the [centrifugal force](driving.md) depends on `curve`), and a
hill would have a visible crease at its foot.

On top of `add_road` sit the named pieces: `add_straight`, `add_curve`, `add_hill`,
`add_s_curves` (five alternating curves with hills mixed in), `add_bumps` (eight short, sharp
height changes), `add_low_rolling_hills`, and `add_downhill_to_end`, which computes however much
height is left and brings the road back down to `y = 0`. That last one matters: the track
loops, and the end has to meet the start at the same height, or the lap ends in a cliff. The
whole lap is then one readable list:

```rust
t.add_straight(LENGTH_SHORT);
t.add_low_rolling_hills(LENGTH_SHORT, HILL_LOW);
t.add_s_curves();
t.add_curve(LENGTH_MEDIUM, CURVE_MEDIUM, HILL_LOW);
t.add_bumps();
// ... a dozen more ...
t.add_downhill_to_end(200);
```

Lengths are 25, 50 or 100 segments per phase, curves are 2 (easy) or 4 (medium), and hills are
20, 40 or 60 segment lengths of rise. Those numbers mean nothing in isolation; they are simply
what Jake tuned until it drove well.

## Scenery

Roadside sprites hang off the segment they stand next to, each with a horizontal `offset` in
road half-widths (so `-1.0` is the left edge of the road, `1.2` a little past the right one).
`add_scenery` places them in passes: billboards every 20 segments at the start, two rows of palm
trees whose spacing slowly grows, columns and trees from segment 250 to 1000, a random plant every
three segments after 200, and clusters of billboards and plants every 100 segments for the rest
of the lap. The randomness only ever touches the scenery (and, later, the traffic); the road itself is
the same every time.
