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
//!   - [`Controllers::player`] — the input for one *party member*. Each gamepad
//!     (plus the keyboard, on slot 0) is a player, and members are dealt to
//!     players **round-robin**: gamepad 0 drives member 0, gamepad 1 member 1, and
//!     with fewer pads than members it wraps, so two pads split three heroes
//!     cleanly (player 1 → members 0 & 2, player 2 → member 1). One input source
//!     (a lone pad, or just the keyboard) maps every member to it, so a single
//!     player still commands the whole party.
//!
//! macroquad reports both the held state and the press *edge* for a key (and
//! latches a press even if the key is released within the same frame, as
//! automation tools do), so keyboard `pressed` stays frame-accurate. Gamepad
//! press edges are derived here by diffing against the previous frame.
//!
//! Gamepad support is cross-platform: the native build uses `gilrs`, and the web
//! build reads the browser Gamepad API through the `hoto_gamepads` JS plugin (see
//! `hoto_gamepads.js`). Both feed the same per-pad logical snapshots, so pads work
//! identically — including local co-op — on desktop and in the browser.
//!
//! Touchscreens have neither a keyboard nor a gamepad, so [`Controllers`] also
//! folds in on-screen controls: a directional control plus action buttons laid
//! out along the screen edges (see [`touch_layout`]). The directional control
//! adapts to the current scene ([`TouchScheme`]): an analog **joystick** while
//! walking the overworld, a plain **d-pad** on menus, and just **up/down** in
//! battle, where the command menu only moves vertically. The overlay stays
//! hidden until the first touch is seen, so a desktop keyboard/mouse session
//! never shows it — this needs no platform `cfg`, since `touches()` is simply
//! always empty without a touchscreen. Like a gamepad, the touch controls drive
//! the shared input and party member 0.

use macroquad::prelude::{
    is_key_down, is_key_pressed, screen_dpi_scale, screen_height, screen_width, touches, KeyCode,
    Touch, TouchPhase,
};

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

/// **DEV-ONLY** hidden hotkey (Tab) used by the world map to skip levels, so a
/// developer can advance through the game without grinding. Reads the raw key
/// directly — deliberately *outside* the normal logical [`Button`] mapping so it
/// can't be bound to a pad or triggered by ordinary play.
///
/// The whole function is compiled out of `--release` builds (`debug_assertions`
/// is off there), so the skip simply does not exist in a shipped game.
#[cfg(debug_assertions)]
pub fn dev_skip_pressed() -> bool {
    is_key_pressed(KeyCode::Tab)
}

/// **DEV-ONLY** hidden hotkey (R) used by the world map to reset the highlighted
/// level: it clears the level's cleared flag and defeated-enemy progress and
/// forgets its cutscenes, so a developer can walk back in and **replay it fresh**
/// (every foe respawned, the intro playing again) after tuning it. The mirror of
/// [`dev_skip_pressed`]. Read raw, outside the logical [`Button`] mapping, and
/// compiled out of `--release` builds, so a shipped game has no such reset.
#[cfg(debug_assertions)]
pub fn dev_reset_pressed() -> bool {
    is_key_pressed(KeyCode::R)
}

/// **DEV-ONLY** hidden hotkey (F1) that opens the developer menu from the world
/// map — a menu to set the party's level, add any character, or fight any
/// encounter (see [`crate::devtools`]). Like the other dev hotkeys it reads the
/// raw key, deliberately *outside* the logical [`Button`] mapping so it can't be
/// bound to a pad or triggered by ordinary play, and is compiled out of
/// `--release` builds so a shipped game has no such menu.
#[cfg(debug_assertions)]
pub fn dev_menu_pressed() -> bool {
    is_key_pressed(KeyCode::F1)
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

/// Which on-screen directional control the touch overlay shows, chosen by the
/// current game scene (see [`crate::game::Game::touch_scheme`]).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TouchScheme {
    /// A 4-way d-pad. The default for menu-like screens (title, map, shop).
    Dpad,
    /// An analog joystick. Used while walking the overworld, where any direction
    /// (including diagonals) matters.
    Joystick,
    /// Only Up/Down buttons. Used in battle, whose command menu is a vertical
    /// list, so left/right (and the menu button) would do nothing.
    UpDown,
}

/// One on-screen touch button: the logical [`Button`] it maps to, its rectangle
/// in *logical screen pixels*, and the glyph drawn on it. Produced by
/// [`touch_layout`] and read back by the renderer to paint the overlay.
#[derive(Copy, Clone, Debug)]
pub struct TouchButton {
    pub button: Button,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: &'static str,
}

