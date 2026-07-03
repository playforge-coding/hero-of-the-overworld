//! Background music playback.
//!
//! Native builds use `rodio` (cpal + Vorbis). The web build plays the same
//! embedded OGG through an `<audio>` element fed by an object URL. Both go
//! through the identical [`Audio`] API, and every failure path degrades to
//! silence rather than crashing the game.

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::io::Cursor;

    use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

    /// Owns the audio device and the single looping music track.
    pub struct Audio {
        // Dropping the stream stops all sound, so it's kept alive for the whole
        // program. `None` if no output device was available (e.g. headless).
        _stream: Option<OutputStream>,
        handle: Option<OutputStreamHandle>,
        music: Option<Sink>,
    }

    impl Audio {
        pub fn new() -> Self {
            match OutputStream::try_default() {
                Ok((stream, handle)) => Audio {
                    _stream: Some(stream),
                    handle: Some(handle),
                    music: None,
                },
                Err(e) => {
                    log::warn!("audio disabled (no output device): {e}");
                    Audio {
                        _stream: None,
                        handle: None,
                        music: None,
                    }
                }
            }
        }

        /// Start looping `bytes` (a Vorbis/OGG track) as the music, replacing
        /// anything currently playing. Failures are logged, never fatal.
        pub fn play_music_looping(&mut self, bytes: &'static [u8]) {
            self.stop_music();
            let Some(handle) = &self.handle else { return };
            let sink = match Sink::try_new(handle) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("could not create music sink: {e}");
                    return;
                }
            };
            match Decoder::new(Cursor::new(bytes)) {
                Ok(decoder) => sink.append(decoder.repeat_infinite()),
                Err(e) => {
                    log::warn!("could not decode music: {e}");
                    return;
                }
            }
            self.music = Some(sink);
        }

        /// Stop the music if any is playing.
        pub fn stop_music(&mut self) {
            if let Some(sink) = self.music.take() {
                sink.stop();
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

    /// Web audio: a looping `<audio>` element plus the object URL backing it
    /// (kept so it can be revoked on stop).
    pub struct Audio {
        current: Option<(HtmlAudioElement, String)>,
    }

    impl Audio {
        pub fn new() -> Self {
            Audio { current: None }
        }

        /// Loop `bytes` (an OGG track) via a fresh `<audio>` element. Autoplay is
        /// permitted because battles only start after keyboard input (a user
        /// gesture). Any failure is logged and leaves the game silent.
        pub fn play_music_looping(&mut self, bytes: &'static [u8]) {
            self.stop_music();
            match make_audio(bytes) {
                Ok((el, url)) => {
                    el.set_loop(true);
                    // play() returns a Promise; a rejection (e.g. autoplay block)
                    // just means no music — nothing to await.
                    let _ = el.play();
                    self.current = Some((el, url));
                }
                Err(e) => log::warn!("web audio failed: {e:?}"),
            }
        }

        pub fn stop_music(&mut self) {
            if let Some((el, url)) = self.current.take() {
                let _ = el.pause();
                let _ = Url::revoke_object_url(&url);
            }
        }
    }

    /// Build an `<audio>` element from raw OGG bytes via a Blob object URL.
    fn make_audio(bytes: &[u8]) -> Result<(HtmlAudioElement, String), JsValue> {
        let array = js_sys::Uint8Array::from(bytes);
        let parts = js_sys::Array::of1(&array);
        let options = BlobPropertyBag::new();
        options.set_type("audio/ogg");
        let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
        let url = Url::create_object_url_with_blob(&blob)?;
        let el = HtmlAudioElement::new_with_src(&url)?;
        Ok((el, url))
    }
}

pub use backend::Audio;
