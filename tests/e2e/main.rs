//! End-to-end tests that drive the real game window with rustautogui.
//!
//! Run explicitly on a virtual 960x540 display (see `common.rs` for why):
//!
//! ```text
//! xvfb-run -a -s "-screen 0 960x540x24" cargo test --test e2e -- --ignored --test-threads=1
//! ```

mod common;
use common::*;

/// The game boots (macroquad window, embedded RON data, and textures) and keeps
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

/// Pressing Enter moves from the title to the map screen, then into the first
/// level's intro cutscene, then onto the tiled world. Each transition visibly
/// changes the screen, proving the scene pipeline works.
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
    // A second title shot with no input in between measures how much the idle
    // animation alone changes the screen, so the map check can't pass on noise.
    sleep_ms(500);
    let title_idle = screenshot(&mut gui, "title_idle");
    let idle = changed_fraction(&title, &title_idle);
    press(&gui, "return"); // leave the title for the save-select menu
    sleep_ms(300);
    press(&gui, "return"); // empty slot 1 starts a new game on the map
    sleep_ms(500);
    let map = screenshot(&mut gui, "map");
    // The title and the world map share a dark backdrop, so only the text,
    // sprites, and map nodes register as changed (about 13% of the screen).
    let opened = changed_fraction(&title, &map);
    assert!(
        opened > idle + 0.08,
        "screen barely changed after Enter ({opened:.3} vs {idle:.3} from idle \
         animation), so the map didn't open"
    );

    press(&gui, "return"); // enter the first level's intro cutscene
    sleep_ms(700);
    let cutscene = screenshot(&mut gui, "cutscene");
    assert!(
        changed_fraction(&map, &cutscene) > 0.15,
        "screen barely changed after Enter, so the cutscene didn't play"
    );

    // Play through the intro (three lines plus a slime crossing the trail).
    skip_cutscene(&mut gui);
    let level = screenshot(&mut gui, "level");
    assert!(
        luminance_stddev(&level) > 12.0,
        "level looks blank (luminance stddev too low), so tiles didn't render"
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

    press(&gui, "return"); // leave the title for the save-select menu
    sleep_ms(300);
    press(&gui, "return"); // empty slot 1 starts a new game on the map
    sleep_ms(400);
    press(&gui, "return"); // enter the first level's intro cutscene
    sleep_ms(700);
    skip_cutscene(&mut gui);

    // Walk east into the slime swarm patrolling the starting corridor.
    hold(&gui, "right", 2600);
    sleep_ms(1600); // battle intro + banner
    // Kept on disk so a failed template match can be inspected afterward.
    screenshot(&mut gui, "battle");

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

    // Go from the title through save-select and the map, then play the intro
    // cutscene into the level (GREENWOOD, screen 0).
    press(&gui, "return"); // leave the title for the save-select menu
    sleep_ms(300);
    press(&gui, "return"); // empty slot 1 starts a new game on the map
    sleep_ms(500);
    press(&gui, "return"); // enter the first level's intro cutscene
    sleep_ms(700);
    skip_cutscene(&mut gui);
    let level = screenshot(&mut gui, "shop_level");

    // The keeper stall is at tile (5,6); the player starts at (2,5). Walk right
    // and down to reach it, then confirm to step inside the shop.
    hold(&gui, "right", 700);
    hold(&gui, "down", 250);
    screenshot(&mut gui, "shop_walked");
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

    // The player comes in at the door, so walk up to the counter (the keeper's
    // desk stops them) and open the buy menu there.
    hold(&gui, "w", 2500);
    press(&gui, "z");
    sleep_ms(400);
    let menu = screenshot(&mut gui, "shop_menu");
    assert!(
        changed_fraction(&inside, &menu) > 0.15,
        "buy menu didn't open at the counter"
    );

    // Try to buy the highlighted item. A new game starts with no gold, so the
    // shop answers with a one-line "NOT ENOUGH GOLD" notice in the panel near
    // the bottom of the screen. Only that band is compared, against a control
    // shot taken with no input, so idle animation elsewhere can't pass the check.
    const NOTICE_BAND: (f64, f64) = (0.84, 0.94);
    sleep_ms(400);
    let menu_idle = screenshot(&mut gui, "shop_menu_idle");
    let idle = changed_fraction_in(&menu, &menu_idle, NOTICE_BAND.0, NOTICE_BAND.1);
    press(&gui, "z");
    sleep_ms(400);
    let bought = screenshot(&mut gui, "shop_bought");
    let notice = changed_fraction_in(&menu_idle, &bought, NOTICE_BAND.0, NOTICE_BAND.1);
    assert!(
        notice > idle + 0.01,
        "buying showed no feedback ({notice:.3} of the notice band changed, \
         {idle:.3} with no input)"
    );
}
