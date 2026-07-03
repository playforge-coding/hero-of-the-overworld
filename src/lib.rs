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
pub mod game;
pub mod input;
pub mod overworld;
pub mod party;
pub mod renderer;
pub mod save;
pub mod util;

pub use app::{run, window_conf};
