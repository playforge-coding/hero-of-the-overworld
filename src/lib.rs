//! Hero of the Overworld — a small, extensible turn-based JRPG.
//!
//! Rendering, windowing, input, and audio all run on **macroquad** (native and
//! web/WebGL), so the game keeps no custom GPU or windowing stack of its own.
//!
//! Module map:
//!   - [`renderer`] — macroquad-backed 2D sprite renderer (virtual-resolution,
//!     letterboxed).
//!   - [`data`]      — RON file format + registries (the content database).
//!   - [`party`]     — the persistent, extensible party.
//!   - [`overworld`] — tile-mapped world you explore between battles.
//!   - [`battle`]    — turn-based battle scene.
//!   - [`shop`]      — enter-a-store scene: browse and buy gear from a keeper.
//!   - [`cutscene`]  — data-driven scripted dialogue / party recruitment.
//!   - [`audio`]     — background music playback (macroquad audio).
//!   - [`save`]      — persistent save files (native file / web IndexedDB).
//!   - [`game`]      — scene state machine (title → map → level → battle).
//!   - [`app`]       — the macroquad game loop / window config.

pub mod app;
pub mod audio;
pub mod battle;
pub mod cutscene;
pub mod data;
/// **DEV-ONLY** developer menu (set level / add member / fight encounter). Only
/// exists in debug builds — compiled out of `--release`.
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
