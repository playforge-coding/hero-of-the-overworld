//! Top-level game: owns persistent state (registry, party, RNG) and the current
//! scene, and routes update/draw. Kept deliberately small — the title screen is
//! a stand-in for an eventual overworld; the battle is the real content.

use crate::battle::{Battle, BattleOutcome};
use crate::data::Registry;
use crate::input::{Button, Input};
use crate::party::Party;
use crate::renderer::{color, Renderer, VIRTUAL_H, VIRTUAL_W};
use crate::util::{Rng, TextureCache};

enum Scene {
    Title {
        cursor: usize,
    },
    Battle(Battle),
    Report {
        win: bool,
        lines: Vec<String>,
        timer: f32,
        encounter: usize,
    },
}

pub struct Game {
    reg: Registry,
    party: Party,
    cache: TextureCache,
    rng: Rng,
    scene: Scene,
    time: f32,
    last_encounter: usize,
}

impl Game {
    pub fn new(_renderer: &mut Renderer) -> Self {
        let reg = Registry::load();
        let party = Party::from_registry(&reg);
        Game {
            reg,
            party,
            cache: TextureCache::new(),
            rng: Rng::seeded_now(),
            scene: Scene::Title { cursor: 0 },
            time: 0.0,
            last_encounter: 0,
        }
    }

    pub fn update(&mut self, input: &Input, renderer: &mut Renderer, dt: f32) {
        self.time += dt;
        // Take the scene out to sidestep borrow conflicts, put it back after.
        let scene = std::mem::replace(&mut self.scene, Scene::Title { cursor: 0 });
        self.scene = match scene {
            Scene::Title { cursor } => self.update_title(cursor, input, renderer),
            Scene::Battle(mut battle) => match battle.update(input, &mut self.rng, &self.reg, dt) {
                Some(outcome) => {
                    battle.sync_party(&mut self.party);
                    self.finish_battle(outcome)
                }
                None => Scene::Battle(battle),
            },
            Scene::Report {
                win,
                lines,
                mut timer,
                encounter,
            } => {
                timer -= dt;
                if timer <= 0.0 && input.any_pressed() {
                    if !win {
                        // Wipe out: revive the party for another attempt.
                        self.party.full_heal();
                    }
                    Scene::Title { cursor: encounter }
                } else {
                    Scene::Report {
                        win,
                        lines,
                        timer,
                        encounter,
                    }
                }
            }
        };
    }

    fn update_title(&mut self, mut cursor: usize, input: &Input, renderer: &mut Renderer) -> Scene {
        let count = self.reg.data.encounters.len().max(1);
        if input.pressed(Button::Up) {
            cursor = (cursor + count - 1) % count;
        }
        if input.pressed(Button::Down) {
            cursor = (cursor + 1) % count;
        }
        if input.pressed(Button::Confirm) {
            self.last_encounter = cursor;
            let enc = self.reg.data.encounters[cursor].id.clone();
            let battle = Battle::new(renderer, &mut self.cache, &self.reg, &self.party, &enc);
            return Scene::Battle(battle);
        }
        Scene::Title { cursor }
    }

    fn finish_battle(&mut self, outcome: BattleOutcome) -> Scene {
        let encounter = self.last_encounter;
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
                    encounter,
                }
            }
            BattleOutcome::Defeat => Scene::Report {
                win: false,
                lines: vec!["THE PARTY HAS FALLEN".to_string()],
                timer: 0.8,
                encounter,
            },
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        match &mut self.scene {
            Scene::Title { cursor } => {
                Self::draw_title(&self.reg, &self.party, *cursor, self.time, renderer)
            }
            Scene::Battle(battle) => battle.draw(renderer, &self.reg),
            Scene::Report { win, lines, .. } => Self::draw_report(*win, lines, renderer),
        }
    }

    fn draw_title(reg: &Registry, party: &Party, cursor: usize, time: f32, r: &mut Renderer) {
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

        r.draw_text(
            "CHOOSE A BATTLE:",
            60.0,
            84.0,
            1.0,
            color::rgb(200, 220, 200),
        );
        for (i, enc) in reg.data.encounters.iter().enumerate() {
            let y = 98.0 + i as f32 * 12.0;
            let label = format!("{}  ({} FOES)", enc.id.to_uppercase(), enc.enemies.len());
            if i == cursor {
                r.draw_rect(58.0, y - 1.0, 200.0, 11.0, color::rgba(60, 80, 150, 200));
                r.draw_text(">", 60.0, y, 1.0, color::rgb(255, 240, 150));
            }
            r.draw_text(&label, 70.0, y, 1.0, color::WHITE);
        }

        // Party roster (shows extensibility — grows as members are added).
        let mut px = 60.0;
        r.draw_text("PARTY:", px, 150.0, 1.0, color::rgb(160, 200, 255));
        px += 46.0;
        for m in &party.members {
            r.draw_text(
                &format!("{} LV{}", m.name, m.level),
                px,
                150.0,
                1.0,
                color::WHITE,
            );
            px += r.text_width(&format!("{} LV{} ", m.name, m.level), 1.0) + 6.0;
        }

        if (time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "ARROWS: SELECT   ENTER: BEGIN",
                VIRTUAL_W / 2.0,
                166.0,
                1.0,
                color::rgb(150, 150, 180),
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
