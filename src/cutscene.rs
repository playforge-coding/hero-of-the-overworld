//! Cutscene playback: a scripted sequence of [`CutsceneStep`]s that plays out
//! **on the live overworld map**.
//!
//! Cutscenes are pure data (one `assets/data/cutscenes/<id>.ron` each), so adding
//! story beats — including new party members joining — is a data edit. A `Say`
//! step shows a dialogue box with an optional portrait and a typewriter reveal; a
//! `Recruit` step adds a character to the party.
//!
//! The rest of the steps **choreograph the map underneath the dialogue**. While a
//! cutscene runs, the [`Overworld`] it was launched from stays on screen but stops
//! taking input: the cutscene drives it instead ([`Overworld::cutscene_update`]),
//! placing cast actors ([`Place`](CutsceneStep::Place)), walking them across the
//! field ([`Walk`](CutsceneStep::Walk)), turning them ([`Turn`](CutsceneStep::Turn)),
//! panning the camera ([`Pan`](CutsceneStep::Pan)), and holding beats
//! ([`Wait`](CutsceneStep::Wait)). Interleaving those with `Say` lines is what makes
//! a scene *choreographed* — the movement is timed against the words that narrate
//! it. Instant steps fire the moment they're reached; `Say`/`Walk`/`Wait` hold the
//! scene until the player dismisses the line, the actor arrives, or the beat ends
//! (a Confirm/Cancel press skips ahead in every case).

use std::collections::HashMap;

use crate::data::{CutsceneStep, Registry};
use crate::input::{Button, Input};
use crate::overworld::{walk_sprite_for, CastSprite, Overworld};
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
    /// Overworld walk sprites for every actor a `Place` step brings on, resolved
    /// up front (keyed by character/enemy id) so placing one mid-scene needs no
    /// renderer — mirroring how `portraits` are pre-resolved.
    cast_sprites: HashMap<String, CastSprite>,
    idx: usize,
    /// Characters currently revealed of the active line.
    reveal: f32,
    /// Elapsed time in the active timed step (a `Wait`'s countdown).
    step_time: f32,
    /// Whether the active timed step has been kicked off (a `Walk`'s order issued).
    /// Reset on every advance so each step begins exactly once.
    started: bool,
    time: f32,
}

