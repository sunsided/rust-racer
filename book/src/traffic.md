# Traffic and collisions

A racing game with an empty road is a driving simulator, so there are 200 other cars on the track.
They're scattered over the whole lap at the start, each with a random sprite, a random lane
position within ±0.8 half-widths, and a random speed between a quarter and three quarters of top
speed (semis top out at half):

```rust
let offset = rng.next_f32() * 0.8 * random_side(rng);
let z = rng.range_usize(0..segments) as f32 * SEGMENT_LENGTH;
let sprite = sprites::CARS[rng.range_usize(0..sprites::CARS.len())];
let spread = if sprite == sprites::SEMI { 4.0 } else { 2.0 };
let speed = MAX_SPEED / 4.0 + rng.next_f32() * MAX_SPEED / spread;
```

([`src/ecs/world.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/world.rs))

Their speed never changes. AI cars don't brake, don't accelerate, and don't race you; they drive at
their speed until the end of time, and the only decision they ever make is to swerve.

## A grid of segments

Both the AI and the collision checks need to know which cars are near a given spot, and the track
already comes chopped into handy buckets. The `TrafficGrid` state holds one list of cars per
segment, and the `TrafficIndex` system clears and refills it every step:

```rust
fn on_begin_phase(&mut self, _track: &TrackState, grid: &mut TrafficGridState) -> Result<(), Self::Error> {
    grid.clear();
    Ok(())
}

fn apply_many(&mut self, track: &TrackState, grid: &mut TrafficGridState, /* ... */) {
    for ((pos, speed), model) in cars {
        grid.insert(track.find(pos.z), TrafficCar { /* z, offset, percent, speed, sprite */ });
    }
}
```

([`src/ecs/systems/traffic_index.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/traffic_index.rs))

The per-segment vectors are cleared, not dropped, so after the first few steps the grid doesn't
allocate anymore. The renderer reads the same grid to find the cars to draw on each segment, which
is why it's a state of its own rather than something private to the AI.

## Swerving

Each step, every car looks up to 20 segments ahead for something slower in its way: the player's
car, or another car from the grid. "In its way" means the two overlap sideways, with their widths
padded by 20 %. The first obstacle found decides the swerve, and the swerve is stronger the closer
the obstacle and the bigger the speed difference:

```rust
for i in 1..LOOKAHEAD {
    let segment = (car_segment + i) % segments;
    if segment == self.player_segment
        && speed > player.speed
        && overlap(player.x, PLAYER_W, offset, car_w, 1.2)
    {
        let dir = swerve_direction(offset, player.x);
        return dir / i as f32 * (speed - player.speed) / MAX_SPEED;
    }
    for other in self.grid.cars(segment) {
        if speed > other.speed && overlap(offset, car_w, other.offset, other.width(), 1.2) {
            let dir = swerve_direction(offset, other.offset);
            return dir / i as f32 * (speed - other.speed) / MAX_SPEED;
        }
    }
}
```

([`src/ecs/systems/traffic.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/traffic.rs))

The direction is simple: if the obstacle is near one edge of the road, pass it on the inside;
otherwise, pass on whichever side you're already on. And if there's nothing ahead but the car has
somehow ended up beyond ±0.9, it drifts back towards the middle by 0.1 per step.

Two shortcuts keep this cheap. Cars more than `DRAW_DISTANCE` segments ahead of the player don't
bother steering at all, since nobody would see it. And the grid the AI reads is the one built in
the *previous* step, since `Traffic` runs before `TrafficIndex`; the same goes for the player's
position, which `Snapshot` publishes at the end of each step. Every car decides based on where
everybody was 1/60 s ago, which is close enough that you'd never notice, and the AI never has to
see a half-updated world.

What the AI doesn't do is collide with other AI cars. Two cars that fail to swerve in time simply
drive through each other; the original works the same way, and the port keeps it.

## Hitting things

The player's collisions are handled by the `Collision` system, right after `Driving`. Both kinds of
collision boil down to the same one-dimensional overlap test, since everything else is already
known to be in the same segment:

```rust
pub fn overlap(x1: f32, w1: f32, x2: f32, w2: f32, percent: f32) -> bool {
    let half = percent / 2.0;
    let (min1, max1) = (x1 - w1 * half, x1 + w1 * half);
    let (min2, max2) = (x2 - w2 * half, x2 + w2 * half);
    !(max1 < min2 || min1 > max2)
}
```

Scenery only matters when you're off the road (`|offset| > 1`), since nothing stands on it.
Hitting a palm tree drops you to a fifth of top speed and puts you back at the start of the segment
the tree stands on, in front of it:

```rust
if overlap(pos.offset, PLAYER_W, center, sprite_w, 1.0) {
    **speed = MAX_SPEED / 5.0;
    pos.z = increase(segment.p1[1], -player_z, track.length());
    break;
}
```

The `-player_z` is there because `pos.z` is the camera's position, and the camera trails the car
by `player_z`.

Other cars only matter when you're faster than them (a slower car can't be rear-ended by you, it's
the other way around, and they'll swerve). Hitting one puts you right behind it, at a speed that
drops the harder you hit:

```rust
if **speed > car.speed && overlap(pos.offset, PLAYER_W, car.offset, car.width(), 0.8) {
    **speed = car.speed * (car.speed / **speed);
    pos.z = increase(car.z, -player_z, track.length());
    break;
}
```

That's \\(v' = v_\text{car}^2 / v\\): bump a car going nearly your speed and you barely notice;
plow into a slow truck at full speed and you end up well below the truck's speed. The 0.8 makes
the hitboxes a bit narrower than the sprites, so a pass that looks like a close call stays one.

All of this runs against the segment the car was in when the step *began*, not the one it ended up
in; the [ECS chapter](ecs.md#the-step-origin) explains why that needed its own component.
