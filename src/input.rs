//! Logical input abstraction so game code never touches raw key codes — or raw
//! gamepad buttons.
//!
//! [`Controllers`] is polled once per frame ([`Controllers::poll`]) and folds the
//! keyboard together with every connected gamepad into logical [`Input`]
//! snapshots:
//!
//!   - [`Controllers::shared`] — the keyboard OR'd with *all* gamepads. This is
//!     what drives every single-player screen (title, world map, overworld,
//!     menus, dialogue), so any controller — or the keyboard — works everywhere.
//!   - [`Controllers::player`] — the input for one *party member*. Gamepad 0 (plus
//!     the keyboard) drives member 0, gamepad 1 drives member 1, and so on. When
//!     fewer gamepads are plugged in than there are members, the extra members
//!     fall back to the shared input, so a single player still commands the whole
//!     party exactly as before.
//!
//! macroquad reports both the held state and the press *edge* for a key (and
//! latches a press even if the key is released within the same frame, as
//! automation tools do), so keyboard `pressed` stays frame-accurate. Gamepad
//! press edges are derived here by diffing against the previous frame.
//!
//! Gamepad support uses `gilrs`, which is native-only; on the web build the
//! gamepad code compiles out and only the keyboard remains.

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

#[cfg(not(target_arch = "wasm32"))]
use gilrs::{Axis, Button as PadButton, GamepadId, Gilrs};

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

const N: usize = 7;

const ALL: [Button; N] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::Confirm,
    Button::Cancel,
    Button::Menu,
];

/// Physical keys mapped to each logical button, in [`ALL`] order.
const KEYS: [&[KeyCode]; N] = [
    &[KeyCode::Up, KeyCode::W],
    &[KeyCode::Down, KeyCode::S],
    &[KeyCode::Left, KeyCode::A],
    &[KeyCode::Right, KeyCode::D],
    &[KeyCode::Enter, KeyCode::Space, KeyCode::Z],
    &[KeyCode::Escape, KeyCode::X, KeyCode::Backspace],
    &[KeyCode::LeftShift, KeyCode::C],
];

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

/// A single frame's logical button state for one input source (or the merge of
/// several). Produced by [`Controllers`]; game code only ever reads it.
#[derive(Default, Clone)]
pub struct Input {
    down: [bool; N],
    pressed: [bool; N],
}

impl Input {
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

    /// OR another source's held/pressed state into this one.
    fn merge(&mut self, other: &Input) {
        for i in 0..N {
            self.down[i] |= other.down[i];
            self.pressed[i] |= other.pressed[i];
        }
    }
}

/// Read the current keyboard state into a logical [`Input`], using macroquad's
/// own press-edge latching for `pressed`.
fn read_keyboard() -> Input {
    let mut inp = Input::default();
    for (i, keys) in KEYS.iter().enumerate() {
        inp.down[i] = keys.iter().any(|&k| is_key_down(k));
        inp.pressed[i] = keys.iter().any(|&k| is_key_pressed(k));
    }
    inp
}

/// Left-stick deflection past this fraction counts as a d-pad press.
#[cfg(not(target_arch = "wasm32"))]
const STICK_DEADZONE: f32 = 0.5;

/// Read one gamepad's held button state (d-pad and left stick both drive the
/// directions; South/East are Confirm/Cancel; Start/Select are Menu).
#[cfg(not(target_arch = "wasm32"))]
fn read_pad(gp: &gilrs::Gamepad) -> [bool; N] {
    let x = gp.value(Axis::LeftStickX);
    let y = gp.value(Axis::LeftStickY);
    [
        gp.is_pressed(PadButton::DPadUp) || y > STICK_DEADZONE,
        gp.is_pressed(PadButton::DPadDown) || y < -STICK_DEADZONE,
        gp.is_pressed(PadButton::DPadLeft) || x < -STICK_DEADZONE,
        gp.is_pressed(PadButton::DPadRight) || x > STICK_DEADZONE,
        gp.is_pressed(PadButton::South),
        gp.is_pressed(PadButton::East),
        gp.is_pressed(PadButton::Start) || gp.is_pressed(PadButton::Select),
    ]
}

