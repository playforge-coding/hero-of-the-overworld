//! Small self-contained helpers: an RNG (no `rand` dep, works on wasm), a
//! texture cache keyed by the data file's texture keys, and the scrolling
//! window shared by the menus' lists.

use std::collections::HashMap;

use crate::data::embedded_texture;
use crate::renderer::{Renderer, TextureHandle, color};

/// Index of the first row to show when a list of `count` rows has room for
/// `visible` of them and `cursor` is selected. One row of context is kept above
/// the cursor where the window has room for it, and the window never scrolls
/// past the end.
pub fn list_window_start(cursor: usize, count: usize, visible: usize) -> usize {
    let context = if visible > 1 { 1 } else { 0 };
    cursor
        .saturating_sub(context)
        .min(count.saturating_sub(visible))
}

/// Draw a thin vertical scrollbar at `x`, spanning `y` to `y + h`, for a list
/// of `count` rows showing `visible` of them from row `first`. Draws nothing
/// when everything fits.
pub fn draw_scrollbar(
    r: &mut Renderer,
    (x, y, h): (f32, f32, f32),
    first: usize,
    visible: usize,
    count: usize,
) {
    if count <= visible {
        return;
    }
    r.draw_rect(x, y, 2.0, h, color::rgba(40, 46, 70, 255));
    r.draw_rect(
        x,
        y + h * first as f32 / count as f32,
        2.0,
        h * visible as f32 / count as f32,
        color::rgba(150, 160, 210, 255),
    );
}

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
        self.try_get(renderer, key)
            .unwrap_or_else(|| panic!("no embedded texture registered for key '{key}'"))
    }

    /// Like [`get`](Self::get), but returns `None` instead of panicking when no
    /// texture is registered under `key`. This lets a caller probe for an optional
    /// asset (e.g. a dedicated dialogue portrait) with a plain fallback.
    pub fn try_get(&mut self, renderer: &mut Renderer, key: &str) -> Option<TextureHandle> {
        if let Some(h) = self.map.get(key) {
            return Some(*h);
        }
        let bytes = embedded_texture(key)?;
        let handle = renderer.load_png(&bytes);
        self.map.insert(key.to_string(), handle);
        Some(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_window_keeps_cursor_visible() {
        // A list that fits always starts at the top.
        for cursor in 0..3 {
            assert_eq!(list_window_start(cursor, 3, 3), 0);
        }
        // Five rows in a window of three scroll with one row of context above
        // the cursor, and stop at the end.
        let starts: Vec<_> = (0..5).map(|c| list_window_start(c, 5, 3)).collect();
        assert_eq!(starts, [0, 0, 1, 2, 2]);
        // Every cursor position lands inside its window, for any list shape.
        for count in 0..12 {
            for visible in 1..6 {
                for cursor in 0..count {
                    let first = list_window_start(cursor, count, visible);
                    assert!((first..first + visible).contains(&cursor));
                }
            }
        }
    }
}
