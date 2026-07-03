//! Top-level game: owns persistent state (registry, party, RNG, audio) and the
//! current scene, and routes update/draw.
//!
//! Scene flow: the title leads to a **map screen** where you pick a level. Each
//! level is a set of connected [`Overworld`] screens you walk between; roaming
//! demons chase you and start a turn-based [`Battle`] on contact. Clearing every
//! demon in a level marks it done on the map.

use std::collections::HashSet;

use crate::audio::Audio;
use crate::battle::{Battle, BattleOutcome};
use crate::cutscene::{Cutscene, CutsceneOutcome};
use crate::data::{Registry, BATTLE_MUSIC_OGG};
use crate::input::{Button, Input};
use crate::overworld::{Event, Overworld, Trigger};
use crate::party::Party;
use crate::renderer::{color, Renderer, VIRTUAL_H, VIRTUAL_W};
use crate::util::{Rng, TextureCache};

enum Scene {
    Title,
    Map,
    Level,
    Cutscene(Cutscene),
    Battle(Battle),
    Report {
        win: bool,
        lines: Vec<String>,
        timer: f32,
    },
}

pub struct Game {
    reg: Registry,
    party: Party,
    cache: TextureCache,
    rng: Rng,
    audio: Audio,
    /// The active level runtime (present while in `Level`/`Battle`/`Report`).
    level: Option<Overworld>,
    /// Index into `reg.data.levels` of the active level.
    current_level: usize,
    /// Which levels the player has fully cleared (parallel to `reg.data.levels`).
    cleared: Vec<bool>,
    /// Selected level on the map screen.
    map_cursor: usize,
    /// The enemy that started the current battle, so the level can update on end.
    pending: Option<Trigger>,
    /// Cutscenes already played (by id), so intros/recruits fire only once.
    played_cutscenes: HashSet<String>,
    /// A cutscene queued to play after the current victory report (e.g. a
    /// character joining once a level is cleared).
    pending_cutscene: Option<Cutscene>,
    scene: Scene,
    time: f32,
}

impl Game {
    pub fn new(_renderer: &mut Renderer, audio: Audio) -> Self {
        let reg = Registry::load();
        let party = Party::from_registry(&reg);
        let cleared = vec![false; reg.data.levels.len()];
        Game {
            reg,
            party,
            cache: TextureCache::new(),
            rng: Rng::seeded_now(),
            audio,
            level: None,
            current_level: 0,
            cleared,
            map_cursor: 0,
            pending: None,
            played_cutscenes: HashSet::new(),
            pending_cutscene: None,
            scene: Scene::Title,
            time: 0.0,
        }
    }

    /// Build a cutscene by id if it exists and hasn't played yet (marking it
    /// played). Returns `None` for unknown or already-seen cutscenes.
    fn build_cutscene(&mut self, id: &str, renderer: &mut Renderer) -> Option<Cutscene> {
        if self.played_cutscenes.contains(id) {
            return None;
        }
        let steps = self.reg.cutscene(id)?.steps.clone();
        self.played_cutscenes.insert(id.to_string());
        Some(Cutscene::new(renderer, &mut self.cache, &self.reg, steps))
    }

    pub fn update(&mut self, input: &Input, renderer: &mut Renderer, dt: f32) {
        self.time += dt;
        // Take the scene out to sidestep borrow conflicts, put it back after.
        let scene = std::mem::replace(&mut self.scene, Scene::Title);
        self.scene = match scene {
            Scene::Title => {
                if input.pressed(Button::Confirm) {
                    Scene::Map
                } else {
                    Scene::Title
                }
            }
            Scene::Map => self.update_map(input, renderer),
            Scene::Level => self.update_level(input, renderer, dt),
            Scene::Cutscene(mut cs) => match cs.update(input, &mut self.party, &self.reg, dt) {
                Some(CutsceneOutcome::Finished) => {
                    if self.level.is_some() {
                        Scene::Level
                    } else {
                        Scene::Map
                    }
                }
                None => Scene::Cutscene(cs),
            },
            Scene::Battle(mut battle) => match battle.update(input, &mut self.rng, &self.reg, dt) {
                Some(outcome) => {
                    battle.sync_party(&mut self.party);
                    self.finish_battle(outcome, renderer)
                }
                None => Scene::Battle(battle),
            },
            Scene::Report {
                win,
                lines,
                mut timer,
            } => {
                timer -= dt;
                if timer <= 0.0 && input.any_pressed() {
                    // A queued recruit/story cutscene plays before returning.
                    match self.pending_cutscene.take() {
                        Some(cs) => Scene::Cutscene(cs),
                        None => Scene::Level,
                    }
                } else {
                    Scene::Report { win, lines, timer }
                }
            }
        };
    }