impl Cutscene {
    /// Build a runtime for `steps`, resolving portrait and cast-actor textures up
    /// front so update/draw need no mutable access to the renderer/cache.
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
        // Resolve one walk sprite per distinct character named by a `Place` step.
        let mut cast_sprites = HashMap::new();
        for step in &steps {
            if let CutsceneStep::Place { character, .. } = step {
                if cast_sprites.contains_key(character) {
                    continue;
                }
                if let Some((walk, tint)) = walk_sprite_for(reg, character) {
                    let tex = cache.get(renderer, &walk.texture);
                    cast_sprites.insert(character.clone(), CastSprite { walk, tex, tint });
                } else {
                    log::warn!("cutscene Place references unknown character '{character}'");
                }
            }
        }
        Cutscene {
            steps,
            portraits,
            cast_sprites,
            idx: 0,
            reveal: 0.0,
            step_time: 0.0,
            started: false,
            time: 0.0,
        }
    }

    /// Is the active step an interactive line the player must dismiss?
    fn on_say(&self) -> bool {
        matches!(self.steps.get(self.idx), Some(CutsceneStep::Say { .. }))
    }

    /// Move the play-head to the next step, clearing the per-step timers so the
    /// next `Say`/`Walk`/`Wait` starts fresh.
    fn advance(&mut self) {
        self.idx += 1;
        self.reveal = 0.0;
        self.step_time = 0.0;
        self.started = false;
    }

    /// Execute every **instant** step at the play-head — party recruits and stage
    /// direction (place / turn / leave / pan) — stopping at the first dialogue or
    /// timed step (`Say`/`Walk`/`Wait`) or the end of the script.
    fn run_instant(
        &mut self,
        party: &mut Party,
        reg: &Registry,
        mut level: Option<&mut Overworld>,
    ) {
        while let Some(step) = self.steps.get(self.idx) {
            match step {
                CutsceneStep::Say { .. }
                | CutsceneStep::Walk { .. }
                | CutsceneStep::Wait { .. } => break,
                CutsceneStep::Recruit { character } => {
                    if !party.members.iter().any(|m| m.def_id == *character) {
                        party.recruit(reg, character);
                    }
                }
                CutsceneStep::Place {
                    actor,
                    character,
                    at,
                    facing,
                } => {
                    if let (Some(ov), Some(spr)) =
                        (level.as_deref_mut(), self.cast_sprites.get(character))
                    {
                        ov.cast_place(actor, spr, at.0, at.1, *facing);
                    }
                }
                CutsceneStep::Turn { actor, facing } => {
                    if let Some(ov) = level.as_deref_mut() {
                        ov.cast_turn(actor, *facing);
                    }
                }
                CutsceneStep::Leave { actor } => {
                    if let Some(ov) = level.as_deref_mut() {
                        ov.cast_leave(actor);
                    }
                }
                CutsceneStep::Pan { at } => {
                    if let Some(ov) = level.as_deref_mut() {
                        ov.cam_focus_tile(at.0, at.1);
                    }
                }
            }
            // Direct field bumps (not `advance()`) so the immutable borrow of
            // `self.steps` held by `step` stays valid through the loop.
            self.idx += 1;
            self.reveal = 0.0;
            self.step_time = 0.0;
            self.started = false;
        }
    }

    pub fn update(
        &mut self,
        input: &Input,
        party: &mut Party,
        reg: &Registry,
        dt: f32,
        mut level: Option<&mut Overworld>,
    ) -> Option<CutsceneOutcome> {
        self.time += dt;
        // Advance the choreographed motion (cast walking, camera pan) each frame.
        if let Some(ov) = level.as_deref_mut() {
            ov.cutscene_update(dt);
        }

        self.run_instant(party, reg, level.as_deref_mut());
        if self.idx >= self.steps.len() {
            return Some(CutsceneOutcome::Finished);
        }

        // Kick off the active timed step once: a `Walk` issues its order here so
        // the actor is moving by the next frame.
        if !self.started {
            self.started = true;
            if let Some(CutsceneStep::Walk { actor, to, speed }) = self.steps.get(self.idx) {
                if let Some(ov) = level.as_deref_mut() {
                    ov.cast_walk_to(actor, to.0, to.1, *speed);
                }
            }
        }

        let skip = input.pressed(Button::Confirm) || input.pressed(Button::Cancel);
        let mut advanced = false;
        match self.steps.get(self.idx) {
            Some(CutsceneStep::Say { text, .. }) => {
                self.reveal += dt * REVEAL_CPS;
                let full = text.chars().count() as f32;
                if skip {
                    if self.reveal < full {
                        self.reveal = full; // first press: reveal the whole line
                    } else {
                        advanced = true;
                    }
                }
            }
            Some(CutsceneStep::Walk { actor, .. }) => {
                let arrived = level
                    .as_deref()
                    .map(|o| o.cast_arrived(actor))
                    .unwrap_or(true);
                if skip && !arrived {
                    if let Some(ov) = level.as_deref_mut() {
                        ov.cast_snap(actor);
                    }
                    advanced = true;
                } else if arrived {
                    advanced = true;
                }
            }
            Some(CutsceneStep::Wait { secs }) => {
                self.step_time += dt;
                if self.step_time >= *secs || skip {
                    advanced = true;
                }
            }
            _ => {}
        }

        if advanced {
            self.advance();
            self.run_instant(party, reg, level);
            if self.idx >= self.steps.len() {
                return Some(CutsceneOutcome::Finished);
            }
        }
        None
    }

    /// Draw the scene: the live map as a backdrop (or a plain void if the cutscene
    /// is playing with no level attached), then the dialogue box for a `Say` line.
    pub fn draw(&self, r: &mut Renderer, level: Option<&Overworld>) {
        match level {
            Some(ov) => ov.draw_world(r),
            None => {
                r.set_clear_color(color::rgb(6, 6, 12));
                r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(10, 10, 18));
            }
        }

        if !self.on_say() {
            return; // an action beat: pure choreography, no dialogue box.
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
