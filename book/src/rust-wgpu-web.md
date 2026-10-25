# game-toolkit, wgpu and the browser

JavaScript gets a lot for free here: a canvas with polygon fills and image blits, a
`requestAnimationFrame` loop, keyboard events, and an `<audio>` tag. In Rust, all of that comes from
[game-toolkit](https://github.com/sunsided/game-toolkit), a 2D-first game-jam toolkit I wrote that
bundles winit for windows and input, wgpu for rendering, kira for audio, and gilrs for gamepads behind
one `use game_toolkit::prelude::*;`. This chapter is about the places where that difference shows.

## The game loop

A game-toolkit game is a type implementing `Game`, and the toolkit owns the loop:

```rust
impl Game for Racer {
    fn init(ctx: &mut Context) -> Result<Self> { /* load textures, build the track, spawn cars */ }
    fn update(&mut self, ctx: &mut Context, _dt: f32) { /* input, music, run the ECS */ }
    fn event(&mut self, _ctx: &mut Context, e: &GameEvent) { /* reset the clock on focus */ }
    fn render(&mut self, ctx: &mut Context, frame: &mut Frame) { /* draw the scene and the HUD */ }
}

fn main() -> Result<()> {
    run::<Racer>(AppConfig {
        title: "Rust Racer".into(),
        width: 1024,
        height: 768,
        asset_root: asset_root!(),
        canvas_id: Some("game".into()),
        ..Default::default()
    })
}
```

([`src/main.rs`](https://github.com/sunsided/rust-racer/blob/main/src/main.rs))

Notice that `update` ignores the frame's `dt`; the simulation steps itself at a fixed rate (see
[Driving](driving.md#fixed-step-real-time)). `render` only reads the world's states, never writes them,
which is what lets rendering happen at whatever rate the display likes while the simulation sticks to
60 Hz.

## Drawing with a Painter

`frame.painter(&mut ctx.gfx)` gives an immediate-mode `Painter` for the frame, and everything queued on it
is batched and flushed to wgpu when it drops. The racer needs four of its calls:

- `clear` for the sky color,
- `quad` for the road's trapezoids (four arbitrary corners, one color, a layer),
- `sprite_ex` for everything textured, with a `SpriteInstance` built from a destination rectangle,
  an atlas region (`with_pixel_region`) and optional mirroring (`flip_x`),
- `rect_ex` and `text` for the HUD.

The canvas had one rule for draw order (last one wins); a batching GPU renderer has more, and they're
what the [layers](drawing.md#why-call-order-still-matters) are for. The other difference is texture
coordinates: the original's `drawImage` takes pixel rectangles, while a GPU sprite wants normalized
UVs. The atlas rectangles stay in pixels in the code (they're copied from Jake's sprite sheet layout
as is), and `with_pixel_region` leaves the division to the painter, which knows the atlas's size
when `sprite_ex` draws the sprite:

```rust
let inst = SpriteInstance::at([dst[0], dst[1]], [dst[2], dst[3]])
    .with_pixel_region([src.x, src.y], [src.w, src.h]);
let inst = if mirror { inst.flip_x() } else { inst };
p.sprite_ex(tex, inst, layer, BlendMode::Alpha);
```

([`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs))

Before game-toolkit 0.4 had atlas helpers, the atlas sizes were hard-coded constants in the racer,
which works right up until somebody re-exports the sprite sheet. 0.4's `with_region` still wanted
the size passed in, looked up with `Painter::texture_size`; `with_pixel_region` arrived in 0.5
([#41](https://github.com/sunsided/game-toolkit/issues/41)).

The scene is redrawn from scratch each frame: no retained geometry, no vertex buffers that
survive the frame on the game's side. For a game made of a few thousand quads that's the simplest
thing that works, and the batching keeps it cheap.

## Input

Input is polled, not evented. `read_input` turns the held keys (arrows or WASD) and the first gamepad
(left stick past ±0.3, D-pad, south button or right trigger for gas, west button or left trigger to brake)
into a plain `ControlsState` of four booleans, which the game loop writes into the world before running
the systems. The systems never see a key code.

## Music

The music is Jake's, played through kira, and it comes with one line of folklore. The original sets its
volume to 5 %, next to the comment "shhhh! annoying music!", and the port does the same:

```rust
/// The original plays its music at 5% volume ("shhhh! annoying music!").
const VOLUME: f32 = 0.05;
```

([`src/music.rs`](https://github.com/sunsided/rust-racer/blob/main/src/music.rs))

`Music::load` starts the track once, with `PlayOptions::looped().volume(VOLUME).deferred()`, and keeps
the `SoundHandle` to pause and resume it on M. It copes with there being no audio device at all, in
which case `ctx.audio` is `None` and the game is simply silent.

On game-toolkit 0.4, this took more work: `play` returned kira's own handle, so the racer depended on
kira directly, set the volume in decibels (5 % amplitude is \\(20 \log_{10} 0.05 \approx -26\\) dB),
and looped the music by checking every frame whether it had stopped and playing it again. 0.5's
`SoundHandle` and `PlayOptions` ([#38](https://github.com/sunsided/game-toolkit/issues/38)) made all of
that one call.

## The browser build

The same crate builds for `wasm32-unknown-unknown` with [trunk](https://trunkrs.dev/), and the published
version is what runs in the [introduction](introduction.md). Getting there took a handful of small
things:

- [`index.html`](https://github.com/sunsided/rust-racer/blob/main/index.html) holds a
  `<canvas id="game">` and two trunk directives: build the crate, and copy the `assets` folder next to
  it. `canvas_id: Some("game".into())` tells the toolkit to render into that canvas instead of creating
  a window. `public_url = "./"` in `Trunk.toml` makes all URLs relative, so the same build works from
  the root of a domain, from `/rust-racer/play/` on GitHub Pages, and from an itch.io iframe.
- In the browser, files arrive over HTTP, asynchronously, and `Game::init` wants them
  synchronously. The toolkit therefore preloads every file listed in
  [`assets/manifest.txt`](https://github.com/sunsided/rust-racer/blob/main/assets/manifest.txt) before
  calling `init`; `task assets:manifest` regenerates the list. `asset_root!()` resolves to the crate's
  `assets/` folder natively and to `assets` next to `index.html` on the web.
- `std::time::Instant` panics on `wasm32-unknown-unknown`, and sillyecs measures its phases
  with it. Its `wasm` feature switches to `performance.now()`, enabled only for that target:
  ```toml
  [target.'cfg(target_arch = "wasm32")'.dependencies]
  sillyecs = { version = "0.2.1", features = ["wasm"] }
  ```
- Browsers keep audio suspended until the first user gesture. The first version took the
  resulting error from `play` for a missing audio device and gave up on the music for good; the
  fix was to wait for `Audio::is_started()` before trying. Since game-toolkit 0.5, the error is
  its own `AudioError::NotStarted`, and a `deferred` play is queued until the gesture
  ([#39](https://github.com/sunsided/game-toolkit/issues/39)), which is what the racer uses now.
- `env_logger` is a native-only dependency; on the web, the toolkit logs to the browser
  console.

## Debug builds

One more thing, unrelated to the browser: an unoptimized build decodes the Ogg music and pushes the
frame's quads slowly enough to be no fun at all. Optimizing only the *dependencies* in debug builds
fixes that while keeping the game's own code quick to compile and easy to debug:

```toml
[profile.dev.package."*"]
opt-level = 3
```

## Deploying

[GitHub Pages](https://github.com/sunsided/rust-racer/blob/main/.github/workflows/pages.yml) builds this
book as the site's root, builds the game into `play/` with a pinned, checksum-verified trunk release,
and deploys both from `main`. The build is deliberately uncached, so a deployed site never restores a cache
written by some other branch.
