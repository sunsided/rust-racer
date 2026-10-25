# Introduction

Rust Racer is a port of Jake Gordon's [javascript-racer](https://github.com/jakesgordon/javascript-racer)
(the v4 "final" version from 2012) to Rust, running on my own
[game-toolkit](https://github.com/sunsided/game-toolkit) with the simulation living in a
[sillyecs](https://github.com/sunsided/sillyecs) world. It is an OutRun-style racer: a road that
curves and rolls over hills towards the horizon, palm trees and billboards flying past, 200 cars
of traffic to weave through, and a lap timer that judges you. It also contains no 3D geometry
whatsoever, which is the fun part, and the reason this book exists.

Here it is, running right in this page (click into it first; browsers keep the audio muted
until the game has seen a key press or a click):

<!-- The game renders at a fixed 1024×768, so the frame keeps that size and is scaled to fit. -->
<div class="racer-embed" style="position: relative; width: 100%; aspect-ratio: 4 / 3; overflow: hidden; margin: 1em 0; background: #111;">
  <iframe title="Rust Racer" loading="lazy" scrolling="no"
          allow="autoplay; gamepad; fullscreen"
          style="position: absolute; top: 0; left: 0; width: 1024px; height: 768px; border: 0; transform-origin: 0 0;"></iframe>
</div>
<script>
  // Published, the game lives in play/ next to the book. A local `mdbook serve` doesn't have it,
  // so fall back to the `task web` dev server there.
  const embeds = document.querySelectorAll(".racer-embed");
  const devServer = `http://${location.hostname}:8080/`;
  fetch("play/index.html")
    .then((r) => (r.ok ? r.text() : ""))
    .then((html) => (html.includes('id="game"') ? "play/index.html" : devServer))
    .catch(() => devServer)
    .then((src) => embeds.forEach((embed) => (embed.querySelector("iframe").src = src)));
  for (const embed of embeds) {
    const frame = embed.querySelector("iframe");
    new ResizeObserver(() => {
      frame.style.transform = `scale(${embed.clientWidth / 1024})`;
    }).observe(embed);
  }
</script>

Arrow keys or WASD (or a gamepad) drive, M toggles the music. [Play it full screen](play/) (or with the ▶ button
at the top of every page) if the frame is too small. (Reading this from a local `task book`? The frame then shows the `task web`
dev server, so start that, too, or run `cargo run --release`.)

![Driving through a curve past billboards, with traffic ahead](images/screenshot.png)

## What this book is

This is the long version of how it works: the pseudo-3D trick itself,
which mechanics make it feel like a racing game, the small hacks that keep it from falling apart,
and what changes when you do it in Rust on top of wgpu instead of in JavaScript on a 2D canvas.
It roughly follows the order in which you would build such a thing yourself (straight road
first, then curves, then hills, then everything that lives on and next to the road), and it
ends with a [playbook](playbook.md) that condenses all of it into a checklist.

The chapters quote the actual code, trimmed to the interesting lines, and link to the full
source on GitHub. Constants and formulas are the real ones; where I give a number like "about
6,700 segments", it's computed from the code rather than guessed. The game design, graphics and
music are Jake's (MIT licensed, see
[LICENSE-javascript-racer](https://github.com/sunsided/rust-racer/blob/main/LICENSE-javascript-racer)),
and most of the clever bits described here are his, too, or older still: the
[history chapter](history.md) traces them back to the arcades of the early 1980s.

## Where the code lives

| File | What's in it |
|---|---|
| [`src/track.rs`](https://github.com/sunsided/rust-racer/blob/main/src/track.rs) | Road geometry, the sprite atlas, the track builder, easing helpers. |
| [`src/render.rs`](https://github.com/sunsided/rust-racer/blob/main/src/render.rs) | Projection, the road, the sprites, the parallax background. |
| [`src/race.rs`](https://github.com/sunsided/rust-racer/blob/main/src/race.rs) | Driving, camera and traffic constants. |
| [`ecs.yaml`](https://github.com/sunsided/rust-racer/blob/main/ecs.yaml) | The ECS world: states, components, archetypes, systems. |
| [`src/ecs/`](https://github.com/sunsided/rust-racer/tree/main/src/ecs) | One module per component, state and system. |
| [`src/main.rs`](https://github.com/sunsided/rust-racer/blob/main/src/main.rs) | The game loop, input and the HUD. |

All in all it's about 1,700 lines of Rust, which says more about how little a pseudo-3D racer
needs than about Rust.