impl TouchButton {
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// A *floating* analog joystick, all in *logical screen pixels*. It rests at its
/// `home` position, but a touch anywhere in its `zone` (the bottom-left region)
/// re-anchors the base under that finger and then follows *that finger by its
/// touch id* anywhere on screen until it lifts — so the stick never cuts out
/// just because the thumb drifted past the ring. The knob is clamped to `radius`
/// and its deflection past a deadzone becomes direction presses. Produced by
/// [`touch_layout`] for [`TouchScheme::Joystick`].
#[derive(Copy, Clone, Debug)]
pub struct TouchStick {
    /// Resting base centre, drawn while no finger owns the stick.
    pub home_x: f32,
    pub home_y: f32,
    pub radius: f32,
    /// Region a fresh touch must start in to grab the stick (once grabbed, the
    /// finger is tracked by id and may leave this region).
    pub zone_x: f32,
    pub zone_y: f32,
    pub zone_w: f32,
    pub zone_h: f32,
}

impl TouchStick {
    fn zone_contains(&self, px: f32, py: f32) -> bool {
        px >= self.zone_x
            && px < self.zone_x + self.zone_w
            && py >= self.zone_y
            && py < self.zone_y + self.zone_h
    }
}

/// The joystick as the renderer needs to draw it: the base plus the live knob
/// position (absolute logical pixels) and whether it's currently deflected.
#[derive(Copy, Clone, Debug)]
pub struct StickView {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub knob_x: f32,
    pub knob_y: f32,
    pub active: bool,
}

/// Knob deflection past this fraction of the base radius counts as a direction
/// press (per axis, so diagonals need both axes past it). Mirrors the gamepad
/// [`STICK_DEADZONE`] in feel.
const STICK_DIR_DEADZONE: f32 = 0.35;

/// Lay the virtual controls out for a `sw`×`sh` (logical-pixel) screen. The
/// directional control (bottom-left) depends on `scheme`; Confirm/Cancel anchor
/// the bottom-right and Menu the top-right (dropped for [`TouchScheme::UpDown`],
/// which is battle-only). Sizes scale with the screen's short edge and are
/// clamped so they stay thumb-sized on phones without swallowing a small desktop
/// window. Recomputed each frame so it tracks rotation and resizes. Returns the
/// tappable buttons and, for [`TouchScheme::Joystick`], the joystick base.
pub fn touch_layout(
    scheme: TouchScheme,
    sw: f32,
    sh: f32,
) -> (Vec<TouchButton>, Option<TouchStick>) {
    let u = (sw.min(sh) * 0.16).clamp(36.0, 96.0);
    // Inset from the screen edge. Kept generous so the controls sit within a
    // resting thumb's arc instead of jammed into the very corner (which is
    // awkward to press), but capped in absolute pixels so they don't drift too
    // far inward on a large tablet where `u` has hit its clamp.
    let m = u.min(sw.min(sh) * 0.12);

    // Directional cluster: centred on (pcx, pcy) in the bottom-left corner. The
    // d-pad plus and the joystick base share this footprint (radius 1.5·u).
    let pcx = m + 1.5 * u;
    let pcy = sh - m - 1.5 * u;
    let dpad = |b, label, dx: f32, dy: f32| TouchButton {
        button: b,
        x: pcx + dx * u - u / 2.0,
        y: pcy + dy * u - u / 2.0,
        w: u,
        h: u,
        label,
    };

    let mut buttons = Vec::new();
    let mut stick = None;
    match scheme {
        TouchScheme::Dpad => buttons.extend([
            dpad(Button::Up, "^", 0.0, -1.0),
            dpad(Button::Down, "v", 0.0, 1.0),
            dpad(Button::Left, "<", -1.0, 0.0),
            dpad(Button::Right, ">", 1.0, 0.0),
        ]),
        TouchScheme::UpDown => buttons.extend([
            dpad(Button::Up, "^", 0.0, -1.0),
            dpad(Button::Down, "v", 0.0, 1.0),
        ]),
        TouchScheme::Joystick => {
            // The claim zone is the lower-left of the screen — big enough to land
            // a thumb without aiming, clear of the action buttons on the right.
            stick = Some(TouchStick {
                home_x: pcx,
                home_y: pcy,
                radius: 1.5 * u,
                zone_x: 0.0,
                zone_y: sh * 0.35,
                zone_w: sw * 0.5,
                zone_h: sh * 0.65,
            })
        }
    }

    // Action buttons: Confirm at the bottom-right, Cancel up-and-left of it.
    let action = |b, label, x: f32, y: f32| TouchButton {
        button: b,
        x,
        y,
        w: u,
        h: u,
        label,
    };
    buttons.push(action(Button::Confirm, "A", sw - m - u, sh - m - u));
    buttons.push(action(
        Button::Cancel,
        "B",
        sw - m - 2.1 * u,
        sh - m - 1.6 * u,
    ));
    if scheme != TouchScheme::UpDown {
        let menu_u = u * 0.8;
        buttons.push(TouchButton {
            button: Button::Menu,
            x: sw - m - menu_u,
            y: m,
            w: menu_u,
            h: menu_u,
            label: "=",
        });
    }

    (buttons, stick)
}

/// Whether a touch point is currently down (not ended/cancelled this frame).
fn is_active(t: &Touch) -> bool {
    !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled)
}

