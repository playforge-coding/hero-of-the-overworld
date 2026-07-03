//! Background music playback via `macroquad::audio` (one code path on native and
//! web). Each looping track (the normal battle theme, the boss theme, …) is
//! decoded once up front with [`Audio::load_music`] and later selected for
//! playback by its [`Track`] id. Playback degrades to silence rather than
//! crashing if a track failed to decode.

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

/// A named background-music track. Callers select what to play by id rather than
/// by the byte slice it was decoded from: a `&'static [u8]` const has no stable
/// address across codegen units, so pointer identity can't key the lookup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    /// The ordinary battle theme.
    Battle,
    /// The boss theme, swapped in for boss encounters.
    Boss,
}

/// Owns the decoded music tracks, keyed by their [`Track`] id.
#[derive(Default)]
pub struct Audio {
    tracks: Vec<(Track, Sound)>,
    playing: bool,
}

impl Audio {
    pub fn new() -> Self {
        Audio::default()
    }

    /// Decode and register a looping track under its [`Track`] id, so a later
    /// [`Audio::play_music_looping`] with the same id selects it. Async
    /// (macroquad decodes off the byte slice); call once per track during
    /// startup. Failure is logged, never fatal.
    pub async fn load_music(&mut self, track: Track, bytes: &[u8]) {
        match load_sound_from_bytes(bytes).await {
            Ok(s) => self.tracks.push((track, s)),
            Err(e) => log::warn!("could not load {track:?} music: {e:?}"),
        }
    }

    /// Start `track` looping, stopping whatever was playing first. No-op if that
    /// track failed to load (or was never registered).
    pub fn play_music_looping(&mut self, track: Track) {
        self.stop_music();
        if let Some((_, s)) = self.tracks.iter().find(|(t, _)| *t == track) {
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

    /// Stop any music that is playing.
    pub fn stop_music(&mut self) {
        for (_, s) in &self.tracks {
            stop_sound(s);
        }
        self.playing = false;
    }
}
