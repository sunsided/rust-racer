# Projection: one division per point

Getting a point of the road onto the screen takes three steps: move it into camera space (subtract
the camera position), project it onto a projection plane (divide by its distance), and scale the
result from the plane's `-1..1` to pixels. There is no rotation step, since the road never
turns in world space; and there is no matrix, since one division is all the perspective there is.

## Similar triangles

Side view: the camera sits at the origin, the projection plane stands at distance \\(d\\) in
front of it, and a point on the road lies \\(z\\) ahead and \\(y\\) below the camera. The ray from
the camera to the point crosses the plane at height \\(y'\\), and the two triangles it forms are
similar:

<svg viewBox="0 0 520 210" width="100%" style="max-width: 520px" role="img"
     aria-label="Side view of the camera, the projection plane at distance d, and a road point at distance z below the camera">
  <g fill="none" stroke="currentColor" stroke-width="1.5">
    <line x1="30" y1="170" x2="500" y2="170" stroke-dasharray="4 4" opacity="0.6"/>
    <line x1="140" y1="20" x2="140" y2="190" opacity="0.8"/>
    <line x1="30" y1="40" x2="460" y2="40" stroke-dasharray="2 4" opacity="0.6"/>
    <line x1="30" y1="40" x2="460" y2="170"/>
    <line x1="460" y1="40" x2="460" y2="170" stroke-dasharray="2 4" opacity="0.6"/>
  </g>
  <g fill="currentColor">
    <circle cx="30" cy="40" r="5"/>
    <circle cx="460" cy="170" r="5"/>
    <circle cx="140" cy="73" r="4"/>
  </g>
  <g fill="currentColor" font-family="sans-serif" font-size="14">
    <text x="18" y="28">camera</text>
    <text x="146" y="16">projection plane</text>
    <text x="80" y="34">d</text>
    <text x="290" y="34">z</text>
    <text x="146" y="62">y′</text>
    <text x="468" y="110">y</text>
    <text x="400" y="195">road point</text>
    <text x="34" y="190">road</text>
  </g>
</svg>

\\[
\frac{y'}{d} = \frac{y}{z}
\quad\Longrightarrow\quad
y' = y \cdot \frac{d}{z}
\\]

The same holds for \\(x\\), so everything a point needs is one factor, `scale = d / z`, which is
also exactly how much bigger or smaller anything at that distance gets drawn. Sprites reuse it
later.

The plane's distance comes from the field of view. With the plane spanning \\(-1..1\\), half the
field of view has to fit into one unit of height:

\\[
d = \frac{1}{\tan(\mathrm{fov}/2)} = \frac{1}{\tan 50^\circ} \approx 0.839
\\]

```rust
pub const FIELD_OF_VIEW: f32 = 100.0;
pub const CAMERA_HEIGHT: f32 = 1000.0;

pub fn camera_depth() -> f32 {
    1.0 / (FIELD_OF_VIEW / 2.0).to_radians().tan()
}
```

([`src/race.rs`](https://github.com/sunsided/rust-racer/blob/main/src/race.rs))

## The project function

Here's the projection as it appears in the renderer, as a closure over the camera position for
the current segment:

```rust
let camera_x = player.x * ROAD_WIDTH;
let camera_y = player_y + CAMERA_HEIGHT;
let project = |world: [f32; 2], camera_x: f32| {
    let camera_y = world[0] - camera_y;
    let camera_z = world[1] - camera_z;
    let scale = camera_depth / camera_z;
    Projected {
        x: (width / 2.0 - scale * camera_x * width / 2.0).round(),
        y: (height / 2.0 - scale * camera_y * height / 2.0).round(),
        w: (scale * ROAD_WIDTH * width / 2.0).round(),
        scale,
        camera_y,
        camera_z,
    }
};
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

A few things are worth pointing out:

- The screen y axis points down, the world's points up; hence the `height / 2.0 - ...`.
- `w` is the projected *half*-width of the road at that point, which is all you need to draw it:
  the road spans `x - w..x + w` on screen.
- The `x` passed in is the camera's lateral position, and since the road's center line is always
  at world x = 0, the camera's offset is the only thing that moves the road sideways. Steering
  the car moves the camera; the road doesn't care. (The curve offset gets subtracted here as well,
  but that's for the [next chapter](curves.md).)
- The camera follows the terrain: `player_y` is the road height under the player's car,
  interpolated within its segment, and the camera floats `CAMERA_HEIGHT` above it.
- The results are rounded to whole pixels, as in the original. Since a segment's far edge and the
  next segment's near edge are projected with identical inputs, they still meet exactly.
- Horizontal and vertical scale use `width / 2` and `height / 2` separately. That means the image
  stretches with the window's aspect ratio instead of showing more of the world, which is also
  what the original does, and on the usual 4:3 to 16:9 range it looks fine.

## Where the car sits

The player's car is not where the camera is. It's drawn a fixed distance ahead of it:

```rust
pub fn player_z() -> f32 {
    CAMERA_HEIGHT * camera_depth()
}
```

That's about 839 units, a bit over four segments, and the number isn't arbitrary. Plug a road
point at \\(z = H\\,d\\) (that's `player_z`) and \\(y = -H\\) (flat road,
camera `CAMERA_HEIGHT` \\(= H\\) above it) into the projection, with \\(h\\) the screen height in pixels:

\\[
y_s
= \frac{h}{2} - \frac{d}{H\\,d}\\,(-H)\\,\frac{h}{2}
= \frac{h}{2} + \frac{h}{2}
= h
\\]

The road under the car lands exactly on the bottom edge of the screen, for any field of view and
camera height. This is why the game keeps track of the *camera's* `z` as "the player's
position" (`PlayerViewState::position`), and why almost every system adds `player_z()` to it
before asking which segment the car is in.
