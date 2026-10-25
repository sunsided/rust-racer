//! An OutRun-style pseudo-3D racer: a Rust port of Jake Gordon's
//! [javascript-racer](https://github.com/jakesgordon/javascript-racer) (v4, "final") onto
//! game-toolkit, with the simulation running in a sillyecs world.
//!
//! Arrow keys / WASD (or a gamepad) drive, M toggles the music, Esc quits.

mod ecs;
mod music;
mod race;
mod render;
mod track;

use game_toolkit::prelude::*;

use ecs::{CommandQueue, MainWorldStates, RacerWorld};
use ecs::{ControlsState, LapsState};
use music::Music;
use race::player_z;
use render::{Renderer, Textures};
use track::Track;

struct Racer {
    world: RacerWorld,
    renderer: Renderer,
    textures: Textures,
    music: Music,
}

impl Game for Racer {
    fn init(ctx: &mut Context) -> Result<Self> {
        let textures = Textures {
            background: ctx.load_texture("background.png")?,
            sprites: ctx.load_texture("sprites.png")?,
        };

        let track = Track::build(&mut ctx.rng, player_z());
        let states = ecs::race_states(track);
        let mut world = RacerWorld::new(&(), states, CommandQueue::default());
        ecs::spawn_racers(&mut world, &mut ctx.rng);

        let music = Music::load(ctx)?;

        Ok(Self {
            world,
            renderer: Renderer::new(),
            textures,
            music,
        })
    }

    fn update(&mut self, ctx: &mut Context, _dt: f32) {
        if ctx.input.key_pressed(Key::Escape) {
            ctx.quit();
        }
        if ctx.input.key_pressed(Key::KeyM) {
            self.music.toggle_mute();
        }
        self.music.update(ctx);

        self.world.states.controls = read_input(&ctx.input);
        // Runs the FixedUpdate phase as many times as the elapsed real time allows.
        self.world.apply_system_phases();
    }

    fn event(&mut self, _ctx: &mut Context, e: &GameEvent) {
        // Don't fast-forward through the time spent in the background.
        if let GameEvent::FocusChanged(true) = e {
            self.world.reset_frame_context();
        }
    }

    fn render(&mut self, ctx: &mut Context, frame: &mut Frame) {
        let (w, h) = ctx.gfx.size();
        let size = [w as f32, h as f32];
        let states = &self.world.states;
        let mut p = frame.painter(&mut ctx.gfx);
        self.renderer
            .draw(&mut p, &self.textures, states, size, &mut ctx.rng);
        draw_hud(&mut p, states, size[0]);
    }
}

fn read_input(input: &Input) -> ControlsState {
    let held = |keys: [Key; 2]| keys.iter().any(|&k| input.key_held(k));
    let mut controls = ControlsState {
        left: held([Key::ArrowLeft, Key::KeyA]),
        right: held([Key::ArrowRight, Key::KeyD]),
        faster: held([Key::ArrowUp, Key::KeyW]),
        slower: held([Key::ArrowDown, Key::KeyS]),
    };
    if let Some(pad) = input.first_gamepad() {
        let stick = pad.axis(Axis::LeftStickX);
        controls.left |= stick < -0.3 || pad.button_held(Button::DPadLeft);
        controls.right |= stick > 0.3 || pad.button_held(Button::DPadRight);
        controls.faster |= pad.button_held(Button::South) || pad.button_held(Button::RightTrigger2);
        controls.slower |= pad.button_held(Button::West) || pad.button_held(Button::LeftTrigger2);
    }
    controls
}

fn draw_hud(p: &mut Painter<'_>, states: &MainWorldStates, width: f32) {
    const SIZE: f32 = 20.0;
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const GOLD: [f32; 4] = [1.0, 0.85, 0.2, 1.0];

    // Above the scene's background, road and sprite layers.
    let bar_color = [0.0, 0.0, 0.0, 0.55];
    p.rect_ex(
        [0.0, 0.0],
        [width, 36.0],
        bar_color,
        render::HUD_LAYER,
        BlendMode::Alpha,
    );

    let speed = 5.0 * (states.player_view.speed / 500.0).round();
    let LapsState {
        current,
        last,
        fastest,
        last_was_fastest,
    } = states.laps;
    let lap_color = if last_was_fastest { GOLD } else { WHITE };

    let column = width / 4.0;
    p.text([12.0, 8.0], &format!("Speed: {speed} mph"), SIZE, WHITE);
    let current = format_time(current);
    p.text(
        [12.0 + column, 8.0],
        &format!("Time: {current}"),
        SIZE,
        WHITE,
    );
    if let Some(last) = last {
        let last = format_time(last);
        p.text(
            [12.0 + 2.0 * column, 8.0],
            &format!("Last Lap: {last}"),
            SIZE,
            lap_color,
        );
    }
    let fastest = format_time(fastest);
    let text = format!("Fastest Lap: {fastest}");
    p.text([12.0 + 3.0 * column, 8.0], &text, SIZE, lap_color);
}

/// `s.t` below a minute, `m.ss.t` above.
fn format_time(secs: f32) -> String {
    let minutes = (secs / 60.0).floor();
    let seconds = (secs - minutes * 60.0).floor();
    let tenths = (10.0 * secs.fract()).floor();
    if minutes > 0.0 {
        format!("{minutes}.{seconds:02}.{tenths}")
    } else {
        format!("{seconds}.{tenths}")
    }
}

fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    run::<Racer>(AppConfig {
        title: "Rust Racer".into(),
        width: 1024,
        height: 768,
        // The crate's assets/ natively; on the web, "assets" next to index.html.
        asset_root: asset_root!(),
        // On the web, render into index.html's <canvas id="game">.
        canvas_id: Some("game".into()),
        ..Default::default()
    })
}
