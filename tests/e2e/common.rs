//! Shared helpers for the rustautogui end-to-end suite.
//!
//! These tests drive the *real* game window with rustautogui (keyboard input,
//! screenshots, and template matching), so they need a display and they take
//! over the screen briefly. They are therefore marked `#[ignore]` and run
//! explicitly on a virtual 960x540 X display:
//!
//! ```text
//! xvfb-run -a -s "-screen 0 960x540x24" cargo test --test e2e -- --ignored --test-threads=1
//! ```
//!
//! The game is launched with `HOTO_TEST_WINDOW=1`, which asks for a fullscreen
//! window. Xvfb has no window manager to honor that, so the window keeps its
//! default 960x540 size; a screen of exactly that size means the window covers
//! the whole display and receives every key press. The sprite templates in
//! `tests/fixtures/` were captured at that size, so template matching only works
//! on a 960x540 screen. The luminance and changed-fraction checks don't depend
//! on scale.

#![allow(dead_code)]

use std::process::{Child, Command};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use image::RgbaImage;
use rustautogui::{MatchMode, RustAutoGui};

/// Path to the built game binary (Cargo sets this for integration tests).
pub const GAME_BIN: &str = env!("CARGO_BIN_EXE_hero");

/// Only one GUI test may touch the screen at a time.
static GUI_LOCK: Mutex<()> = Mutex::new(());

pub fn gui_guard() -> MutexGuard<'static, ()> {
    GUI_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// GUI tests need a display; allow opting out via env for headless CI.
pub fn gui_available() -> bool {
    std::env::var("DISPLAY")
        .map(|d| !d.is_empty())
        .unwrap_or(false)
        && std::env::var("HOTO_SKIP_GUI_TESTS").is_err()
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// A launched game process that is killed when it goes out of scope.
pub struct Game {
    child: Child,
}

impl Game {
    /// Launch the game in fullscreen test mode and wait for it to come up.
    ///
    /// Each launch gets its own empty save folder through `HOTO_SAVE_DIR`, which
    /// the game honors on every OS. The game autosaves, so without this one
    /// test's progress would land in slot 1 and the next test's "new game" press
    /// would load that save instead. It also keeps the tests away from the
    /// player's real saves.
    pub fn launch() -> Self {
        let save_dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("e2e-saves");
        let _ = std::fs::remove_dir_all(&save_dir);
        std::fs::create_dir_all(&save_dir).expect("create e2e save dir");
        let child = Command::new(GAME_BIN)
            .env("HOTO_SAVE_DIR", &save_dir)
            .env("HOTO_TEST_WINDOW", "1")
            .env("RUSTAUTOGUI_SUPPRESS_WARNINGS", "1")
            .spawn()
            .expect("failed to spawn game binary");
        let mut game = Game { child };
        // Give macroquad time to open the window and present a frame.
        sleep_ms(2500);
        assert!(
            game.is_running(),
            "game exited during startup, check the panic output"
        );
        game
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Create a rustautogui handle. We must NOT touch the mouse: on a display
/// without a window manager, keys go to whatever window is under the pointer,
/// and clicking or moving the cursor can steal focus from the game window, after
/// which key events go elsewhere.
pub fn autogui() -> RustAutoGui {
    let mut gui = RustAutoGui::new(false).expect("init rustautogui");
    gui.set_suppress_warnings(true);
    gui
}

pub fn press(gui: &RustAutoGui, key: &str) {
    gui.keyboard_command(key).expect("send key");
    sleep_ms(120);
}

/// Whether a cutscene dialogue box is on screen. The box is a near-opaque dark
/// navy panel across the lower third (see `Cutscene::draw`), so this checks
/// that most of that band is that color. Text, the portrait, and the border
/// are a small share of the band, and a level's own art there is never this
/// uniformly dark blue.
pub fn dialogue_open(img: &RgbaImage) -> bool {
    let (w, h) = (img.width(), img.height());
    let (x0, x1) = (w * 3 / 100, w * 97 / 100);
    let (y0, y1) = (h * 66 / 100, h * 94 / 100);
    let (mut navy, mut total) = (0u32, 0u32);
    for y in (y0..y1).step_by(4) {
        for x in (x0..x1).step_by(4) {
            let p = img.get_pixel(x, y);
            let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
            if r < 45 && g < 50 && b < 70 && b > r + 8 {
                navy += 1;
            }
            total += 1;
        }
    }
    navy * 2 > total
}

/// How long the screen must go without a dialogue box before the intro counts
/// as over. Longer than the slowest action beat (an actor walking a few tiles),
/// so the helper doesn't stop between two lines.
const CUTSCENE_QUIET_MS: u128 = 3000;

/// Play through a level's intro cutscene. Enter is pressed only while a
/// dialogue box is up (the first press reveals the whole line, the next one
/// advances), and action beats are left to play out on their own. The intro is
/// over once no box has shown for [`CUTSCENE_QUIET_MS`], so the helper never
/// sends stray presses into the level and works for an intro of any length.
/// Fails the test if the cutscene is still going after 60 seconds.
pub fn skip_cutscene(gui: &mut RustAutoGui) {
    let start = Instant::now();
    let mut last_box = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(60),
            "the intro cutscene never finished"
        );
        if dialogue_open(&screenshot(gui, "cutscene_probe")) {
            press(gui, "return");
            last_box = Instant::now();
        } else if last_box.elapsed().as_millis() > CUTSCENE_QUIET_MS {
            return;
        }
        sleep_ms(150);
    }
}

/// Hold a key down for `ms` milliseconds, then release. Used to walk the
/// overworld player a meaningful distance (a single tap barely moves them).
pub fn hold(gui: &RustAutoGui, key: &str, ms: u64) {
    gui.key_down(key).expect("key down");
    sleep_ms(ms);
    gui.key_up(key).expect("key up");
    sleep_ms(120);
}

/// Grab the whole screen to a PNG and load it back for analysis.
pub fn screenshot(gui: &mut RustAutoGui, name: &str) -> RgbaImage {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("{name}.png"));
    let path_str = path.to_str().unwrap();
    gui.save_screenshot(path_str).expect("save screenshot");
    image::open(path_str).expect("load screenshot").to_rgba8()
}

