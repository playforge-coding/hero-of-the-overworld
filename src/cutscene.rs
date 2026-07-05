//! Cutscene playback: a scripted sequence of [`CutsceneStep`]s.
//!
//! Cutscenes are pure data (see `game.ron`), so adding story beats — including
//! new party members joining — is a data edit. A `Say` step shows a dialogue
//! box with an optional portrait and a typewriter reveal; a `Recruit` step adds
//! a character to the party. Non-visual steps (like `Recruit`) execute the
//! moment they are reached and the scene advances to the next line.

use crate::data::{CutsceneStep, Registry};
use crate::input::{Button, Input};
use crate::party::Party;
use crate::renderer::{color, virtual_w, Color, Renderer, TextureHandle, VIRTUAL_H};
use crate::util::TextureCache;

/// Characters revealed per second by the typewriter effect.
const REVEAL_CPS: f32 = 42.0;
/// Dialogue text scale, and the resulting characters-per-line for wrapping.
const TEXT_SCALE: f32 = 1.0;

/// A resolved portrait: which sprite frame to draw beside a line.
struct Portrait {
    tex: TextureHandle,
    src: [f32; 4],
    tint: Color,
}

pub enum CutsceneOutcome {
    Finished,
}

pub struct Cutscene {
    steps: Vec<CutsceneStep>,
    /// Portrait for each step (only `Say` steps have one); parallel to `steps`.
    portraits: Vec<Option<Portrait>>,
    idx: usize,
    /// Characters currently revealed of the active line.
    reveal: f32,
    time: f32,
}

impl Cutscene {
    /// Build a runtime for `steps`, resolving portrait textures up front so
    /// drawing needs no mutable access to the renderer/cache.
    pub fn new(
        renderer: &mut Renderer,
        cache: &mut TextureCache,
        reg: &Registry,
        steps: Vec<CutsceneStep>,
    ) -> Self {
        let portraits = steps
            .iter()
            .map(|s| match s {
                CutsceneStep::Say {
                    portrait: Some(id), ..
                } => resolve_portrait(renderer, cache, reg, id),
                _ => None,
            })
            .collect();
        // Instant steps (e.g. a leading `Recruit`) are applied on the first
        // update(), where the party is available.
        Cutscene {
            steps,
            portraits,
            idx: 0,
            reveal: 0.0,
            time: 0.0,
        }
    }

    /// Is the active step an interactive line the player must dismiss?
    fn on_say(&self) -> bool {
        matches!(self.steps.get(self.idx), Some(CutsceneStep::Say { .. }))
    }

    fn active_text(&self) -> Option<&str> {
        match self.steps.get(self.idx) {
            Some(CutsceneStep::Say { text, .. }) => Some(text),
            _ => None,
        }
    }

    /// Execute non-visual steps (e.g. `Recruit`) until the active step is a line
    /// of dialogue or the script ends.
    fn apply_instant_steps(&mut self, party: &mut Party, reg: &Registry) {
        while let Some(step) = self.steps.get(self.idx) {
            match step {
                CutsceneStep::Say { .. } => break,
                CutsceneStep::Recruit { character } => {
                    let already = party.members.iter().any(|m| m.def_id == *character);
                    if !already {
                        party.recruit(reg, character);
                    }
                    self.idx += 1;
                }
            }
        }
    }

    pub fn update(
        &mut self,
        input: &Input,
        party: &mut Party,
        reg: &Registry,
        dt: f32,
    ) -> Option<CutsceneOutcome> {
        self.time += dt;
        self.apply_instant_steps(party, reg);
        if self.idx >= self.steps.len() {
            return Some(CutsceneOutcome::Finished);
        }

        // Active step is a Say line: reveal it, then advance on confirm/cancel.
        self.reveal += dt * REVEAL_CPS;
        let full = self.active_text().map(|t| t.chars().count()).unwrap_or(0) as f32;
        if input.pressed(Button::Confirm) || input.pressed(Button::Cancel) {
            if self.reveal < full {
                self.reveal = full; // first press: reveal the whole line
            } else {
                self.idx += 1;
                self.reveal = 0.0;
                self.apply_instant_steps(party, reg);
                if self.idx >= self.steps.len() {
                    return Some(CutsceneOutcome::Finished);
                }
            }
        }
        None
    }

