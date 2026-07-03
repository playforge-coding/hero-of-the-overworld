//! Shared helpers for the rustautogui end-to-end suite.
//!
//! These tests drive the *real* game window with rustautogui (keyboard +
//! screenshots + template matching), so they need a display and they take over
//! the screen/mouse briefly. They are therefore marked `#[ignore]` and run
//! explicitly:
//!
//! ```text
//! cargo test --test e2e -- --ignored --test-threads=1
//! ```
//!
//! The game is launched with `HOTO_TEST_WINDOW=1`, which makes it run
//! borderless-fullscreen so the 320x180 canvas maps at a clean integer scale
//! and always owns focus.

#![allow(dead_code)]

use std::process::{Child, Command};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

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
    pub fn launch() -> Self {
        let child = Command::new(GAME_BIN)
            .env("HOTO_TEST_WINDOW", "1")
            .env("RUSTAUTOGUI_SUPPRESS_WARNINGS", "1")
            .spawn()
            .expect("failed to spawn game binary");
        let mut game = Game { child };
        // Give winit + wgpu time to create the surface and present a frame.
        sleep_ms(2500);
        assert!(
            game.is_running(),
            "game exited during startup — check panic output"
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

/// Create a rustautogui handle. The game claims keyboard focus itself on
/// startup (via `focus_window()` in test mode). We must NOT touch the mouse:
/// both clicking and moving the cursor steal focus away from the borderless
/// fullscreen window, after which key events go elsewhere.
pub fn autogui() -> RustAutoGui {
    let mut gui = RustAutoGui::new(false).expect("init rustautogui");
    gui.set_suppress_warnings(true);
    gui
}

pub fn press(gui: &RustAutoGui, key: &str) {
    gui.keyboard_command(key).expect("send key");
    sleep_ms(120);
}

/// Hold a key down for `ms` milliseconds, then release — used to walk the
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

/// Fraction of sampled pixels whose colour differs noticeably between two
/// same-size screenshots (used to prove a scene transition happened).
pub fn changed_fraction(a: &RgbaImage, b: &RgbaImage) -> f64 {
    let (w, h) = (a.width().min(b.width()), a.height().min(b.height()));
    let mut changed = 0.0f64;
    let mut total = 0.0f64;
    let mut y = 0;
    while y < h {
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
