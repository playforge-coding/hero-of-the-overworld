//! macroquad main loop. macroquad owns the window, GL context, input, and audio
//! on both native and web; this module just wires the game's update/draw into
//! its per-frame loop.

use macroquad::prelude::{get_frame_time, next_frame, Conf};

use crate::audio::{Audio, Track};
use crate::data::{BATTLE_MUSIC_OGG, BOSS_MUSIC_OGG, FONT_TTF};
use crate::game::Game;
use crate::input::Controllers;
use crate::renderer::{color, Renderer};

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

    // Preload the looping music tracks (async) before entering the loop.
    let mut audio = Audio::new();
    audio.load_music(Track::Battle, BATTLE_MUSIC_OGG).await;
    audio.load_music(Track::Boss, BOSS_MUSIC_OGG).await;

    let mut renderer = Renderer::new(FONT_TTF);
    let mut game = Game::new(&mut renderer, audio);
    let mut controllers = Controllers::new();

    loop {
        // Cap dt so a stall (or a paused tab) can't teleport the simulation.
        let dt = get_frame_time().min(0.05);

        // Poll before update, so the touch overlay matches the scene the player
        // is currently looking at (the joystick while walking, up/down in battle).
        controllers.poll(game.touch_scheme());
        game.update(&controllers, &mut renderer, dt);
        game.draw(&mut renderer);
        draw_touch_controls(&controllers, &mut renderer);
        renderer.render();

        next_frame().await;
    }
}

/// Paint the on-screen touch controls over everything (only present on a
/// touchscreen session; see [`Controllers::touch_overlay`]). Each button is a
/// translucent rounded pad with its glyph, brightened while held so a thumb gets
/// visible feedback; the overworld's [joystick](Controllers::touch_stick) is a
/// base ring with a thumb knob that follows the finger. Drawn in raw window
/// pixels so the controls sit in the letterbox margins around the game.
fn draw_touch_controls(controllers: &Controllers, renderer: &mut Renderer) {
    let held_col = color::rgba(255, 245, 200, 210);
    let idle_col = color::rgba(235, 240, 255, 120);
    let border = color::rgba(10, 12, 20, 180);
    for tb in controllers.touch_overlay() {
        let held = controllers.shared().held(tb.button);
        let fill = if held {
            color::rgba(120, 130, 170, 190)
        } else {
            color::rgba(40, 46, 66, 150)
        };
        renderer.draw_overlay_rect(tb.x, tb.y, tb.w, tb.h, fill);
        renderer.draw_overlay_rect_outline(tb.x, tb.y, tb.w, tb.h, 2.0, border);
        renderer.draw_overlay_text_centered(
            tb.label,
            tb.x + tb.w / 2.0,
            tb.y + tb.h / 2.0,
            tb.h * 0.5,
            if held { held_col } else { idle_col },
        );
    }

    // The overworld joystick: a translucent base ring with a brighter thumb knob
    // that tracks the finger (and lights up while deflected).
    if let Some(s) = controllers.touch_stick() {
        renderer.draw_overlay_circle(s.cx, s.cy, s.radius, color::rgba(40, 46, 66, 130));
        renderer.draw_overlay_circle_outline(s.cx, s.cy, s.radius, 2.0, border);
        let knob = if s.active {
            color::rgba(120, 130, 170, 210)
        } else {
            color::rgba(90, 100, 140, 170)
        };
        renderer.draw_overlay_circle(s.knob_x, s.knob_y, s.radius * 0.5, knob);
        renderer.draw_overlay_circle_outline(s.knob_x, s.knob_y, s.radius * 0.5, 2.0, border);
    }
}
