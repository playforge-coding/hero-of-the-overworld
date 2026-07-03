//! Logical input abstraction so game code never touches raw key codes.

use winit::keyboard::KeyCode;

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

#[derive(Default)]
pub struct Input {
    down: [bool; 7],
    /// Latched on the up→down transition, held until the next `end_frame`.
    /// This makes edge detection independent of frame timing, so a press and
    /// release delivered within a single frame (as automation tools do) still
    /// registers as one `pressed()` that frame.
    pressed_latch: [bool; 7],
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

    /// Map a physical key to zero or more logical buttons.
    fn buttons_for(key: KeyCode) -> Option<Button> {
        Some(match key {
            KeyCode::ArrowUp | KeyCode::KeyW => Button::Up,
            KeyCode::ArrowDown | KeyCode::KeyS => Button::Down,
            KeyCode::ArrowLeft | KeyCode::KeyA => Button::Left,
            KeyCode::ArrowRight | KeyCode::KeyD => Button::Right,
            KeyCode::Enter | KeyCode::Space | KeyCode::KeyZ => Button::Confirm,
            KeyCode::Escape | KeyCode::KeyX | KeyCode::Backspace => Button::Cancel,
            KeyCode::ShiftLeft | KeyCode::KeyC => Button::Menu,
            _ => return None,
        })
    }

    pub fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if let Some(b) = Self::buttons_for(key) {
            let i = index(b);
            if pressed && !self.down[i] {
                self.pressed_latch[i] = true;
            }
            self.down[i] = pressed;
        }
    }

    /// Call once per update after reading, to clear the per-frame press latches.
    pub fn end_frame(&mut self) {
        self.pressed_latch = [false; 7];
    }

    pub fn held(&self, b: Button) -> bool {
        self.down[index(b)]
    }

    /// True on the frame a button went from up to down (survives a same-frame
    /// release, so single injected key taps are never missed).
    pub fn pressed(&self, b: Button) -> bool {
        self.pressed_latch[index(b)]
    }

    pub fn any_pressed(&self) -> bool {
        ALL.iter().any(|&b| self.pressed(b))
    }
}
