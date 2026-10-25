# Driving

The driving model fits on an index card, and nearly every constant in it is derived from one
other constant. That's not laziness; it's the most useful design decision in the whole game.

## Top speed is one segment per step

The simulation runs at a fixed 60 steps per second, and the top speed is chosen so that a car
at full speed moves exactly one segment per step:

```rust
pub const STEP: f32 = 1.0 / 60.0;
/// Top speed: never move more than one segment per step, which keeps collisions simple.
pub const MAX_SPEED: f32 = SEGMENT_LENGTH / STEP;
pub const ACCEL: f32 = MAX_SPEED / 5.0;
pub const BREAKING: f32 = -MAX_SPEED;
pub const DECEL: f32 = -MAX_SPEED / 5.0;
pub const OFF_ROAD_DECEL: f32 = -MAX_SPEED / 2.0;
pub const OFF_ROAD_LIMIT: f32 = MAX_SPEED / 4.0;
pub const CENTRIFUGAL: f32 = 0.3;
```

([`src/race.rs`](https://github.com/sunsided/rust-racer/blob/main/src/race.rs))

That's 12,000 world units per second. Because nothing can ever skip a segment, collision
detection never has to look further than the segment you started the step in: anything you could
have hit this step is in there. No swept volumes, no tunneling, no continuous collision detection;
the speed limit *is* the collision detection's precondition.

The rest follows as fractions of top speed, which makes the feel easy to reason about:

| | Constant | Effect |
|---|---|---|
| Accelerating | `MAX_SPEED / 5` | 0 to top speed in 5 seconds |
| Braking | `-MAX_SPEED` | top speed to standstill in 1 second |
| Coasting | `-MAX_SPEED / 5` | rolls to a stop in 5 seconds |
| Off-road | `-MAX_SPEED / 2`, on top | down to a quarter of top speed, quickly |

The HUD shows `5 * round(speed / 500)` mph, so top speed reads as a nice round 120 mph.

## One step of the player's car

The whole `Driving` system, minus the bookkeeping:

```rust
let dt = STEP;
let segment = origin_segment(track, pos.z);
let speed_percent = **speed / MAX_SPEED;
// At top speed, crossing the road from left to right takes one second.
let dx = dt * 2.0 * speed_percent;

pos.z = increase(pos.z, dt * **speed, track.length());

if controls.left {
    pos.offset -= dx;
} else if controls.right {
    pos.offset += dx;
}
pos.offset -= dx * speed_percent * segment.curve * CENTRIFUGAL;

**speed += dt * if controls.faster { ACCEL } else if controls.slower { BREAKING } else { DECEL };
let off_road = pos.offset < -1.0 || pos.offset > 1.0;
if off_road && **speed > OFF_ROAD_LIMIT {
    **speed += dt * OFF_ROAD_DECEL;
}

pos.offset = pos.offset.clamp(-3.0, 3.0);
**speed = speed.clamp(0.0, MAX_SPEED);
```

([`src/ecs/systems/driving.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/driving.rs))

A few things to notice:

- The car's lateral position, `offset`, is in road half-widths: `-1..1` is on the road, and the
  clamp lets you wander up to three half-widths into the grass but no further.
- Steering scales with speed. Standing still, you can't move sideways at all; at top speed,
  crossing the whole road takes a second. This is also what keeps a stationary car from sliding
  out of a curve: the centrifugal term has `dx` in it, and `dx` is zero when you're stopped.
- The off-road penalty only kicks in above a quarter of top speed. Below that, the grass is just
  slow, not a wall: you can always drive back onto the road.
- The car moves before the controls are applied, using the speed from the previous step. With a
  fixed step of 1/60 s the difference is tiny, and it's the order the original uses.

The system also records what the sprite selection needs: `steer` is -1, 0 or 1 (and forced to 0 when standing still, since a parked car doesn't lean into corners), and `updown` is the
height difference across the current segment, positive when going uphill.

## Fixed step, real time

The game loop doesn't call the systems with the frame's delta time. Instead, the sillyecs world
has a `FixedUpdate` phase at 60 Hz, and each frame runs it as many times as the elapsed real time
allows:

```rust
fn update(&mut self, ctx: &mut Context, _dt: f32) {
    // ...
    self.world.states.controls = read_input(&ctx.input);
    // Runs the FixedUpdate phase as many times as the elapsed real time allows.
    self.world.apply_system_phases();
}
```

([`src/main.rs`](https://github.com/sunsided/rust-racer/blob/main/src/main.rs))

That decouples the simulation from the frame rate: on a 144 Hz display you get mostly frames with
zero or one step, on a struggling laptop two or three steps per frame, and either way the car
covers the same distance per second and collides the same way. It's also what keeps the
one-segment-per-step guarantee intact; with a variable `dt`, a slow frame would let the car jump
several segments at once. (Jake's original uses the same approach, crediting Gaffer On Games'
[Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/).)

One gotcha: when the window loses focus (or the browser tab is in the background), real time keeps
passing, and on return the world would try to catch up on all of it at once. The game resets the
frame clock when it gets focus back:

```rust
fn event(&mut self, _ctx: &mut Context, e: &GameEvent) {
    // Don't fast-forward through the time spent in the background.
    if let GameEvent::FocusChanged(true) = e {
        self.world.reset_frame_context();
    }
}
```

## Laps

The start line is where the camera crosses `z = player_z`, which puts the car just past the two
white segments when the lap ticks over. The `LapTimer` system counts up while the camera is past
that point, and when the step started before it and ended after it, the lap is done:

```rust
if pos.z <= player_z {
    return;
}
let crossed_start = **origin < player_z;
if laps.current > 0.0 && crossed_start {
    laps.last_was_fastest = laps.current <= laps.fastest;
    if laps.last_was_fastest {
        laps.fastest = laps.current;
    }
    laps.last = Some(laps.current);
    laps.current = 0.0;
} else {
    laps.current += STEP;
}
```

([`src/ecs/systems/lap_timer.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/lap_timer.rs))

Since the timer only starts once the camera has passed the line, the clock doesn't run while you
sit on the grid. The time to beat starts at 3 minutes; beat it, and the lap times in the HUD turn
gold.