    fn update_map(&mut self, input: &Input, renderer: &mut Renderer) -> Scene {
        if input.pressed(Button::Cancel) {
            return Scene::Title;
        }
        for (dir, pressed) in [
            (Button::Left, input.pressed(Button::Left)),
            (Button::Right, input.pressed(Button::Right)),
            (Button::Up, input.pressed(Button::Up)),
            (Button::Down, input.pressed(Button::Down)),
        ] {
            if pressed {
                self.move_map_cursor(dir);
            }
        }
        if input.pressed(Button::Confirm) && !self.reg.data.levels.is_empty() {
            self.current_level = self.map_cursor;
            self.level = Some(Overworld::new(
                renderer,
                &mut self.cache,
                &self.reg,
                &self.party,
                self.current_level,
            ));
            // Play the level's intro cutscene the first time it's entered.
            if let Some(id) = self.reg.data.levels[self.current_level]
                .intro_cutscene
                .clone()
            {
                if let Some(cs) = self.build_cutscene(&id, renderer) {
                    return Scene::Cutscene(cs);
                }
            }
            return Scene::Level;
        }
        Scene::Map
    }

    fn update_level(&mut self, input: &Input, renderer: &mut Renderer, dt: f32) -> Scene {
        let Some(level) = &mut self.level else {
            return Scene::Map;
        };
        match level.update(input, dt) {
            None => Scene::Level,
            Some(Event::ExitToMap) => {
                self.cleared[self.current_level] |= level.all_cleared();
                self.level = None;
                Scene::Map
            }
            Some(Event::Battle(trigger)) => {
                let battle = Battle::new(
                    renderer,
                    &mut self.cache,
                    &self.reg,
                    &self.party,
                    &trigger.encounter,
                );
                self.audio.play_music_looping(BATTLE_MUSIC_OGG);
                self.pending = Some(trigger);
                Scene::Battle(battle)
            }
        }
    }

    /// Move the map selection to the nearest level marker in `dir`.
    fn move_map_cursor(&mut self, dir: Button) {
        let levels = &self.reg.data.levels;
        let Some(cur) = levels.get(self.map_cursor) else {
            return;
        };
        let (cx, cy) = (cur.node.0 as i32, cur.node.1 as i32);
        let mut best: Option<(i32, usize)> = None;
        for (i, lv) in levels.iter().enumerate() {
            if i == self.map_cursor {
                continue;
            }
            let dx = lv.node.0 as i32 - cx;
            let dy = lv.node.1 as i32 - cy;
            let aligned = match dir {
                Button::Left => dx < 0,
                Button::Right => dx > 0,
                Button::Up => dy < 0,
                Button::Down => dy > 0,
                _ => false,
            };
            if !aligned {
                continue;
            }
            // Prefer close nodes, penalise perpendicular drift.
            let (along, perp) = match dir {
                Button::Left | Button::Right => (dx.abs(), dy.abs()),
                _ => (dy.abs(), dx.abs()),
            };
            let score = along + perp * 3;
            if best.map_or(true, |(b, _)| score < b) {
                best = Some((score, i));
            }
        }
        if let Some((_, i)) = best {
            self.map_cursor = i;
        }
    }

