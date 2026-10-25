# Hills, and what they hide

Unlike curves, hills are real geometry: each segment's edges have an actual height `y`, built by
the easing functions from [the segments chapter](segments.md), and the projection divides it by
distance like any other coordinate. A segment higher than the camera lands higher on screen, a
segment in a valley lower; nothing else is needed to make the road roll.

What hills do add is occlusion. On a flat road, every segment further away is drawn higher up on
the screen than the one before it, and nothing ever hides anything. Crest a hill, and the road
beyond it drops out of sight behind the road in front of you. Without a depth buffer, the renderer
has to work that out on its own.

## Front to back, with a horizon that moves

The answer is to draw the road from the camera outwards and keep track of the highest point on
screen (the smallest screen y) that the road has reached so far, `max_y`. It starts at the bottom
of the screen. Each segment is then checked against three conditions before it's drawn:

```rust
let behind_us = view.p1.camera_z <= camera_depth;
let back_face = view.p2.y >= view.p1.y;
let hidden_by_hill = view.p2.y >= max_y;
if behind_us || back_face || hidden_by_hill {
    continue;
}

let fog = exponential_fog(n as f32 / DRAW_DISTANCE as f32, FOG_DENSITY);
draw_segment(p, width, &view, fog, &segment.color);
max_y = view.p1.y;
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

- `behind_us`: the near edge is closer than the projection plane. Projecting it would divide by
  something tiny or negative and throw the trapezoid across the whole screen.
- `back_face`: the far edge is drawn *below* the near edge, which happens on the far side of a
  crest, where you'd be looking at the road's underside if it had one. Skip.
- `hidden_by_hill`: the far edge doesn't rise above what's already been drawn, so the whole
  segment is behind the road in front of it.

Note that the loop `continue`s rather than `break`s. A segment hidden behind a crest can be
followed by one that's visible again further away, when the road climbs another, higher hill in
the distance; you see the far hillside while the valley between stays hidden.

The order is the opposite of what you'd expect from the painter's algorithm (which paints far
things first and lets near things cover them), and that's deliberate: drawing front to back with
`max_y` means hidden segments are skipped instead of drawn and painted over. It's also why the
road is drawn in *one* pass: once a segment is drawn, nothing nearer ever comes along to cover it.
(The check uses the previous segment's near edge, so a far segment can occasionally overlap a
sliver of the one in front. The original does the same, and keeping the draw order identical
keeps it looking identical.)

## Remembering the clip line for sprites

The one thing the front-to-back pass can't handle is sprites. A palm tree standing on a segment
just behind a crest should show its top while its trunk is hidden by the hill in front of it. But
sprites are drawn after the road, on a higher layer, in a separate back-to-front pass, so the road
can't occlude them by paint order.

So the renderer remembers, for every visible segment, what `max_y` was at the moment that segment
was projected:

```rust
struct SegmentView {
    p1: Projected,
    p2: Projected,
    /// Screen y above which the hills in front have already covered the screen.
    clip: f32,
}
```

and the sprite pass cuts off whatever part of a sprite reaches below its segment's `clip`; the
details are in [the sprites chapter](sprites.md#clipping-behind-hills). The `views` buffer holding
these lives in the `Renderer` and is reused every frame, which keeps the render loop free of
allocations.

## Wrapping around the end of the track

Drawing 300 segments ahead (`DRAW_DISTANCE`) means that close to the finish line, the segments in
view are partly at the end of the track and partly at its start again. Their stored `z` values are
small again, though, so they'd project as if they were behind the camera. The fix is to pretend the
camera is one track length further back for those:

```rust
let looped = segment.index < base;
let camera_z = player.position - if looped { track.length() } else { 0.0 };
```

Since the track's last segment ends at height 0 and its first starts there, the seam is invisible.

## The camera follows the terrain

The last ingredient is the camera height. If the camera stayed at a fixed `y`, the car would sink
into every hill and float above every valley. Instead, the renderer looks up the segment under the
car (not under the camera; the car is `player_z` ahead), interpolates the road height at the car's
position within it, and puts the camera `CAMERA_HEIGHT` above that:

```rust
let player_segment = track.find(player.position + player_z);
let player_percent = percent_remaining(player.position + player_z, SEGMENT_LENGTH);
let ps = &track.segments[player_segment];
let player_y = interpolate(ps.p1[0], ps.p2[0], player_percent);
// ...
let camera_y = player_y + CAMERA_HEIGHT;
```

The same `player_y` also nudges the background layers up and down a little, so the horizon rises
and falls with the road.
