# Painting a segment

Once a segment has passed the visibility checks, it has two projected edges, each with a screen
`x` (center), `y`, and half-width `w`. Everything that gets drawn for it is a trapezoid between
those two horizontal spans:

```rust
/// A filled trapezoid between a horizontal span at `near` and one at `far`
/// (`[x_center, half_width, y]` each).
fn trapezoid(p: &mut Painter<'_>, near: [f32; 3], far: [f32; 3], color: [f32; 4]) {
    let [x1, w1, y1] = near;
    let [x2, w2, y2] = far;
    let corners = [[x1 - w1, y1], [x1 + w1, y1], [x2 + w2, y2], [x2 - w2, y2]];
    p.quad(corners, color, ROAD_LAYER);
}
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

A segment is up to six of those, stacked on top of each other, with later ones covering earlier ones:

<svg viewBox="0 0 520 150" width="100%" style="max-width: 520px" role="img"
     aria-label="One road segment: grass band, rumble strip trapezoid, road trapezoid, two lane markers">
  <polygon points="0,140 520,140 520,20 0,20" fill="#10AA10"/>
  <polygon points="20,140 500,140 330,20 190,20" fill="#555555"/>
  <polygon points="60,140 460,140 320,20 200,20" fill="#6B6B6B"/>
  <polygon points="190,140 198,140 241,20 239,20" fill="#CCCCCC"/>
  <polygon points="322,140 330,140 281,20 279,20" fill="#CCCCCC"/>
  <g font-family="sans-serif" font-size="13" fill="#fff">
    <text x="440" y="40">grass</text>
    <text x="24" y="132">rumble</text>
    <text x="226" y="120">road</text>
  </g>
</svg>

1. a full-width band of grass from the left to the right edge of the screen,
2. the rumble strip, a trapezoid a bit wider than the road (`w / 6` on each side),
3. the road itself,
4. the lane markers (for three lanes, two thin trapezoids `w / 32` wide), only on light segments,
5. and the fog, which is fully transparent only on the very first segment.

```rust
fn draw_segment(p: &mut Painter<'_>, width: f32, view: &SegmentView, fog: f32, color: &Palette) {
    let (near, far) = (&view.p1, &view.p2);
    let half = width / 2.0;
    let (band_near, band_far) = ([half, half, near.y], [half, half, far.y]);
    trapezoid(p, band_near, band_far, color.grass);

    let rumble = |w: f32| w / (2.0 * LANES as f32).max(6.0);
    let edge = |pt: &Projected| [pt.x, pt.w + rumble(pt.w), pt.y];
    trapezoid(p, edge(near), edge(far), color.rumble);
    let road = |pt: &Projected| [pt.x, pt.w, pt.y];
    trapezoid(p, road(near), road(far), color.road);
    // ... lane markers ...

    if fog < 1.0 {
        let [r, g, b, _] = FOG_COLOR;
        trapezoid(p, band_near, band_far, [r, g, b, 1.0 - fog]);
    }
}
```

Drawing the rumble strip as one wide trapezoid under the road, instead of two thin ones at its
sides, saves a quad per segment, and the road covers the middle anyway.

## Fog

Fog is a semi-transparent band in the fog color (a dark green, `#005108`) over the whole width of
the segment, with an opacity that grows exponentially with distance:

\\[
\mathrm{fog}(n) = e^{-\left(n / N\right)^2 \cdot \rho}
\\]

where \\(n\\) is the segment's index in the draw loop, \\(N = 300\\) the draw distance, and
\\(\rho = 5\\) the fog density. At the far end, \\(e^{-5} \approx 0.007\\) of the segment's own
color is left. The fog does double duty: it looks like haze, and it hides the fact that the world
simply ends 300 segments ahead of you (60,000 units, five seconds at top speed).

## From scanlines to quads

The road didn't always look this tidy. Jake's original draws each segment as canvas polygons, which
game-toolkit didn't have when I started porting; the first version of this renderer filled the
road line by line with one-pixel-high rectangles, four or five per screen row. That's thousands of
sprite instances per frame (twice as many on a HiDPI display), and the edges of the road came out
stair-stepped. It worked, and it looked like it.

Adding a `Painter::quad` to game-toolkit fixed both (the quad, the atlas helpers the sprites now
use, and layer support for plain rectangles were filed while porting as
[#26](https://github.com/sunsided/game-toolkit/issues/26),
[#27](https://github.com/sunsided/game-toolkit/issues/27) and
[#28](https://github.com/sunsided/game-toolkit/issues/28), and landed in 0.4), and the road is now a handful of quads per segment.

## Why call order still matters

There's a subtlety here, since a GPU renderer isn't a canvas. game-toolkit's `Painter` batches
its draws: content is sorted by layer, and within a layer, sprites and sprite-based shapes are
grouped by blend mode and texture. On a canvas, the last thing drawn is on top, always. In a
batching renderer, that only holds within a batch.

The racer uses four layers:

```rust
const BACKGROUND_LAYER: i16 = 0;
const ROAD_LAYER: i16 = 1;
const SPRITE_LAYER: i16 = 2;
pub const HUD_LAYER: i16 = 3;
```

All road quads go onto one layer, are untextured, and share one blend mode, which puts them in a
single group, and filled shapes keep their call order within their group. So a segment's road
covers its rumble strip, its fog covers its road, and the occasional far segment that overlaps a
nearer one paints over it exactly like on Jake's canvas. If the road used two blend modes (say,
additive fog), the batching would reorder them and the fog would end up underneath the road.

The HUD has a smaller version of the same problem: untextured rectangles use the toolkit's
internal white texture, which sorts before every loaded texture, so a semi-transparent HUD bar
drawn with a plain `rect` on the sprite layer would end up *under* the sprites. Putting it on its
own `HUD_LAYER` with `rect_ex` keeps it on top. (Text isn't layered at all and always draws last,
which is exactly what a HUD wants.)
