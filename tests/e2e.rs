//! End-to-end tests that drive the real game window with rustautogui.
//!
//! Run explicitly (they need a display and take over the screen briefly):
//!
//! ```text
//! cargo test --test e2e -- --ignored --test-threads=1
//! ```

mod common;
use common::*;

/// The game boots (winit window + wgpu surface + RON data + textures) and keeps
/// running rather than panicking on startup.
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn boots_and_survives() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let mut game = Game::launch();
    sleep_ms(1500);
    assert!(game.is_running(), "game process died after startup");
}

/// The title screen actually renders content (not a blank/black frame).
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn title_screen_renders_content() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    let shot = screenshot(&mut gui, "title");
    let spread = luminance_stddev(&shot);
    assert!(
        spread > 12.0,
        "title screen looks blank (luminance stddev {spread:.1})"
    );
}

/// Enter → map screen → the first level's intro cutscene → the tiled world.
/// Each transition visibly changes the screen, proving the scene pipeline works.
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn enter_map_then_level_via_cutscene() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    let title = screenshot(&mut gui, "title");
    press(&gui, "return"); // title -> map
    sleep_ms(500);
    let map = screenshot(&mut gui, "map");
    assert!(
        changed_fraction(&title, &map) > 0.15,
        "screen barely changed after Enter — map didn't open"
    );

    press(&gui, "return"); // map -> intro cutscene
    sleep_ms(700);
    let cutscene = screenshot(&mut gui, "cutscene");
    assert!(
        changed_fraction(&map, &cutscene) > 0.15,
        "screen barely changed after Enter — cutscene didn't play"
    );

    // Dismiss the one-line intro (reveal, then advance) into the level.
    press(&gui, "return");
    sleep_ms(300);
    press(&gui, "return");
    sleep_ms(700);
    let level = screenshot(&mut gui, "level");
    assert!(
        luminance_stddev(&level) > 12.0,
        "level looks blank (luminance stddev too low) — tiles didn't render"
    );
    assert!(
        changed_fraction(&cutscene, &level) > 0.15,
        "screen didn't change from cutscene to level"
    );
}

/// Walking the overworld player into a roaming demon starts a battle, and the
/// hero and demon sprites are drawn (verified via template matching).
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn walking_into_demon_starts_battle() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    press(&gui, "return"); // title -> map
    sleep_ms(400);
    press(&gui, "return"); // map -> intro cutscene
    sleep_ms(700);
    // Dismiss the intro cutscene into the level.
    press(&gui, "return");
    sleep_ms(300);
    press(&gui, "return");
    sleep_ms(600);

    // Walk east into the demon patrolling the starting corridor.
    hold(&gui, "right", 2600);
    sleep_ms(1600); // battle intro + banner

    let hero = best_match(&mut gui, "hero_template.png", 0.7, 6);
    assert!(
        hero.map_or(false, |c| c >= 0.7),
        "hero sprite not found on screen (best correlation {hero:?})"
    );
    let demon = best_match(&mut gui, "demon_template.png", 0.55, 6);
    assert!(
        demon.map_or(false, |c| c >= 0.55),
        "demon sprite not found on screen (best correlation {demon:?})"
    );
}