    fn finish_battle(&mut self, outcome: BattleOutcome, renderer: &mut Renderer) -> Scene {
        self.audio.stop_music();
        let trigger = self.pending.take();
        let won = matches!(outcome, BattleOutcome::Victory { .. });
        let mut clear_cutscene = None;
        if let (Some(level), Some(t)) = (&mut self.level, &trigger) {
            level.resolve_battle(t, won);
            if won && level.all_cleared() {
                self.cleared[self.current_level] = true;
                clear_cutscene = self.reg.data.levels[self.current_level]
                    .clear_cutscene
                    .clone();
            }
        }
        // Queue the level-clear cutscene (e.g. a new ally joining) to play after
        // the victory report.
        if let Some(id) = clear_cutscene {
            self.pending_cutscene = self.build_cutscene(&id, renderer);
        }
        match outcome {
            BattleOutcome::Victory { xp, gold } => {
                self.party.gold += gold;
                let leveled = self.party.grant_xp(xp);
                let mut lines = vec![format!("GAINED {xp} XP  {gold} GOLD")];
                for i in &leveled {
                    if let Some(m) = self.party.members.get(*i) {
                        lines.push(format!("{} REACHED LV {}", m.name, m.level));
                    }
                }
                Scene::Report {
                    win: true,
                    lines,
                    timer: 0.8,
                }
            }
            BattleOutcome::Defeat => {
                // Wipe out: revive the party so exploration can continue.
                self.party.full_heal();
                Scene::Report {
                    win: false,
                    lines: vec!["THE PARTY IS REVIVED AT CAMP".to_string()],
                    timer: 0.8,
                }
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        match &mut self.scene {
            Scene::Title => Self::draw_title(&self.party, self.time, renderer),
            Scene::Map => self.draw_map(renderer),
            Scene::Level => {
                if let Some(level) = &self.level {
                    level.draw(renderer);
                }
            }
            Scene::Cutscene(cs) => cs.draw(renderer),
            Scene::Battle(battle) => battle.draw(renderer, &self.reg),
            Scene::Report { win, lines, .. } => Self::draw_report(*win, lines, renderer),
        }
    }

    fn draw_title(party: &Party, time: f32, r: &mut Renderer) {
        r.set_clear_color(color::rgb(10, 10, 20));
        r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(14, 12, 26));
        r.draw_rect(0.0, 40.0, VIRTUAL_W, 40.0, color::rgba(40, 30, 70, 255));

        r.draw_text_centered(
            "HERO OF THE OVERWORLD",
            VIRTUAL_W / 2.0,
            30.0,
            1.6,
            color::rgb(255, 226, 120),
        );
        r.draw_text_centered(
            "a tiny extensible JRPG",
            VIRTUAL_W / 2.0,
            52.0,
            1.0,
            color::rgb(180, 180, 210),
        );

        r.draw_text_centered(
            "TRAVEL THE MAP. CLEAR EACH LEVEL OF DEMONS.",
            VIRTUAL_W / 2.0,
            96.0,
            1.0,
            color::rgb(200, 220, 200),
        );

        let mut px = 60.0;
        r.draw_text("PARTY:", px, 130.0, 1.0, color::rgb(160, 200, 255));
        px += 46.0;
        for m in &party.members {
            r.draw_text(
                &format!("{} LV{}", m.name, m.level),
                px,
                130.0,
                1.0,
                color::WHITE,
            );
            px += r.text_width(&format!("{} LV{} ", m.name, m.level), 1.0) + 6.0;
        }

        if (time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "PRESS ENTER TO BEGIN",
                VIRTUAL_W / 2.0,
                160.0,
                1.0,
                color::rgb(150, 150, 180),
            );
        }
    }

    fn draw_map(&self, r: &mut Renderer) {
        r.set_clear_color(color::rgb(16, 22, 30));
        r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(20, 28, 36));
        r.draw_rect(0.0, 0.0, VIRTUAL_W, 16.0, color::rgba(10, 14, 22, 220));
        r.draw_text("WORLD MAP", 6.0, 3.0, 1.0, color::rgb(220, 225, 200));
        let cleared_count = self.cleared.iter().filter(|&&c| c).count();
        let prog = format!("CLEARED {}/{}", cleared_count, self.reg.data.levels.len());
        let pw = r.text_width(&prog, 1.0);
        r.draw_text(
            &prog,
            VIRTUAL_W - pw - 6.0,
            3.0,
            1.0,
            color::rgb(150, 210, 160),
        );

