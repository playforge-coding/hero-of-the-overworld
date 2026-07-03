//! Logical input abstraction so game code never touches raw key codes.
//!
//! State is refreshed once per frame from macroquad via [`Input::poll`].
//! macroquad already reports both the held state and the press *edge* for a key
//! (and latches a press even if the key is released within the same frame, as
//! automation tools do), so the game gets frame-accurate `pressed`/`held`.

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Menu,
}

const ALL: [Button; 7] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::Confirm,
    Button::Cancel,
    Button::Menu,
];

/// Physical keys mapped to each logical button, in [`ALL`] order.
const KEYS: [&[KeyCode]; 7] = [
    &[KeyCode::Up, KeyCode::W],
    &[KeyCode::Down, KeyCode::S],
    &[KeyCode::Left, KeyCode::A],
    &[KeyCode::Right, KeyCode::D],
    &[KeyCode::Enter, KeyCode::Space, KeyCode::Z],
    &[KeyCode::Escape, KeyCode::X, KeyCode::Backspace],
    &[KeyCode::LeftShift, KeyCode::C],
];

#[derive(Default)]
pub struct Input {
    down: [bool; 7],
    pressed: [bool; 7],
}

fn index(b: Button) -> usize {
    match b {
        Button::Up => 0,
        Button::Down => 1,
        Button::Left => 2,
        Button::Right => 3,
        Button::Confirm => 4,
        Button::Cancel => 5,
        Button::Menu => 6,
    }
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Refresh held/pressed state from macroquad. Call once per frame before the
    /// game reads input.
    pub fn poll(&mut self) {
        for (i, keys) in KEYS.iter().enumerate() {
            self.down[i] = keys.iter().any(|&k| is_key_down(k));
            self.pressed[i] = keys.iter().any(|&k| is_key_pressed(k));
        }
    }

    /// Retained for call-site compatibility; macroquad clears edges itself.
    pub fn end_frame(&mut self) {}

    pub fn held(&self, b: Button) -> bool {
        self.down[index(b)]
    }

    /// True on the frame a button went from up to down.
    pub fn pressed(&self, b: Button) -> bool {
        self.pressed[index(b)]
    }

    pub fn any_pressed(&self) -> bool {
        ALL.iter().any(|&b| self.pressed(b))
    }
}
