//! Small self-contained helpers: an RNG (no `rand` dep, works on wasm) and a
//! texture cache keyed by the data file's texture keys.

use std::collections::HashMap;

use crate::data::embedded_texture;
use crate::renderer::{Renderer, TextureHandle};

/// xorshift64* PRNG. Deterministic given a seed; good enough for battle rolls.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1) // avoid the all-zero state
    }

    /// Seed from wall-clock time (works on native and web via macroquad/miniquad).
    pub fn seeded_now() -> Self {
        let t = macroquad::miniquad::date::now();
        Rng::new(t.to_bits() ^ 0xD1B5_4A32_D192_ED03)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Inclusive integer range.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u64;
        lo + (self.next_u64() % span) as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }

    /// Pick a random index in 0..len, or None if empty.
    pub fn index(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            None
        } else {
            Some((self.next_u64() % len as u64) as usize)
        }
    }
}

/// Caches decoded textures so each sheet is uploaded to the GPU only once.
#[derive(Default)]
pub struct TextureCache {
    map: HashMap<String, TextureHandle>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, renderer: &mut Renderer, key: &str) -> TextureHandle {
        if let Some(h) = self.map.get(key) {
            return *h;
        }
        let bytes = embedded_texture(key)
            .unwrap_or_else(|| panic!("no embedded texture registered for key '{key}'"));
        let handle = renderer.load_png(bytes);
        self.map.insert(key.to_string(), handle);
        handle
    }
}
