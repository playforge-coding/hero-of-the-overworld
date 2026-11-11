//! Hero of the Overworld: a small, extensible turn-based JRPG.
//!
//! Rendering, windowing, input, and audio all run on **macroquad** (native and
//! web/WebGL), so the game keeps no custom GPU or windowing stack of its own.
//!
//! Module map:
//!   - [`renderer`]: macroquad-backed 2D sprite renderer (virtual-resolution,
//!     letterboxed).
//!   - [`data`]: RON file format and registries (the content database).
//!   - [`party`]: the persistent, extensible party.
//!   - [`overworld`]: tile-mapped world you explore between battles.
//!   - [`battle`]: turn-based battle scene.
//!   - [`shop`]: enter-a-store scene where you browse and buy gear from a keeper.
//!   - [`inventory`]: the between-fights menu for gear, items, and the line-up.
//!   - [`cutscene`]: data-driven scripted dialogue and party recruitment.
//!   - [`audio`]: background music playback (macroquad audio).
//!   - [`save`]: persistent save files (a native file or web IndexedDB).
//!   - [`input`]: keyboard, gamepad, and touch input mapped to logical buttons.
//!   - [`input_config`]: the controls screen that assigns input sources to
//!     players.
//!   - [`game`]: the scene state machine (title, then map, then level, then
//!     battle).
//!   - [`app`]: the macroquad game loop and window config.
//!   - [`util`]: small shared helpers (an RNG and the texture cache).
//!   - `devtools`: the debug-only developer menu.

pub mod app;
pub mod audio;
pub mod battle;
pub mod cutscene;
pub mod data;
/// **DEV-ONLY** developer menu (set level, add member, fight encounter). Only
/// exists in debug builds; it is compiled out of `--release`.
#[cfg(debug_assertions)]
pub mod devtools;
pub mod game;
pub mod input;
pub mod input_config;
pub mod inventory;
pub mod overworld;
pub mod party;
pub mod renderer;
pub mod save;
pub mod shop;
pub mod util;

pub use app::{run, window_conf};