    pub fn draw(&self, r: &mut Renderer) {
        r.set_clear_color(color::rgb(6, 6, 12));
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(10, 10, 18));

        if !self.on_say() {
            return;
        }
        let (speaker, text) = match self.steps.get(self.idx) {
            Some(CutsceneStep::Say { speaker, text, .. }) => (speaker.as_deref(), text.as_str()),
            _ => return,
        };
        let portrait = self.portraits.get(self.idx).and_then(|p| p.as_ref());

        // Dialogue panel across the lower third.
        let box_x = 8.0;
        let box_y = VIRTUAL_H - 66.0;
        let box_w = virtual_w() - 16.0;
        let box_h = 58.0;
        r.draw_rect(box_x, box_y, box_w, box_h, color::rgba(12, 14, 30, 236));
        r.draw_rect_outline(
            box_x,
            box_y,
            box_w,
            box_h,
            1.0,
            color::rgba(90, 110, 170, 255),
        );

        // Portrait on the left, if any.
        let mut text_x = box_x + 8.0;
        if let Some(p) = portrait {
            let pw = 40.0;
            let py = box_y + 9.0;
            r.draw_rect(box_x + 6.0, py, pw, pw, color::rgba(20, 24, 44, 255));
            r.draw_sprite(p.tex, [box_x + 6.0, py, pw, pw], p.src, false, p.tint);
            r.draw_rect_outline(
                box_x + 6.0,
                py,
                pw,
                pw,
                1.0,
                color::rgba(120, 140, 200, 255),
            );
            text_x = box_x + 6.0 + pw + 8.0;
        }

        // Speaker name.
        let mut text_y = box_y + 6.0;
        if let Some(name) = speaker {
            r.draw_text(name, text_x, text_y, 1.0, color::rgb(255, 226, 120));
            text_y += 12.0;
        }

        // Word-wrapped, progressively revealed body text.
        let max_w = box_x + box_w - 6.0 - text_x;
        let per_line = ((max_w / (5.0 * TEXT_SCALE)).floor() as usize).max(1);
        let lines = wrap(text, per_line);
        let mut shown = self.reveal as usize;
        for line in &lines {
            let n = line.chars().count();
            let take = shown.min(n);
            let slice: String = line.chars().take(take).collect();
            r.draw_text(&slice, text_x, text_y, TEXT_SCALE, color::WHITE);
            text_y += 11.0;
            shown = shown.saturating_sub(n);
        }

        // Blinking advance prompt once the line is fully shown.
        let full = text.chars().count() as f32;
        if self.reveal >= full && (self.time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "ENTER",
                virtual_w() - 26.0,
                box_y + box_h - 10.0,
                1.0,
                color::rgb(180, 190, 220),
            );
        }
    }
}

/// Resolve a character/enemy id to a portrait (idle frame 0 of its sprite).
fn resolve_portrait(
    renderer: &mut Renderer,
    cache: &mut TextureCache,
    reg: &Registry,
    id: &str,
) -> Option<Portrait> {
    let sprite = reg
        .character(id)
        .map(|c| &c.sprite)
        .or_else(|| reg.enemy(id).map(|e| &e.sprite))?;
    let tex = cache.get(renderer, &sprite.texture);
    let clip = &sprite.idle;
    let src = [
        (clip.first_col * sprite.frame_w) as f32,
        (clip.row * sprite.frame_h) as f32,
        sprite.frame_w as f32,
        sprite.frame_h as f32,
    ];
    let tint = sprite
        .tint
        .map(|(r, g, b)| color::rgb(r, g, b))
        .unwrap_or(color::WHITE);
    Some(Portrait { tex, src, tint })
}

/// Greedy word wrap to at most `max` characters per line (the font is
/// fixed-width, so character count maps directly to pixel width).
fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= max {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