/// Standard deviation of luminance across the image (0 for a blank screen).
pub fn luminance_stddev(img: &RgbaImage) -> f64 {
    let mut sum = 0.0f64;
    let mut sum_sq = 0.0f64;
    let mut n = 0.0f64;
    for p in img.pixels().step_by(37) {
        let l = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
        sum += l;
        sum_sq += l * l;
        n += 1.0;
    }
    let mean = sum / n;
    (sum_sq / n - mean * mean).max(0.0).sqrt()
}

/// Fraction of sampled pixels whose color differs noticeably between two
/// same-size screenshots (used to prove a scene transition happened).
pub fn changed_fraction(a: &RgbaImage, b: &RgbaImage) -> f64 {
    changed_fraction_in(a, b, 0.0, 1.0)
}

/// Like [`changed_fraction`], but only over the horizontal band from `top` to
/// `bottom`, given as fractions of the screen height. Used where the change to
/// look for is small and sits in a known strip of the screen.
pub fn changed_fraction_in(a: &RgbaImage, b: &RgbaImage, top: f64, bottom: f64) -> f64 {
    let (w, h) = (a.width().min(b.width()), a.height().min(b.height()));
    let y_end = (h as f64 * bottom) as u32;
    let mut changed = 0.0f64;
    let mut total = 0.0f64;
    let mut y = (h as f64 * top) as u32;
    while y < y_end {
        let mut x = 0;
        while x < w {
            let pa = a.get_pixel(x, y);
            let pb = b.get_pixel(x, y);
            let d = (pa[0] as i32 - pb[0] as i32).abs()
                + (pa[1] as i32 - pb[1] as i32).abs()
                + (pa[2] as i32 - pb[2] as i32).abs();
            if d > 40 {
                changed += 1.0;
            }
            total += 1.0;
            x += 3;
        }
        y += 3;
    }
    changed / total
}

/// Prepare a template PNG and return the best correlation found on screen,
/// retrying for up to `timeout_secs` to ride out sprite animation frames.
pub fn best_match(
    gui: &mut RustAutoGui,
    template: &str,
    precision: f32,
    timeout_secs: u64,
) -> Option<f32> {
    let path = format!("{}/tests/fixtures/{template}", env!("CARGO_MANIFEST_DIR"));
    gui.prepare_template_from_file(&path, None, MatchMode::Segmented)
        .expect("prepare template");
    let result = match gui.loop_find_image_on_screen(precision, timeout_secs) {
        Ok(Some(matches)) => matches
            .iter()
            .map(|m| m.2)
            .fold(None, |best, c| Some(best.map_or(c, |b: f32| b.max(c)))),
        _ => None,
    };
    eprintln!("[e2e] template {template}: best correlation = {result:?}");
    result
}
