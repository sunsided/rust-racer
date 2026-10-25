//! The background music: looped, turned down, and mutable.

use game_toolkit::prelude::*;
use kira::Tween;
use kira::sound::PlaybackState;
use kira::sound::static_sound::StaticSoundHandle;

/// The original plays its music at 5% volume ("shhhh! annoying music!"): -26 dB.
const VOLUME_DB: f32 = -26.0;

/// The background music, restarted whenever it ends.
pub struct Music {
    /// `None` when there is no audio device.
    sound: Option<SoundId>,
    handle: Option<StaticSoundHandle>,
    muted: bool,
}

impl Music {
    pub fn load(ctx: &mut Context) -> Result<Self> {
        Ok(Self {
            sound: ctx.load_sound("racer.ogg")?,
            handle: None,
            muted: false,
        })
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Starts (or restarts) the music, and pauses or resumes it to match the mute state.
    pub fn update(&mut self, ctx: &mut Context) {
        let (Some(audio), Some(sound)) = (&mut ctx.audio, self.sound) else {
            return;
        };
        // On the web, audio only starts with the first key press or click.
        if !audio.is_started() {
            return;
        }
        let playing = self
            .handle
            .as_ref()
            .is_some_and(|h| h.state() != PlaybackState::Stopped);
        if !playing {
            match audio.play(sound) {
                Ok(mut handle) => {
                    handle.set_volume(VOLUME_DB, Tween::default());
                    self.handle = Some(handle);
                }
                Err(err) => {
                    log::warn!("cannot play music: {err}");
                    self.sound = None;
                    return;
                }
            }
        }
        if let Some(handle) = &mut self.handle {
            match (self.muted, handle.state()) {
                (true, PlaybackState::Playing) => handle.pause(Tween::default()),
                (false, PlaybackState::Paused) => handle.resume(Tween::default()),
                _ => {}
            }
        }
    }
}
