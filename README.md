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
  are entities. The track, controls, per-segment traffic grid, player view, background scroll
  and lap times are separate states, so each system declares exactly what it reads and writes.
  Seven systems run in the 60 Hz `FixedUpdate` phase: `Traffic` (AI steering and movement),
  `TrafficIndex` (buckets cars into segments), `Driving` (the player's controls),
  `Collision`, `LapTimer`, `Parallax` and `Snapshot` (publishes the player car).
- `src/track.rs` builds the road and scenery, `src/render.rs` projects and draws it. The road
  trapezoids are filled as scanline rects, since the toolkit has no polygon primitive.

In the browser: `task web` serves a live-rebuilding build at http://localhost:8080, and
`PROFILE=release task web:build` builds into `dist/`. Both need `trunk`
(`cargo install trunk --locked`) and the `wasm32-unknown-unknown` target. The music starts with
the first key press, since browsers keep audio muted until then. Run `task assets:manifest`
after adding files to `assets/`, so the web build preloads them.

## Credits

Game design, graphics and music are from
[javascript-racer](https://github.com/jakesgordon/javascript-racer),
Copyright (c) 2012-2016 Jake Gordon and contributors, MIT licensed. See
[LICENSE-javascript-racer](LICENSE-javascript-racer).