/// The floating joystick carried between frames: which touch id currently owns
/// it (if any) and where its base is anchored (logical px). Threaded through
/// [`read_touch`] so the stick can follow one finger across frames.
#[derive(Copy, Clone, Debug, Default)]
pub struct StickState {
    /// The `touches()` id that grabbed the stick, or `None` when at rest.
    id: Option<u64>,
    /// Base centre: where the owning finger first touched, else the home rest.
    origin: (f32, f32),
    /// Knob deflection from `origin` (clamped to radius), for drawing.
    knob: (f32, f32),
}

/// One frame's touch reading: the merged logical [`Input`] and the carried-over
/// joystick state (fed back in next frame as `prev_stick`).
struct TouchRead {
    input: Input,
    stick: StickState,
}

/// Read the touchscreen into a [`TouchRead`]. `touches()` reports positions in
/// *physical* pixels, so scale by the live DPI to match the layout's
/// logical-pixel geometry, then delegate to [`read_touch_at`].
fn read_touch(
    active: &[Touch],
    buttons: &[TouchButton],
    stick: Option<&TouchStick>,
    prev_down: &[bool; N],
    prev_stick: StickState,
) -> TouchRead {
    read_touch_at(
        active,
        buttons,
        stick,
        prev_down,
        prev_stick,
        screen_dpi_scale().max(0.01),
    )
}

/// Advance the floating joystick. Keeps following `prev.id` while that finger is
/// still down; otherwise a fresh touch starting in the stick's zone (and not on
/// a button) grabs it, anchoring the base under that finger. The knob is the
/// finger's offset from that anchor, clamped to `radius`; past a per-axis
/// deadzone it becomes direction presses. Pure, so it's unit-testable.
fn track_stick(
    active: &[Touch],
    buttons: &[TouchButton],
    stick: &TouchStick,
    prev: StickState,
    dpi: f32,
) -> ([bool; N], StickState) {
    let logical = |t: &Touch| (t.position.x / dpi, t.position.y / dpi);

    // Keep the finger we were already tracking; failing that, claim a new one
    // that started inside the zone and isn't pressing an action button.
    let owner = prev
        .id
        .and_then(|id| active.iter().find(|t| t.id == id))
        .or_else(|| {
            active.iter().find(|t| {
                let (px, py) = logical(t);
                stick.zone_contains(px, py) && !buttons.iter().any(|b| b.contains(px, py))
            })
        });

    let mut down = [false; N];
    let Some(t) = owner else {
        // Nobody's holding it: rest at home, no deflection.
        return (
            down,
            StickState {
                id: None,
                origin: (stick.home_x, stick.home_y),
                knob: (0.0, 0.0),
            },
        );
    };

    let (px, py) = logical(t);
    // Re-anchor only when a *new* finger grabs the stick; a tracked finger keeps
    // the base it first pressed, so the knob measures travel from there.
    let origin = if prev.id == Some(t.id) {
        prev.origin
    } else {
        (px, py)
    };
    let (mut dx, mut dy) = (px - origin.0, py - origin.1);
    let mag = (dx * dx + dy * dy).sqrt();
    if mag > stick.radius {
        dx = dx / mag * stick.radius;
        dy = dy / mag * stick.radius;
    }
    let dz = stick.radius * STICK_DIR_DEADZONE;
    down[index(Button::Left)] = dx < -dz;
    down[index(Button::Right)] = dx > dz;
    down[index(Button::Up)] = dy < -dz;
    down[index(Button::Down)] = dy > dz;
    (
        down,
        StickState {
            id: Some(t.id),
            origin,
            knob: (dx, dy),
        },
    )
}

