//! macroquad main loop. macroquad owns the window, GL context, input, and audio
//! on both native and web; this module just wires the game's update/draw into
//! its per-frame loop.

use macroquad::prelude::{get_frame_time, next_frame, Conf};

use crate::audio::Audio;
use crate::data::{BATTLE_MUSIC_OGG, FONT_TTF};
use crate::game::Game;
use crate::input::Controllers;
use crate::renderer::Renderer;

/// Window configuration passed to `#[macroquad::main]`. Honours
/// `HOTO_TEST_WINDOW` (used by the e2e suite) to run fullscreen so the game owns
/// the whole screen for reproducible screenshots.
pub fn window_conf() -> Conf {
    let fullscreen = std::env::var("HOTO_TEST_WINDOW").is_ok();
    Conf {
        window_title: "Hero of the Overworld".to_owned(),
        window_width: 960,
        window_height: 540,
        fullscreen,
        high_dpi: true,
        ..Default::default()
    }
}

/// The async game loop. Called from the `#[macroquad::main]` entry in `main.rs`.
pub async fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init();

    // Preload the looping music (async) before entering the loop.
    let mut audio = Audio::new();
    audio.load_music(BATTLE_MUSIC_OGG).await;

    let mut renderer = Renderer::new(FONT_TTF);
    let mut game = Game::new(&mut renderer, audio);
    let mut controllers = Controllers::new();

    loop {
        // Cap dt so a stall (or a paused tab) can't teleport the simulation.
        let dt = get_frame_time().min(0.05);

        controllers.poll();
        game.update(&controllers, &mut renderer, dt);
        game.draw(&mut renderer);
        renderer.render();

        next_frame().await;
    }
}
