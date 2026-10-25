# Sprites

Everything that isn't road is a sprite: the palm trees, billboards, boulders and cacti on the
roadside, the 200 cars of traffic, the player's car, and the three background layers. They all
come out of two texture atlases (`sprites.png` and `background.png`), with every image's pixel
rectangle written down as a constant:

```rust
pub const PALM_TREE: AtlasRect = rect(5.0, 5.0, 215.0, 540.0);
pub const BILLBOARD08: AtlasRect = rect(230.0, 5.0, 385.0, 265.0);
// ...
pub const PLAYER_STRAIGHT: AtlasRect = rect(1085.0, 480.0, 80.0, 41.0);
```

([`src/track.rs`](https://github.com/sunsided/rust-racer/blob/main/src/track.rs))

## How big is a palm tree?

Sprites need a size in world units, and the original derives one from a single decision: the
player's car is 0.3 road half-widths wide. Dividing by the car sprite's 80 pixels gives a
world-units-per-pixel factor that applies to every sprite in the atlas:

```rust
/// World units per sprite pixel: the player sprite is 1/3rd of the (half-)road width.
pub const SCALE: f32 = 0.3 * (1.0 / PLAYER_STRAIGHT.w);
```

(Measured in road half-widths, that is; the factor gets multiplied by `ROAD_WIDTH` when drawing.)
That makes the 215 × 540 pixel palm tree about 0.8 half-widths wide and 2 tall, i.e. 1,600 by
4,000 world units, which is a fine palm tree. The same factor turns sprite widths into the
collision widths used by [traffic and collisions](traffic.md), so what you see is what you hit.

On screen, a sprite then gets scaled by the same `scale = d / z` as the road under it:

```rust
let world = scale * width / 2.0 * (sprites::SCALE * ROAD_WIDTH);
let dest_w = sprite.w * world;
let dest_h = sprite.h * world;
```

## Back to front

Sprites are drawn in a second pass over the segments the road pass has already projected, this
time from far to near, which is the plain painter's algorithm: near things are drawn later and
cover far things. For each segment, first its cars, then its roadside sprites, then the player
when the loop reaches the player's segment.

Roadside sprites stand on the segment's near edge, offset sideways by their `offset` in road
half-widths:

```rust
let x = view.p1.x + scale * sprite.offset * ROAD_WIDTH * width / 2.0;
let anchor = [if sprite.offset < 0.0 { -1.0 } else { 0.0 }, -1.0];
```

The anchor is a fraction of the sprite's size added to its position. `-1.0` vertically puts its
bottom edge on the road; horizontally, sprites on the left hang off to the left of their position
and sprites on the right to the right, so a billboard at `offset = -1.0` ends exactly at the left
edge of the road and never sticks into it.

Cars move, so they're placed at their exact position within their segment by interpolating between
the segment's two projected edges with the car's `percent` (how far into the segment it is). They
are anchored at their bottom center.

## Clipping behind hills

Here's where the [`clip` value from the road pass](hills.md#remembering-the-clip-line-for-sprites)
comes in. A sprite whose bottom reaches below the segment's clip line is partly hidden behind the
hill in front of it, so the sprite is cut: both its destination rectangle and the source rectangle
in the atlas lose the same fraction of their height, which keeps the visible part undistorted.

```rust
let clip_h = clip_y.map_or(0.0, |clip| (dest_y + dest_h - clip).max(0.0));
if clip_h >= dest_h {
    return;
}
let src = AtlasRect {
    h: sprite.h - sprite.h * clip_h / dest_h,
    ..sprite
};
let dst = [dest_x, dest_y, dest_w, dest_h - clip_h];
```

This only clips from the bottom, which is all a hill ever does: it hides the base of things behind
it. It's crude next to a depth buffer, and it's also all you need. The player's car is the one
sprite that is never clipped; it's always in front.

## Mirroring traffic

The car sprites in the atlas are drawn at a slight three-quarter angle, showing the rear and the
right flank, which is correct for a car on the left of the camera. A car to the right of the screen
center drawn with the same image looks like it's permanently turning (the blue Beetle is the
worst offender). So cars right of center are mirrored:

```rust
let mirror = x > width / 2.0;
```

and the blit flips the sprite's u coordinates (`SpriteInstance::flip_x`). That one isn't in the
original; it's a deliberate deviation, and it costs two lines.

## The player's car

The player's car is drawn in the same back-to-front pass, when the loop reaches the segment the car
is in, so cars and scenery further ahead are drawn before it, and the few roadside sprites between
the car and the camera (by then mostly at the screen's edges) are drawn after it. It's always horizontally
centered, at the fixed scale `camera_depth / player_z`, and vertically at the road height under it.

Six sprites cover the car's states: steering left, right or straight, each on a level road or
uphill. The choice comes straight from the `Driver` component:

```rust
fn player_sprite(steer: f32, updown: f32) -> AtlasRect {
    let uphill = updown > 0.0;
    match (steer.total_cmp(&0.0), uphill) {
        (Ordering::Less, true) => PLAYER_UPHILL_LEFT,
        (Ordering::Less, false) => PLAYER_LEFT,
        (Ordering::Greater, true) => PLAYER_UPHILL_RIGHT,
        (Ordering::Greater, false) => PLAYER_RIGHT,
        (_, true) => PLAYER_UPHILL_STRAIGHT,
        (_, false) => PLAYER_STRAIGHT,
    }
}
```

And then there's the bounce: every frame, the car is shifted up or down by a random amount of up to
1.5 pixels (scaled with the screen's height relative to the original's 480), proportional to speed.
At a standstill it sits still; at top speed it rattles. It's the cheapest possible suspension
model, and it does more for the feeling of speed than you'd think.

## The parallax background

The three background layers (sky, hills, trees) are each 1,280 pixels wide in the atlas, and the
screen shows half of that at a time, starting at the layer's current scroll offset. When the window
runs past the end of the layer, the remainder is filled with a second blit from the layer's start:

```rust
let image_w = layer.w / 2.0;
let source_x = layer.x + (layer.w * rotation).floor();
let source_w = image_w.min(layer.x + layer.w - source_x);
let dest_w = (width * (source_w / image_w)).floor();
// draw [source_x, source_w] into [0, dest_w], then, if source_w < image_w,
// draw the first (image_w - source_w) pixels into [dest_w - 1, width]
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

The `- 1.0` on the second blit overlaps the two halves by a pixel, so no seam shows where they meet.
The layers scroll horizontally with the curves (see [curves](curves.md#feeling-the-curve)) and
vertically by a small amount proportional to the road height under the car, at the same 1:2:3 ratio.
