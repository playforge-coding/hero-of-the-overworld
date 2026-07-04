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

/// Walking the overworld player into the first roaming foe (a slime swarm now
/// guards the GREENWOOD corridor) starts a battle, and the hero and slime sprites
/// are drawn (verified via template matching).
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn walking_into_slime_starts_battle() {
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

    // Walk east into the slime swarm patrolling the starting corridor.
    hold(&gui, "right", 2600);
    sleep_ms(1600); // battle intro + banner

    let hero = best_match(&mut gui, "hero_template.png", 0.7, 6);
    assert!(
        hero.is_some_and(|c| c >= 0.7),
        "hero sprite not found on screen (best correlation {hero:?})"
    );
    let slime = best_match(&mut gui, "slime_template.png", 0.55, 6);
    assert!(
        slime.is_some_and(|c| c >= 0.55),
        "slime sprite not found on screen (best correlation {slime:?})"
    );
}

/// Enter the GREENWOOD, walk to the OUTFITTER's keeper, step inside, open the
/// buy menu, and buy an item. Each step visibly changes the screen, proving the
/// shop scene, its UI, and the purchase flow all work end to end.
#[test]
#[ignore = "GUI e2e: needs a display, drives the real window"]
fn enter_shop_and_buy() {
    if !gui_available() {
        eprintln!("skipping: no DISPLAY");
        return;
    }
    let _lock = gui_guard();
    let _game = Game::launch();
    let mut gui = autogui();

    // Title -> map -> intro cutscene -> level (GREENWOOD, screen 0).
    press(&gui, "return");
    sleep_ms(500);
    press(&gui, "return");
    sleep_ms(700);
    press(&gui, "return");
    sleep_ms(300);
    press(&gui, "return");
    sleep_ms(700);
    let level = screenshot(&mut gui, "shop_level");

    // The keeper stall is at tile (5,6); the player starts at (2,5). Walk right
    // and down to reach it, then confirm to step inside the shop.
    hold(&gui, "d", 700);
    hold(&gui, "s", 250);
    press(&gui, "z"); // enter the shop
    sleep_ms(600);
    let inside = screenshot(&mut gui, "shop_inside");
    assert!(
        changed_fraction(&level, &inside) > 0.2,
        "screen didn't change entering the shop"
    );
    assert!(
        luminance_stddev(&inside) > 12.0,
        "shop interior looks blank"
    );

    // Open the buy menu at the counter.
    press(&gui, "z");
    sleep_ms(400);
    let menu = screenshot(&mut gui, "shop_menu");
    assert!(
        changed_fraction(&inside, &menu) > 0.15,
        "buy menu didn't open at the counter"
    );

    // Buy the highlighted item; the gold readout / feedback line changes.
    press(&gui, "z");
    sleep_ms(400);
    let bought = screenshot(&mut gui, "shop_bought");
    assert!(
        changed_fraction(&menu, &bought) > 0.02,
        "buying didn't change anything (no feedback / gold update)"
    );
}