/// The DPI-parameterised core of [`read_touch`] (kept context-free for tests).
/// Each active touch presses whichever button its (physical) position falls
/// inside once divided by `dpi`, and the finger owning the joystick adds its
/// direction; `prev_down` (last frame's held state) yields the press edges.
fn read_touch_at(
    active: &[Touch],
    buttons: &[TouchButton],
    stick: Option<&TouchStick>,
    prev_down: &[bool; N],
    prev_stick: StickState,
    dpi: f32,
) -> TouchRead {
    let mut down = [false; N];
    for t in active {
        let (px, py) = (t.position.x / dpi, t.position.y / dpi);
        for tb in buttons {
            if tb.contains(px, py) {
                down[index(tb.button)] = true;
            }
        }
    }
    let stick_state = if let Some(s) = stick {
        let (sdown, state) = track_stick(active, buttons, s, prev_stick, dpi);
        for i in 0..N {
            down[i] |= sdown[i];
        }
        state
    } else {
        StickState::default()
    };
    let mut inp = Input::default();
    for i in 0..N {
        inp.down[i] = down[i];
        inp.pressed[i] = down[i] && !prev_down[i];
    }
    TouchRead {
        input: inp,
        stick: stick_state,
    }
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

/// Web gamepad bridge. macroquad/miniquad has no gamepad support, so on the web
/// build we read the browser Gamepad API through the `hoto_gamepads` JS plugin
/// (see `hoto_gamepads.js`), which maps each pad to our [`N`] logical buttons and
/// hands back a flat buffer of `count * N` 0/1 flags — the same button layout
/// [`read_pad`] produces natively.
#[cfg(target_arch = "wasm32")]
mod web_pads {
    use super::N;
    use sapp_jsutils::JsObject;

    extern "C" {
        fn hoto_gamepads_poll() -> JsObject;
    }

    /// This frame's pad flags: `count * N` bytes, one 0/1 per logical button per
    /// pad (order Up, Down, Left, Right, Confirm, Cancel, Menu). Truncated to a
    /// whole number of pads so a short/garbled buffer can't misalign.
    pub fn poll_raw() -> Vec<u8> {
        let obj = unsafe { hoto_gamepads_poll() };
        if obj.is_nil() || obj.is_undefined() {
            return Vec::new();
        }
        let mut buf = Vec::new();
        obj.to_byte_buffer(&mut buf);
        let usable = buf.len() - (buf.len() % N);
        buf.truncate(usable);
        buf
    }
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
    /// Whether a touch has ever been seen; once true the on-screen controls are
    /// drawn and drive input. Stays false for keyboard/gamepad-only sessions.
    touch_active: bool,
    /// Current-frame on-screen button layout (empty until `touch_active`).
    touch_buttons: Vec<TouchButton>,
    /// Current-frame joystick geometry, present only under
    /// [`TouchScheme::Joystick`].
    touch_stick: Option<TouchStick>,
    /// Floating-joystick state (owning finger + anchor), carried across frames.
    stick_state: StickState,
    /// Previous-frame held state for the touch controls, for press edges.
    prev_touch: [bool; N],
    #[cfg(not(target_arch = "wasm32"))]
    gilrs: Option<Gilrs>,
    /// Previous-frame held state per connected pad, keyed by id, for press edges.
    #[cfg(not(target_arch = "wasm32"))]
    prev_pad_down: Vec<(GamepadId, [bool; N])>,
    /// Web equivalent: previous-frame held state per pad, by index (the browser
    /// Gamepad API's stable slot), for deriving press edges.
    #[cfg(target_arch = "wasm32")]
    prev_pad_down_web: Vec<[bool; N]>,
}

/// Which player-input slot commands party `member`, given `player_count` input
/// sources (connected pads, or 1 for keyboard-only). Round-robin, so P players
/// cover N members with none left unowned; returns 0 when there are no sources.
fn member_slot(member: usize, player_count: usize) -> usize {
    if player_count == 0 {
        0
    } else {
        member % player_count
    }
}

/// Who controls whom: the **player number** each input source is bound to. This is
/// the in-game-configurable mapping — assign the keyboard to player 1 and a pad to
/// player 2 and the two share the party; leave the keyboard and first pad on the
/// same player and one person drives both. Missing gamepad entries default to the
/// pad's own index (so pad 0 shares player 0 with the keyboard, pad 1 is player 1,
/// …), which reproduces the original "each pad is the next player" behaviour.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InputAssignment {
    /// Player number (0-based) the keyboard belongs to.
    pub keyboard: usize,
    /// Player number for each connected gamepad, by connection order.
    pub gamepads: Vec<usize>,
}

impl InputAssignment {
    /// Player number bound to gamepad `i` — the stored value, or `i` by default.
    pub fn gamepad_player(&self, i: usize) -> usize {
        self.gamepads.get(i).copied().unwrap_or(i)
    }

