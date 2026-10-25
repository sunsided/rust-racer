//! The background music: looped, turned down, and mutable.

use game_toolkit::prelude::*;

/// The original plays its music at 5% volume ("shhhh! annoying music!").
const VOLUME: f32 = 0.05;

/// The background music.
pub struct Music {
    /// `None` when there is no audio device.
    handle: Option<SoundHandle>,
}

impl Music {
    /// Loads the music and starts it, looped. On the web, audio only starts with the first key
    /// press or click, so playback is deferred until then.
    pub fn load(ctx: &mut Context) -> Result<Self> {
        let sound = ctx.load_sound("racer.ogg")?;
        let handle = match (&mut ctx.audio, sound) {
            (Some(audio), Some(sound)) => {
                let options = PlayOptions::looped().volume(VOLUME).deferred();
                audio
                    .play_with(sound, options)
                    .inspect_err(|err| log::warn!("cannot play music: {err}"))
                    .ok()
            }
            _ => None,
        };
        Ok(Self { handle })
    }

    pub fn toggle_mute(&mut self) {
        let Some(handle) = &self.handle else {
            return;
        };
        if handle.is_paused() {
            handle.resume();
        } else {
            handle.pause();
        }
    }
}
