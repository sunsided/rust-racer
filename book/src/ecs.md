# The ECS version

Jake's original keeps its state in a handful of globals and does the player's whole step in one
update function: move the car, apply the controls, check collisions, time the laps, scroll the
background. It's perfectly readable, and the first version of this port kept that shape, with a
`drive()` of about 100 lines. Today, the same logic runs in a
[sillyecs](https://github.com/sunsided/sillyecs) world, split into seven systems. Is an
entity-component system for 201 cars slightly overkill? Yes. It does make every bit of shared state
and every dependency between the steps explicit, and that's the part this chapter is
about. Mostly, though, I wanted a real game to put sillyecs through its paces, and the issues it
turned up say it was worth it.

## The world is a YAML file

sillyecs doesn't use derive macros or runtime registration. You describe the world in
[`ecs.yaml`](https://github.com/sunsided/rust-racer/blob/main/ecs.yaml), and `build.rs` generates
the world, the archetype storage and the system traits from it:

```rust
fn main() {
    println!("cargo:rerun-if-changed=ecs.yaml");
    let file = File::open("ecs.yaml").expect("open ecs.yaml");
    EcsCode::generate(BufReader::new(file))
        .expect("generate ECS")
        .write_files()
        .expect("write ECS files");
}
```

The racer's world has two archetypes:

```yaml
archetypes:
  - name: Car
    components: [TrackPosition, Speed, CarModel]
  - name: Player
    components: [TrackPosition, Speed, Driver, StepOrigin]
```

and six *states*, which are the ECS's shared resources: singletons that systems read or write, as
opposed to components, which exist per entity.

| State | Holds | Written by |
|---|---|---|
| `Track` | road and scenery, read-only during a race | nobody |
| `Controls` | the keys held this frame | the game loop |
| `TrafficGrid` | AI cars per segment | `TrafficIndex` |
| `PlayerView` | the player car as of the last step | `Snapshot` |
| `Backdrop` | parallax scroll offsets | `Parallax` |
| `Laps` | current, last and fastest lap time | `LapTimer` |

Every state has exactly one writer. That wasn't the first version: initially, all shared data lived
in a single `Race` state that two of the three systems wrote, which meant the scheduler saw one big
resource and the systems' real dependencies were invisible. Splitting it up made the YAML say what
the code does.

## Systems, and in which order

Systems declare which states they use, which components they read (`inputs`) and which they write
(`outputs`), and which systems they have to run after:

```yaml
- name: Collision
  description: Stops the player at roadside scenery and behind slower cars.
  phase: FixedUpdate
  run_after: [Driving]
  states:
    - use: Track
    - use: TrafficGrid
  inputs:
    - StepOrigin
  outputs:
    - TrackPosition
    - Speed
```

Seven systems run in the 60 Hz `FixedUpdate` phase, in this order:

| # | System | Runs on | Does |
|---|---|---|---|
| 1 | `Traffic` | `Car` | steers the AI cars around slower traffic and the player, moves them |
| 2 | `TrafficIndex` | `Car` | rebuilds the `TrafficGrid` |
| 3 | `Driving` | `Player` | controls, centrifugal force, off-road drag, limits |
| 4 | `Collision` | `Player` | scenery and car collisions |
| 5 | `LapTimer` | `Player` | lap timing |
| 5 | `Parallax` | `Player` | background scroll |
| 5 | `Snapshot` | `Player` | publishes the player car into `PlayerView` |

The last three only depend on `Collision`, read the same components and write different states, so
they're independent of each other.

`Snapshot` is the interesting one. The AI needs to know where the player is, and the renderer needs
the player's position, speed and steering. Rather than have them query the `Player` archetype
directly, `Snapshot` copies what they need into the `PlayerView` state at the end of each step. The
AI in the *next* step and the renderer in this frame both read that copy, which keeps the data flow
one-directional and makes the AI's one-step delay explicit instead of an accident of system order.

## Implementing a system

For each system, sillyecs generates a trait with one method per granularity. Per entity, you get
`apply_single` with exactly the states and components the YAML declared, mutable where it declared
writes:

```rust
impl ApplyCollisionSystem for CollisionSystem {
    type Error = Infallible;

    fn apply_single(
        &mut self,
        track: &TrackState,
        grid: &TrafficGridState,
        origin: &StepOriginComponent,
        pos: &mut TrackPositionComponent,
        speed: &mut SpeedComponent,
    ) {
        // ...
    }
}
```

([`src/ecs/systems/collision.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs/systems/collision.rs))

For whole archetypes, `apply_many` gets the component columns as slices, which is what `Traffic` and
`TrafficIndex` use to zip over 200 cars at once. And `on_begin_phase` runs once before any entity
is touched, which is where `TrafficIndex` clears the grid.

The systems here have no data of their own, and writing a struct plus a `CreateSystem` impl for
each of the seven got old quickly, so a small macro does it:

```rust
macro_rules! stateless_systems {
    ($($system:ident($data:ident)),* $(,)?) => {
        $(
            #[derive(Debug, Default)]
            pub struct $data;

            impl CreateSystem<$system> for SystemFactory {
                fn create(&self) -> $system {
                    $system::from($data)
                }
            }
        )*
    };
}
```

([`src/ecs/systems.rs`](https://github.com/sunsided/rust-racer/blob/58132f367092f8f31fee0d00bd8de73f0e6aa834/src/ecs/systems.rs))

**Update:** The macro is gone. sillyecs 0.2 handles stateless systems itself
([#64](https://github.com/sunsided/sillyecs/issues/64)): a system marked `stateless: true` in the
YAML gets its empty data type and a `CreateSystem` impl that works with any factory, so with all
seven marked, the factory is `&()`:

```rust
let mut world = RacerWorld::new(&(), states, ChannelQueue::new());
```

## The step origin

Splitting the original's single update into five player systems had one catch. The original looks
up the player's segment once, at the start of the update, and then does everything (collisions,
lap timing, background scroll) against that segment, even though the car has already moved on by
then. Once those are separate systems, `Collision` only sees where the car is *after* `Driving`
moved it, which can be the next segment over, with different scenery and different cars.

So the `Player` archetype has a `StepOrigin` component: `Driving` writes the car's `z` into it before
moving, and every later system evaluates the step against the segment at that origin:

```rust
/// The segment the player's car was in when the step began. The JS original evaluates the
/// whole step against it, so all player systems do too.
fn origin_segment(track: &TrackState, origin: f32) -> &Segment {
    &track.segments[track.find(origin + player_z())]
}
```

`Parallax` also needs it for a second reason: it scrolls the background by the distance actually
driven, `pos.z - origin`, and that has to be measured *after* a collision has snapped the car back.
In the single update function, that was implicit in the order of statements; in the ECS it's a
component, which is more typing and much harder to break by accident.

There is one small difference to the original that came out of the split: the speed and offset
limits now apply in `Driving`, before the collision checks, rather than after them. In the original,
the speed tested for a hit can be up to one step's acceleration above top speed; here it's clamped
first.

## Fencing in the generated code

The generated code is pulled in with `include!` from `OUT_DIR`, and generated code tends to trip
lints that hand-written code shouldn't (its `unsafe` fast paths, which `allow_unsafe: true` in the
YAML asks for, call `get_unchecked` without `unsafe {}` blocks, a few constants go unused, and some
doc links point nowhere). All four `include!`s live in their own module,
[`src/ecs/generated.rs`](https://github.com/sunsided/rust-racer/blob/58132f367092f8f31fee0d00bd8de73f0e6aa834/src/ecs/generated.rs),
with the lint exemptions on that module only:

```rust
#![allow(unsafe_op_in_unsafe_fn, dead_code)]
#![allow(rustdoc::broken_intra_doc_links, rustdoc::redundant_explicit_links)]

use super::*;

include!(concat!(env!("OUT_DIR"), "/components_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/archetypes_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/systems_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/world_gen.rs"));
```

Keeping them off the parent `ecs` module means `dead_code` and `unsafe_op_in_unsafe_fn` still
apply to the hand-written systems, which is exactly where you want those lints.
[#65](https://github.com/sunsided/sillyecs/issues/65) asks for the generated code to carry its own
allows (and need fewer of them).

**Update:** With sillyecs 0.2, the fence came down as well. The generated items carry their own
allows now (#65 again), `paths:` in the YAML tells the generated code where the racer's types live,
and `sillyecs::include_ecs!` declares the module
([#68](https://github.com/sunsided/sillyecs/issues/68)), so `generated.rs` is deleted and
[`src/ecs.rs`](https://github.com/sunsided/rust-racer/blob/main/src/ecs.rs) has:

```rust
sillyecs::include_ecs!(
    /// The code sillyecs generates from `ecs.yaml` (see `build.rs`).
    mod generated
);
```

The hand-written systems still get `dead_code` and `unsafe_op_in_unsafe_fn`, which was the point of
the fence in the first place.

The port also turned up three issues in sillyecs 0.0.7, all fixed in 0.1.0: unknown keys in the
YAML were silently ignored ([#60](https://github.com/sunsided/sillyecs/issues/60)), phase timing
used `std::time::Instant`, which doesn't exist in the browser
([#61](https://github.com/sunsided/sillyecs/issues/61); more on that in the
[next chapter](rust-wgpu-web.md#the-browser-build)), and the generated code tripped
`unreachable_patterns` warnings and needed undocumented direct dependencies on `rayon` and `tracing`
([#62](https://github.com/sunsided/sillyecs/issues/62)).
