//! Hero of the Overworld — a small, extensible turn-based JRPG.
//!
//! Rendering: wgpu. Windowing/input: winit (native + web). Web build: trunk.
//!
//! Module map:
//!   - [`renderer`] — wgpu 2D sprite renderer (virtual-resolution, letterboxed).
//!   - [`data`]      — RON file format + registries (the content database).
//!   - [`party`]     — the persistent, extensible party.
//!   - [`overworld`] — tile-mapped world you explore between battles.
//!   - [`battle`]    — turn-based battle scene.
//!   - [`cutscene`]  — data-driven scripted dialogue / party recruitment.
//!   - [`audio`]     — background music playback (native; no-op on web).
//!   - [`game`]      — scene state machine (title → map → level → battle).
//!   - [`app`]       — winit application handler / entry point.

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
pub mod util;

/// Native/shared entry point.
pub fn start() {
    init_logging();
    app::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
}

#[cfg(target_arch = "wasm32")]
fn init_logging() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Warn);
}

/// Web entry point, called automatically by the trunk-generated JS glue.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    start();
}
