//! Background music playback via `macroquad::audio` (one code path on native and
//! web). The single looping track is decoded once up front with
//! [`Audio::load_music`]; playback then degrades to silence rather than crashing
//! if decoding failed.

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

/// Owns the decoded music track and its play state.
#[derive(Default)]
pub struct Audio {
    music: Option<Sound>,
    playing: bool,
}

impl Audio {
    pub fn new() -> Self {
        Audio::default()
    }

    /// Decode the looping music track. Async (macroquad decodes off the byte
    /// slice); call once during startup. Failure is logged, never fatal.
    pub async fn load_music(&mut self, bytes: &[u8]) {
        match load_sound_from_bytes(bytes).await {
            Ok(s) => self.music = Some(s),
            Err(e) => log::warn!("could not load music: {e:?}"),
        }
    }

    /// Start the music looping, restarting it if already playing. No-op if the
    /// track failed to load. The `_bytes` argument is kept for call-site
    /// compatibility; the single preloaded track is used.
    pub fn play_music_looping(&mut self, _bytes: &'static [u8]) {
        if let Some(s) = &self.music {
            stop_sound(s);
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: 0.6,
                },
            );
            self.playing = true;
        }
    }

    /// Stop the music if any is playing.
    pub fn stop_music(&mut self) {
        if let Some(s) = &self.music {
            stop_sound(s);
        }
        self.playing = false;
    }
}