        let levels = &self.reg.data.levels;

        // Faint path connecting the levels in order (a travel route).
        for pair in levels.windows(2) {
            let a = node_px(pair[0].node);
            let b = node_px(pair[1].node);
            draw_dotted_line(r, a, b, color::rgba(90, 110, 130, 150));
        }

        // Level markers.
        for (i, lv) in levels.iter().enumerate() {
            let p = node_px(lv.node);
            let selected = i == self.map_cursor;
            let done = self.cleared[i];
            if selected {
                r.draw_rect_outline(
                    p.0 - 9.0,
                    p.1 - 9.0,
                    18.0,
                    18.0,
                    1.0,
                    color::rgb(255, 240, 150),
                );
            }
            let fill = if done {
                color::rgb(90, 200, 110)
            } else {
                color::rgb(180, 90, 90)
            };
            r.draw_rect(p.0 - 6.0, p.1 - 6.0, 12.0, 12.0, fill);
            r.draw_rect_outline(
                p.0 - 6.0,
                p.1 - 6.0,
                12.0,
                12.0,
                1.0,
                color::rgba(20, 20, 30, 255),
            );
            if done {
                r.draw_text_centered("*", p.0, p.1 - 4.0, 1.0, color::rgb(20, 40, 20));
            }
            // Label under the marker.
            let name_col = if selected {
                color::rgb(255, 240, 150)
            } else {
                color::rgb(200, 200, 210)
            };
            r.draw_text_centered(&lv.name, p.0, p.1 + 10.0, 1.0, name_col);
        }

        // Footer: party status + prompt.
        let mut px = 6.0;
        for m in &self.party.members {
            let label = format!(
                "{} LV{} {}/{}HP",
                m.name,
                m.level,
                m.hp.max(0),
                m.stats.max_hp
            );
            r.draw_text(&label, px, VIRTUAL_H - 22.0, 1.0, color::rgb(200, 220, 210));
            px += r.text_width(&label, 1.0) + 10.0;
        }
        if (self.time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "ARROWS: SELECT   ENTER: PLAY   ESC: TITLE",
                VIRTUAL_W / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgb(160, 170, 190),
            );
        }
    }

    fn draw_report(win: bool, lines: &[String], r: &mut Renderer) {
        r.set_clear_color(color::rgb(8, 8, 16));
        r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(12, 10, 22));
        let (title, col) = if win {
            ("VICTORY", color::rgb(255, 230, 120))
        } else {
            ("DEFEAT", color::rgb(230, 120, 120))
        };
        r.draw_text_centered(title, VIRTUAL_W / 2.0, 50.0, 2.0, col);
        for (i, line) in lines.iter().enumerate() {
            r.draw_text_centered(
                line,
                VIRTUAL_W / 2.0,
                84.0 + i as f32 * 12.0,
                1.0,
                color::WHITE,
            );
        }
        r.draw_text_centered(
            "PRESS ENTER TO CONTINUE",
            VIRTUAL_W / 2.0,
            150.0,
            1.0,
            color::rgb(160, 160, 190),
        );
    }
}

/// Pixel position of a level marker from its node grid coords.
fn node_px(node: (u32, u32)) -> (f32, f32) {
    (44.0 + node.0 as f32 * 44.0, 54.0 + node.1 as f32 * 34.0)
}

/// A simple dashed line between two points (evenly spaced dots).
fn draw_dotted_line(r: &mut Renderer, a: (f32, f32), b: (f32, f32), c: [f32; 4]) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let steps = (len / 6.0) as i32;
    for s in 1..steps {
        let t = s as f32 / steps as f32;
        let x = a.0 + dx * t;
        let y = a.1 + dy * t;
        r.draw_rect(x - 1.0, y - 1.0, 2.0, 2.0, c);
    }
}
