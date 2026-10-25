# Curves without geometry

This is the trick the whole genre rests on, and it is almost embarrassingly small. The road
never bends. Every segment sits on the world's center line at x = 0, forever. A segment's
`curve` value is only an instruction to the renderer: from here on, shift everything a little
further sideways than the segment before.

## Accumulating the offset

The renderer walks the visible segments from the camera outwards and keeps two running values:
`x`, the sideways offset of the current segment, and `dx`, how much that offset grows per
segment. After each segment, the offset grows by `dx`, and `dx` grows by the segment's curve:

```rust
let mut x = 0.0;
let mut dx = -(track.segments[base].curve * base_percent);
for n in 0..DRAW_DISTANCE {
    let segment = &track.segments[(base + n) % track.len()];
    // ...
    let view = SegmentView {
        p1: project(segment.p1, camera_x - x),
        p2: project(segment.p2, camera_x - x - dx),
        clip: max_y,
    };
    x += dx;
    dx += segment.curve;
    // ...
}
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

Since `dx` grows linearly with distance, `x` grows quadratically; that is, the road center traces
a parabola on screen, and a parabola is a perfectly convincing curve once it's been through the
perspective division (which shrinks the far end of the offset just like it shrinks everything
else). A segment with a curve of 0 keeps `dx` as is: the road continues in whatever direction it
was heading, so a straight that follows a bend still points off to the side until you get there.
That's what lets you see the exit of a turn before you're in it, a property of the segment
approach that Lou's page points out, too.

Note that `x` is subtracted from the *camera's* position before projecting. Moving the camera to
the right and moving the road to the left are the same thing, and doing it on the camera side keeps
the `project` closure identical for every segment.

## The fractional start

The odd-looking initial value, `dx = -(curve * base_percent)`, removes a jump. `base` is the
segment the camera is in, and `base_percent` is how far into it the camera is (`0.0..1.0`). Without
the correction, the accumulated curve would be computed from the start of the camera's segment
every frame, and the road would hop sideways each time the camera crosses into the next segment.
Starting `dx` slightly negative, in proportion to how much of the current segment is already
behind the camera, makes the shift continuous.

## Feeling the curve

A curve that only exists on screen would be a lie your hands notice quickly, so two other systems
read `curve`, too. The [driving system](driving.md) pushes the car towards the outside of the bend
in proportion to speed:

```rust
pos.offset -= dx * speed_percent * segment.curve * CENTRIFUGAL;
```

with `CENTRIFUGAL = 0.3`. At full speed through a medium curve you have to hold the steering in to
stay on the road; slow down, and the push fades quadratically, since `dx` is itself proportional
to speed.

And the background scrolls. Three parallax layers (sky, hills, trees) each keep a scroll offset in
`0..1`, which the `Parallax` system advances by the curve times the distance driven this step,
with each layer at its own speed:

```rust
let scroll = segment.curve * (pos.z - **origin) / SEGMENT_LENGTH;
backdrop.sky = increase(backdrop.sky, SKY_SPEED * scroll, 1.0);
backdrop.hills = increase(backdrop.hills, HILL_SPEED * scroll, 1.0);
backdrop.trees = increase(backdrop.trees, TREE_SPEED * scroll, 1.0);
```

([`src/ecs/systems/parallax.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/parallax.rs))

The speeds are 0.001, 0.002 and 0.003, so the trees move three times as fast as the sky, which is
the cheapest depth cue there is. Combined with the road bending on screen, it reads like the car
is actually turning, even though nothing in the world has rotated by a single degree. How the
layers are drawn (and how they wrap around) is in [the sprites chapter](sprites.md#the-parallax-background).
