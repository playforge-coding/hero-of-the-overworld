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

/// Pressing Enter on the title transitions into the battle scene — proving the
/// winit→input→game pipeline works end to end.
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn enter_starts_battle() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    let before = screenshot(&mut gui, "before_enter");
    press(&gui, "return");
    sleep_ms(2000); // intro slide-in + banner

    let after = screenshot(&mut gui, "after_enter");
    let changed = changed_fraction(&before, &after);
    assert!(
        changed > 0.15,
        "screen barely changed after Enter ({:.1}% differ) — battle didn't start",
        changed * 100.0
    );
}

/// In battle, the hero and demon sprites are actually drawn on screen. Verified
/// with rustautogui template matching against the source-sheet frames.
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn battle_shows_hero_and_demon_sprites() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    // Enter the demon_solo battle.
    press(&gui, "return");
    sleep_ms(1800);

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

/// Battle menu navigation: moving the cursor and confirming an attack advances
/// the battle (the scene keeps changing as actions resolve).
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn attack_command_resolves() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    press(&gui, "return"); // start battle
    sleep_ms(1800);

    let pre = screenshot(&mut gui, "battle_menu");
    // ATTACK is the first command; confirm it, then target the first enemy.
    press(&gui, "return"); // choose ATTACK
    sleep_ms(200);
    press(&gui, "return"); // confirm target
    sleep_ms(1600); // action animation + damage popups

    let post = screenshot(&mut gui, "battle_action");
    let changed = changed_fraction(&pre, &post);
    assert!(
        changed > 0.02,
        "no visible change after attacking ({:.2}% differ)",
        changed * 100.0
    );
}
