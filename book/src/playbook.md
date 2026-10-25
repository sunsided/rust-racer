# The playbook

If you want to build one of these yourself, in Rust or anything else, this is the order I'd go in.
Each step produces something you can look at, and none of them needs the next one to make sense. It
is, give or take, the order of Jake's four articles, with the parts the port added slotted in.

## 1. A straight road

1. Pick a segment length (200) and a road half-width (2,000), and fill a `Vec` with a few hundred
   segments whose near and far edges sit at `z = n * len` and `(n + 1) * len`, all at `y = 0`.
2. Write the projection: camera-space coordinates, `scale = d / z` with `d = 1 / tan(fov / 2)`,
   then pixels. Round the results.
3. Draw each segment as a grass band, a road trapezoid and a rumble trapezoid underneath it,
   alternating light and dark palettes every three segments.
4. Move the camera forward in `z`, wrap it around the track length, and draw from the camera's
   segment outwards for a few hundred segments, adding the track length to `z` for segments that
   wrapped around.
5. Put the camera at `CAMERA_HEIGHT * d` behind the point the car stands on, so the road under the
   car lands on the bottom edge of the screen.

You now have the Pole Position effect: stripes rushing towards you on a flat road. Add lane markers
on the light segments, and fog.

## 2. Speed and steering

6. Fix the timestep (60 Hz) and decouple it from rendering; set the top speed to one segment per
   step and derive acceleration, braking, coasting and off-road drag from it.
7. Give the player a lateral offset in road half-widths, steer by an amount proportional to speed,
   clamp it, and move the camera with it.

## 3. Curves

8. Give each segment a `curve`. While drawing, accumulate `x += dx; dx += curve`, subtract `x` from
   the camera's x for each segment's near edge and `x + dx` for its far edge, and start `dx` at
   `-(curve * percent_into_current_segment)`.
9. Build the track from `enter / hold / leave` pieces with eased curve values.
10. Push the car outwards by `dx * speed_percent * curve * centrifugal`.
11. Add the parallax background layers and scroll them by `curve * distance`, each at its own speed.

## 4. Hills

12. Give segments eased heights, interpolate the road height under the car, and put the camera
    `CAMERA_HEIGHT` above it.
13. Track `max_y` while drawing front to back; skip segments behind the camera, back-facing ones,
    and ones whose far edge doesn't rise above `max_y`. Remember `max_y` per segment for later.
14. Make the end of the track return to height 0.

## 5. Things on the road

15. Sprite atlas, one world-units-per-pixel factor derived from the player car's width, roadside
    sprites hanging off segments with an offset and an anchor.
16. A second, back-to-front pass for sprites, clipping each one at its segment's remembered `max_y`.
17. The player's car at the screen center, with sprite variants for steering and hills, and a
    speed-dependent bounce.
18. Traffic: cars with a position, lateral offset and fixed speed; a per-segment grid rebuilt every
    step; a lookahead that swerves around slower things.
19. Collisions against the segment the car started the step in: scenery when off-road, slower cars
    always, both as a 1D overlap test.
20. Lap timing at the start line, and a HUD.

## Pitfalls

Things worth getting right from the start:

- `z` wraps. Everything that moves along the track goes through one helper that wraps `z` into
  `0..length`, and everything that looks *ahead* (the draw loop, the AI's lookahead) wraps segment
  indices with `%`. Mix up camera `z` and car `z` (they differ by `player_z`) and every system is off
  by four segments.
- Variable timesteps break the collision model. As soon as one step can move more than one
  segment, cars tunnel through each other.
- The draw order is the occlusion model. Road front to back with `max_y`, sprites back to front
  with clipping, and both passes must stay in their own layers. With a batching renderer, make sure
  everything in the road layer is batched together, or call order stops meaning anything.
- Evaluate a step against where it started. Collisions, lap timing and parallax all look at the
  segment the car was in at the start of the step; once those are separate functions or systems,
  that origin has to be stored somewhere.
- Smooth everything that crosses a segment boundary. The `-(curve * percent)` start for `dx`,
  interpolated camera height, interpolated car positions: each one removes a visible hop at segment
  boundaries.
- Mind the browser. Audio needs a user gesture, `Instant` panics in wasm, files have to be
  preloaded, and a background tab must not fast-forward the simulation when it comes back.