/// Owns every input device and produces per-frame logical snapshots. Created
/// once and [`poll`](Controllers::poll)ed at the top of each frame.
pub struct Controllers {
    /// Keyboard + all gamepads merged; drives all single-player UI.
    shared: Input,
    /// Per-party-member input. Slot `i` is gamepad `i` (slot 0 also merges the
    /// keyboard). Always at least one entry.
    players: Vec<Input>,
    /// How many gamepads are currently connected.
    pad_count: usize,
    #[cfg(not(target_arch = "wasm32"))]
    gilrs: Option<Gilrs>,
    /// Previous-frame held state per connected pad, keyed by id, for press edges.
    #[cfg(not(target_arch = "wasm32"))]
    prev_pad_down: Vec<(GamepadId, [bool; N])>,
}

impl Default for Controllers {
    fn default() -> Self {
        Self::new()
    }
}

impl Controllers {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                // No gamepad backend (headless CI, missing udev, …): keyboard
                // still works, so this is a warning, not a failure.
                log::warn!("gamepad support unavailable: {e}");
                None
            }
        };
        Controllers {
            shared: Input::default(),
            players: vec![Input::default()],
            pad_count: 0,
            #[cfg(not(target_arch = "wasm32"))]
            gilrs,
            #[cfg(not(target_arch = "wasm32"))]
            prev_pad_down: Vec::new(),
        }
    }

    /// Refresh keyboard and gamepad state. Call once per frame before the game
    /// reads input.
    pub fn poll(&mut self) {
        let keyboard = read_keyboard();

        // Collect each connected gamepad's (held, press-edge) state, in a stable
        // order so pad → party-member assignment doesn't shuffle between frames.
        // Only the gamepad path (native) pushes to this; on web it stays empty.
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut pads: Vec<Input> = Vec::new();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(gilrs) = &mut self.gilrs {
            // Draining the event queue is what refreshes gilrs' cached state that
            // `is_pressed`/`value` read from.
            while gilrs.next_event().is_some() {}

            let mut next_prev = Vec::new();
            for (id, gp) in gilrs.gamepads() {
                let down = read_pad(&gp);
                let prev = self
                    .prev_pad_down
                    .iter()
                    .find(|(pid, _)| *pid == id)
                    .map(|(_, d)| *d)
                    .unwrap_or([false; N]);
                let mut inp = Input::default();
                for i in 0..N {
                    inp.down[i] = down[i];
                    inp.pressed[i] = down[i] && !prev[i];
                }
                pads.push(inp);
                next_prev.push((id, down));
            }
            self.prev_pad_down = next_prev;
        }

        self.pad_count = pads.len();

        // Shared = keyboard OR every gamepad.
        let mut shared = keyboard.clone();
        for pad in &pads {
            shared.merge(pad);
        }
        self.shared = shared;

        // One player per gamepad (at least one). Slot 0 also gets the keyboard.
        let mut players = Vec::with_capacity(pads.len().max(1));
        for i in 0..pads.len().max(1) {
            let mut inp = if i == 0 {
                keyboard.clone()
            } else {
                Input::default()
            };
            if let Some(pad) = pads.get(i) {
                inp.merge(pad);
            }
            players.push(inp);
        }
        self.players = players;
    }

    /// Keyboard + all gamepads. The default input for single-player screens.
    pub fn shared(&self) -> &Input {
        &self.shared
    }

    /// The input controlling party member `i`. Gamepad `i` (keyboard too, for
    /// member 0) if one is plugged in for that slot; otherwise the shared input,
    /// so a lone player still commands every member.
    pub fn player(&self, i: usize) -> &Input {
        self.players.get(i).unwrap_or(&self.shared)
    }

    /// Number of gamepads currently connected.
    pub fn gamepad_count(&self) -> usize {
        self.pad_count
    }
}