    /// Grow `gamepads` to cover `count` pads, filling new slots with their default
    /// (the pad's own index). Called when the config screen opens so every
    /// connected pad has an editable entry.
    pub fn ensure_gamepads(&mut self, count: usize) {
        for i in self.gamepads.len()..count {
            self.gamepads.push(i);
        }
    }
}

/// One resolved player: which sources feed it. Ordered by player number, with
/// empty player numbers dropped, so members map onto real (non-empty) players.
struct PlayerGroup {
    keyboard: bool,
    pads: Vec<usize>,
}

/// Collapse a source→player-number assignment into the ordered list of *effective*
/// players (those with at least one source). Player numbers with no source are
/// skipped, so a gappy assignment (keyboard→P2, pad→P3) still yields two usable
/// players rather than empty, uncontrollable slots.
fn group_players(keyboard_player: usize, pad_players: &[usize]) -> Vec<PlayerGroup> {
    let mut nums: Vec<usize> = std::iter::once(keyboard_player)
        .chain(pad_players.iter().copied())
        .collect();
    nums.sort_unstable();
    nums.dedup();
    nums.into_iter()
        .map(|p| PlayerGroup {
            keyboard: keyboard_player == p,
            pads: (0..pad_players.len())
                .filter(|&i| pad_players[i] == p)
                .collect(),
        })
        .collect()
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
            touch_active: false,
            touch_buttons: Vec::new(),
            touch_stick: None,
            stick_state: StickState::default(),
            prev_touch: [false; N],
            #[cfg(not(target_arch = "wasm32"))]
            gilrs,
            #[cfg(not(target_arch = "wasm32"))]
            prev_pad_down: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            prev_pad_down_web: Vec::new(),
        }
    }

    /// Refresh keyboard and gamepad state. Call once per frame before the game
    /// reads input. `scheme` (from the current scene) selects which on-screen
    /// directional control to show and read. `assign` maps each input source
    /// (keyboard, each pad) to a player number, so the party can be split between
    /// people (see [`InputAssignment`]).
    pub fn poll(&mut self, scheme: TouchScheme, assign: &InputAssignment) {
        let keyboard = read_keyboard();

        // Collect each connected gamepad's (held, press-edge) state, in a stable
        // order so pad → party-member assignment doesn't shuffle between frames.
        // Native reads pads via gilrs; web reads the browser Gamepad API through
        // the `hoto_gamepads` JS plugin. Either way, press edges are derived here
        // by diffing this frame's held state against the previous frame's.
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

        // Web: the JS plugin already maps each pad to our N logical buttons, so it
        // hands back `count * N` 0/1 flags. Diff against last frame (by pad index,
        // which is stable per connection) for press edges.
        #[cfg(target_arch = "wasm32")]
        {
            let raw = web_pads::poll_raw();
            let count = raw.len() / N;
            let mut next_prev: Vec<[bool; N]> = Vec::with_capacity(count);
            for p in 0..count {
                let mut down = [false; N];
                for (i, d) in down.iter_mut().enumerate() {
                    *d = raw[p * N + i] != 0;
                }
                let prev = self.prev_pad_down_web.get(p).copied().unwrap_or([false; N]);
                let mut inp = Input::default();
                for i in 0..N {
                    inp.down[i] = down[i];
                    inp.pressed[i] = down[i] && !prev[i];
                }
                pads.push(inp);
                next_prev.push(down);
            }
            self.prev_pad_down_web = next_prev;
        }

        self.pad_count = pads.len();

        // Fold in the on-screen touch controls. The layout tracks the live
        // screen size and scene; a touch anywhere flips the overlay on for the
        // rest of the session so it also drives (and is drawn for) later frames.
        let (buttons, stick) = touch_layout(scheme, screen_width(), screen_height());
        self.touch_buttons = buttons;
        self.touch_stick = stick;
        let active: Vec<Touch> = touches().into_iter().filter(is_active).collect();
        if !active.is_empty() {
            self.touch_active = true;
        }
        let read = read_touch(
            &active,
            &self.touch_buttons,
            self.touch_stick.as_ref(),
            &self.prev_touch,
            self.stick_state,
        );
        let touch = read.input;
        self.prev_touch = touch.down;
        self.stick_state = read.stick;

        // Shared = keyboard OR every gamepad OR the touch pad.
        let mut shared = keyboard.clone();
        for pad in &pads {
            shared.merge(pad);
        }
        shared.merge(&touch);
        self.shared = shared;

        // Build one merged Input per *effective* player from the assignment: each
        // player is the OR of the sources bound to it. The keyboard (and the touch
        // overlay, which mirrors it) rides on the keyboard's player. Members are
        // dealt across these players round-robin by `player()` / `member_slot`.
        let pad_players: Vec<usize> = (0..pads.len()).map(|i| assign.gamepad_player(i)).collect();
        let mut players = Vec::new();
        for group in group_players(assign.keyboard, &pad_players) {
            let mut inp = Input::default();
            if group.keyboard {
                inp.merge(&keyboard);
                inp.merge(&touch);
            }
            for i in group.pads {
                inp.merge(&pads[i]);
            }
            players.push(inp);
        }
        self.players = players;
    }

    /// Keyboard + all gamepads. The default input for single-player screens.
    pub fn shared(&self) -> &Input {
        &self.shared
    }

    /// The input controlling party member `i`. Each connected gamepad (plus the
    /// keyboard, on slot 0) is one player; members are handed to players
    /// **round-robin**, so when there are fewer players than members every member
    /// still has exactly one clear owner. Two pads with three heroes therefore
    /// split cleanly — player 1 drives members 0 & 2, player 2 drives member 1 —
    /// rather than the extra hero becoming a both-pads free-for-all. With a single
    /// input source (one pad, or just the keyboard) every member maps to it, so a
    /// lone player still commands the whole party.
    pub fn player(&self, i: usize) -> &Input {
        if self.players.is_empty() {
            return &self.shared;
        }
        &self.players[member_slot(i, self.players.len())]
    }

    /// Number of gamepads currently connected.
    pub fn gamepad_count(&self) -> usize {
        self.pad_count
    }

    /// The on-screen touch buttons to draw this frame, or an empty slice while
    /// no touch has been seen (keyboard/gamepad sessions draw nothing).
    pub fn touch_overlay(&self) -> &[TouchButton] {
        if self.touch_active {
            &self.touch_buttons
        } else {
            &[]
        }
    }

    /// The joystick to draw this frame, if the current scene shows one and a
    /// touch has been seen. The base sits at the live anchor (the home rest until
    /// a finger grabs it), with the knob offset by its deflection, so the
    /// renderer can paint the floating base and thumb.
    pub fn touch_stick(&self) -> Option<StickView> {
        let s = self.touch_stick.as_ref().filter(|_| self.touch_active)?;
        let (ox, oy) = self.stick_state.origin;
        let (dx, dy) = self.stick_state.knob;
        Some(StickView {
            cx: ox,
            cy: oy,
            radius: s.radius,
            knob_x: ox + dx,
            knob_y: oy + dy,
            active: self.stick_state.id.is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The button whose rect a screen point falls in, if any.
    fn hit(layout: &[TouchButton], x: f32, y: f32) -> Option<Button> {
        layout
            .iter()
            .find(|tb| tb.contains(x, y))
            .map(|tb| tb.button)
    }

    fn center(tb: &TouchButton) -> (f32, f32) {
        (tb.x + tb.w / 2.0, tb.y + tb.h / 2.0)
    }

    fn touch(id: u64, x: f32, y: f32) -> Touch {
        Touch {
            id,
            phase: TouchPhase::Started,
            position: macroquad::prelude::vec2(x, y),
        }
    }

    /// Compact description of a grouping for assertions: for each effective
    /// player, `(has_keyboard, pad_indices)`.
    fn groups(kb: usize, pads: &[usize]) -> Vec<(bool, Vec<usize>)> {
        group_players(kb, pads)
            .into_iter()
            .map(|g| (g.keyboard, g.pads))
            .collect()
    }

    #[test]
    fn default_assignment_merges_keyboard_with_the_first_pad() {
        // Default: keyboard→P0, pad0→P0, pad1→P1. So one couch player uses the
        // keyboard and pad 0 together, and a second pad is its own player.
        assert_eq!(groups(0, &[0, 1]), vec![(true, vec![0]), (false, vec![1])]);
    }

    #[test]
    fn keyboard_and_pad_can_be_separate_players() {
        // The headline case: assign the keyboard to P0 and the one pad to P1 and
        // they become two distinct players — one on keys, one on the controller.
        assert_eq!(groups(0, &[1]), vec![(true, vec![]), (false, vec![0])]);
    }

    #[test]
    fn gappy_assignment_compacts_to_real_players() {
        // Keyboard→P1, pad→P2 (nobody on P0): still two usable players, no empty
        // slot that would leave a member uncontrollable.
        assert_eq!(groups(1, &[2]), vec![(true, vec![]), (false, vec![0])]);
    }

    #[test]
    fn everyone_on_one_player_is_solo() {
        // Keyboard + two pads all on P0 → a single player driving the whole party.
        assert_eq!(groups(0, &[0, 0]), vec![(true, vec![0, 1])]);
        // Keyboard alone is also one player.
        assert_eq!(groups(0, &[]), vec![(true, vec![])]);
    }

    #[test]
    fn members_are_dealt_to_players_round_robin() {
        // Two players, three heroes: player 1 owns members 0 & 2, player 2 owns 1.
        assert_eq!(member_slot(0, 2), 0);
        assert_eq!(member_slot(1, 2), 1);
        assert_eq!(member_slot(2, 2), 0);
        // One input source (lone pad / keyboard): every member maps to it.
        for m in 0..4 {
            assert_eq!(member_slot(m, 1), 0);
        }
        // Three players, three heroes: one each.
        assert_eq!(member_slot(2, 3), 2);
        // No sources: defaults to slot 0 rather than dividing by zero.
        assert_eq!(member_slot(5, 0), 0);
    }

    #[test]
    fn dpad_scheme_has_every_button_on_screen() {
        let (w, h) = (400.0, 800.0); // a portrait phone
        let (layout, stick) = touch_layout(TouchScheme::Dpad, w, h);
        assert!(stick.is_none(), "d-pad scheme has no joystick");
        for b in ALL {
            let tb = layout
                .iter()
                .find(|tb| tb.button == b)
                .unwrap_or_else(|| panic!("{b:?} missing from d-pad layout"));
            assert!(
                tb.x >= 0.0 && tb.y >= 0.0 && tb.x + tb.w <= w && tb.y + tb.h <= h,
                "{b:?} spills off screen: {:?}",
                (tb.x, tb.y, tb.w, tb.h)
            );
        }
    }

    #[test]
    fn battle_scheme_is_up_down_confirm_cancel_only() {
        let (layout, stick) = touch_layout(TouchScheme::UpDown, 400.0, 800.0);
        assert!(stick.is_none(), "battle scheme has no joystick");
        let present: Vec<Button> = layout.iter().map(|tb| tb.button).collect();
        for b in [Button::Up, Button::Down, Button::Confirm, Button::Cancel] {
            assert!(present.contains(&b), "battle layout should keep {b:?}");
        }
        for b in [Button::Left, Button::Right, Button::Menu] {
            assert!(!present.contains(&b), "battle layout should drop {b:?}");
        }
    }

    #[test]
    fn joystick_scheme_swaps_the_dpad_for_a_stick() {
        let (layout, stick) = touch_layout(TouchScheme::Joystick, 400.0, 800.0);
        let stick = stick.expect("joystick scheme has a stick");
        // The stick rests bottom-left and stays on screen.
        assert!(stick.home_x - stick.radius >= 0.0 && stick.home_y + stick.radius <= 800.0);
        // The directions live on the stick now, not as buttons; A/B/Menu remain.
        for b in [Button::Up, Button::Down, Button::Left, Button::Right] {
            assert!(
                !layout.iter().any(|tb| tb.button == b),
                "{b:?} is on the stick"
            );
        }
        for b in [Button::Confirm, Button::Cancel, Button::Menu] {
            assert!(layout.iter().any(|tb| tb.button == b), "{b:?} missing");
        }
    }

    #[test]
    fn tapping_a_buttons_center_hits_exactly_that_button() {
        let (layout, _) = touch_layout(TouchScheme::Dpad, 400.0, 800.0);
        for tb in &layout {
            let (cx, cy) = center(tb);
            assert_eq!(
                hit(&layout, cx, cy),
                Some(tb.button),
                "{:?}'s center should hit itself",
                tb.button
            );
        }
    }

    #[test]
    fn controls_sit_in_the_expected_corners() {
        let (w, h) = (400.0, 800.0);
        let (layout, _) = touch_layout(TouchScheme::Dpad, w, h);
        let c = |b| center(layout.iter().find(|tb| tb.button == b).unwrap());
        // D-pad clusters bottom-left; the plus arms sit around a shared center.
        assert!(c(Button::Up).1 < c(Button::Down).1, "up above down");
        assert!(c(Button::Left).0 < c(Button::Right).0, "left of right");
        // Confirm anchors the bottom-right; Menu the top-right.
        assert!(c(Button::Confirm).0 > w / 2.0 && c(Button::Confirm).1 > h / 2.0);
        assert!(c(Button::Menu).0 > w / 2.0 && c(Button::Menu).1 < h / 2.0);
    }

    #[test]
    fn read_touch_maps_positions_and_edges() {
        // Screen-space taps (dpi 1.0 in the test path is handled by the caller;
        // here we exercise the pure mapping with pre-divided positions).
        let (layout, _) = touch_layout(TouchScheme::Dpad, 400.0, 800.0);
        let up = layout.iter().find(|tb| tb.button == Button::Up).unwrap();
        let (ux, uy) = center(up);

        // First frame with a finger on Up: held and a fresh press edge.
        let prev = [false; N];
        let r = read_touch_at(
            &[touch(0, ux, uy)],
            &layout,
            None,
            &prev,
            StickState::default(),
            1.0,
        );
        assert!(r.input.held(Button::Up) && r.input.pressed(Button::Up));
        // Held again next frame: still down, but no new press edge.
        let r2 = read_touch_at(
            &[touch(0, ux, uy)],
            &layout,
            None,
            &r.input.down,
            StickState::default(),
            1.0,
        );
        assert!(r2.input.held(Button::Up) && !r2.input.pressed(Button::Up));
    }

    /// The floating joystick anchors under the first touch (no deflection yet),
    /// then reads direction from how far that finger has since travelled.
    #[test]
    fn joystick_floats_to_first_touch_then_reads_travel() {
        let (buttons, stick) = touch_layout(TouchScheme::Joystick, 400.0, 800.0);
        let stick = stick.unwrap();
        let (ax, ay, r) = (stick.home_x, stick.home_y, stick.radius);
        let read = |t: &[Touch], prev: StickState| {
            read_touch_at(t, &buttons, Some(&stick), &[false; N], prev, 1.0)
        };

        // Frame 1: finger lands (anywhere in the zone) — anchors, no direction.
        let f1 = read(&[touch(7, ax, ay)], StickState::default());
        assert!(![Button::Up, Button::Down, Button::Left, Button::Right]
            .iter()
            .any(|&b| f1.input.held(b)));
        assert_eq!(f1.stick.id, Some(7));

        // Frame 2: same finger drags up-and-right → Up + Right, knob follows.
        let f2 = read(&[touch(7, ax + r * 0.5, ay - r * 0.5)], f1.stick);
        assert!(f2.input.held(Button::Up) && f2.input.held(Button::Right));
        assert!(!f2.input.held(Button::Down) && !f2.input.held(Button::Left));
        assert!(f2.stick.knob.0 > 0.0 && f2.stick.knob.1 < 0.0);

        // Frame 3: finger lifts → back to rest at home, no direction.
        let f3 = read(&[], f2.stick);
        assert_eq!(f3.stick.id, None);
        assert_eq!(f3.stick.origin, (ax, ay));
        assert!(!f3.input.held(Button::Right));
    }

    /// The regression that motivated the floating design: dragging the finger far
    /// past the base ring must *keep* driving the direction (knob clamped to the
    /// radius), not cut out.
    #[test]
    fn joystick_keeps_driving_when_finger_leaves_the_ring() {
        let (buttons, stick) = touch_layout(TouchScheme::Joystick, 400.0, 800.0);
        let stick = stick.unwrap();
        let (ax, ay, r) = (stick.home_x, stick.home_y, stick.radius);
        let read = |t: &[Touch], prev: StickState| {
            read_touch_at(t, &buttons, Some(&stick), &[false; N], prev, 1.0)
        };

        let f1 = read(&[touch(1, ax, ay)], StickState::default());
        // Yank the finger way past the ring (3× the radius to the right).
        let f2 = read(&[touch(1, ax + r * 3.0, ay)], f1.stick);
        assert!(
            f2.input.held(Button::Right),
            "still driving Right past the ring"
        );
        // Knob is clamped to the ring, not off at the finger.
        assert!((f2.stick.knob.0 - r).abs() < 0.01);
    }

    /// A fresh touch that starts outside the zone (or on an action button) does
    /// not grab the stick.
    #[test]
    fn joystick_ignores_touches_outside_its_zone() {
        let (buttons, stick) = touch_layout(TouchScheme::Joystick, 400.0, 800.0);
        let stick = stick.unwrap();
        // Top-right corner: outside the lower-left claim zone.
        let out = read_touch_at(
            &[touch(2, 360.0, 40.0)],
            &buttons,
            Some(&stick),
            &[false; N],
            StickState::default(),
            1.0,
        );
        assert_eq!(
            out.stick.id, None,
            "a top-right touch shouldn't grab the stick"
        );

        // A tap on the Confirm button isn't stolen by the stick either.
        let a = buttons
            .iter()
            .find(|b| b.button == Button::Confirm)
            .unwrap();
        let (acx, acy) = center(a);
        let on_btn = read_touch_at(
            &[touch(3, acx, acy)],
            &buttons,
            Some(&stick),
            &[false; N],
            StickState::default(),
            1.0,
        );
        assert_eq!(on_btn.stick.id, None);
        assert!(on_btn.input.held(Button::Confirm));
    }
}
