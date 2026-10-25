# rust-racer

An OutRun-style pseudo-3D racer: a Rust port of Jake Gordon's
[javascript-racer](https://github.com/jakesgordon/javascript-racer) (v4, "final") built on
[game-toolkit](https://github.com/sunsided/game-toolkit), with the simulation running in a
[sillyecs](https://github.com/sunsided/sillyecs) world.

```sh
cargo run --release
```

Arrow keys / WASD (or a gamepad) drive, M toggles the music, Esc quits.

## How it's put together

- `ecs.yaml` describes the world; `build.rs` generates it with sillyecs. Cars and the player
  are entities; the track, input, lap times and a per-segment traffic index live in the shared
  `Race` state. Three systems run in the 60 Hz `FixedUpdate` phase: `Traffic` (AI steering and
  movement), `TrafficIndex` (buckets cars into segments) and `Driving` (the player).
- `src/track.rs` builds the road and scenery, `src/render.rs` projects and draws it. The road
  trapezoids are filled as scanline rects, since the toolkit has no polygon primitive.

Native only: the code sillyecs 0.0.7 generates uses `std::time::Instant`, which panics in the
browser.

## Credits

Game design, graphics and music are from
[javascript-racer](https://github.com/jakesgordon/javascript-racer),
Copyright (c) 2012-2016 Jake Gordon and contributors, MIT licensed.
