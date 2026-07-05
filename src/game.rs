//! Top-level game: owns persistent state (registry, party, RNG, audio) and the
//! current scene, and routes update/draw.
//!
//! Scene flow: the title leads to a **map screen** where you pick a level. Each
//! level is a set of connected [`Overworld`] screens you walk between; roaming
//! demons chase you and start a turn-based [`Battle`] on contact. Clearing every
//! demon in a level marks it done on the map.

use std::collections::{HashMap, HashSet};

use crate::audio::{Audio, Track};
use crate::battle::{Battle, BattleOutcome};
use crate::cutscene::{Cutscene, CutsceneOutcome};
use crate::data::Registry;
use crate::input::{Button, Controllers, Input, InputAssignment, TouchScheme};
use crate::input_config::{InputConfig, InputConfigEvent};
use crate::inventory::{Inventory, InventoryEvent};
use crate::overworld::{Event, Overworld, Trigger};
use crate::party::{Party, PartyMember};
use crate::renderer::{color, Renderer, VIRTUAL_H, VIRTUAL_W};
use crate::save::{self, SaveData, SavedLevel, SavedLocation, SavedMember};
use crate::shop::{Shop, ShopEvent};
use crate::util::{Rng, TextureCache};

enum Scene {
    Title,
    Map,
    Level,
    Cutscene(Cutscene),
    Battle(Battle),
    /// Inside a shop, entered from a keeper on the overworld. The `Level` runtime
    /// stays alive in `self.level` so leaving drops you back where you entered.
    Shop(Shop),
    /// The party inventory / equipment screen, opened with Menu from a level. The
    /// `Level` runtime stays alive in `self.level` so closing returns you to it.
    Inventory(Inventory),
    /// The input-mapping config, opened with Menu from the title screen: assign
    /// the keyboard and each gamepad to players for local co-op.
    InputConfig(InputConfig),
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
    /// Per-level in-progress state (which demons are beaten), keyed by level id.
    /// Persisted so quitting mid-level keeps the enemies you've already cleared.
    level_progress: HashMap<String, Vec<Vec<bool>>>,
    /// Whether a save was loaded at startup (drives the title's CONTINUE prompt).
    has_save: bool,
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
    /// How input sources (keyboard, each pad) map to players. Edited on the input
    /// config screen, fed to [`Controllers::poll`] each frame, and persisted.
    input_assignment: InputAssignment,
}

impl Game {
    pub fn new(renderer: &mut Renderer, audio: Audio) -> Self {
        let reg = Registry::load();
        let party = Party::from_registry(&reg);
        let cleared = vec![false; reg.data.levels.len()];
        let mut game = Game {
            reg,
            party,
            cache: TextureCache::new(),
            rng: Rng::seeded_now(),
            audio,
            level: None,
            current_level: 0,
            cleared,
            level_progress: HashMap::new(),
            has_save: false,
            map_cursor: 0,
            pending: None,
            played_cutscenes: HashSet::new(),
            pending_cutscene: None,
            scene: Scene::Title,
            time: 0.0,
            input_assignment: InputAssignment::default(),
        };
        // Resume a prior session if one is on disk / in the browser.
        if let Some(data) = save::load() {
            game.apply_save(data, renderer);
            game.has_save = true;
        }
        game
    }

    /// Overwrite live state from a decoded save. Immutable member data (name,
    /// sprite, skills) is rebuilt from the registry via `def_id`; unknown members
    /// or level ids are skipped so an old save still loads against edited content.
    fn apply_save(&mut self, data: SaveData, renderer: &mut Renderer) {
        self.party.gold = data.gold;
        self.party.members.clear();
        for sm in &data.members {
            let Some(mut m) = PartyMember::from_def(&self.reg, &sm.def_id) else {
                log::warn!(
                    "save references unknown character '{}'; skipping",
                    sm.def_id
                );
                continue;
            };
            m.level = sm.level;
            m.xp = sm.xp;
            m.hp = sm.hp;
            m.mp = sm.mp;
            m.stats.max_hp = sm.max_hp;
            m.stats.max_mp = sm.max_mp;
            m.stats.attack = sm.attack;
            m.stats.defense = sm.defense;
            m.stats.magic = sm.magic;
            m.stats.speed = sm.speed;
            m.weapon = sm.weapon.clone();
            m.armor = sm.armor.clone();
            self.party.members.push(m);
        }
        // `cleared` always matches the current level count; copy what overlaps.
        self.cleared = vec![false; self.reg.data.levels.len()];
        for (i, &c) in data.cleared.iter().enumerate() {
            if let Some(slot) = self.cleared.get_mut(i) {
                *slot = c;
            }
        }
        self.party.bag = data.bag;
        self.input_assignment = InputAssignment {
            keyboard: data.input_keyboard as usize,
            gamepads: data.input_gamepads.iter().map(|&p| p as usize).collect(),
        };
        self.played_cutscenes = data.played_cutscenes.into_iter().collect();
        self.level_progress = data.levels.into_iter().map(|l| (l.id, l.screens)).collect();

        // If the save was taken inside a level, rebuild that level and drop the
        // player back at their exact screen/position. A `None` location (saved on
        // the world map), or a level id no longer in the registry, resumes on the
        // map instead. The title's CONTINUE prompt then leads straight into the
        // restored level (see `Scene::Title`).
        if let Some(loc) = data.location {
            if let Some(idx) = self
                .reg
                .data
                .levels
                .iter()
                .position(|l| l.id == loc.level_id)
            {
                self.current_level = idx;
                self.map_cursor = idx;
                let defeated = self
                    .level_progress
                    .get(&loc.level_id)
                    .cloned()
                    .unwrap_or_default();
                let mut level = Overworld::new(
                    renderer,
                    &mut self.cache,
                    &self.reg,
                    &self.party,
                    idx,
                    &defeated,
                );
                level.set_position(loc.screen, loc.x, loc.y);
                self.level = Some(level);
            } else {
                log::warn!(
                    "save references unknown level '{}'; resuming on the map",
                    loc.level_id
                );
            }
        }
    }

    /// Snapshot the whole game into a [`SaveData`] and persist it. Called after
    /// anything that changes lasting state (a battle, a clear, leaving a level).
    fn save(&mut self) {
        // Fold the live level's progress in first so it's never a frame stale.
        self.capture_level_progress();
        let members = self
            .party
            .members
            .iter()
            .map(|m| SavedMember {
                def_id: m.def_id.clone(),
                level: m.level,
                xp: m.xp,
                hp: m.hp,
                mp: m.mp,
                max_hp: m.stats.max_hp,
                max_mp: m.stats.max_mp,
                attack: m.stats.attack,
                defense: m.stats.defense,
                magic: m.stats.magic,
                speed: m.stats.speed,
                weapon: m.weapon.clone(),
                armor: m.armor.clone(),
            })
            .collect();
        let levels = self
            .level_progress
            .iter()
            .map(|(id, screens)| SavedLevel {
                id: id.clone(),
                screens: screens.clone(),
            })
            .collect();
        // Record where the player is standing so a resumed session lands back in
        // the level. `None` while on the map, so continuing then starts on the map.
        let location = self.level.as_ref().map(|level| {
            let (x, y) = level.player_pos();
            SavedLocation {
                level_id: self.reg.data.levels[self.current_level].id.clone(),
                screen: level.current_screen(),
                x,
                y,
            }
        });
        let data = SaveData {
            gold: self.party.gold,
            members,
            cleared: self.cleared.clone(),
            played_cutscenes: self.played_cutscenes.iter().cloned().collect(),
            levels,
            location,
            bag: self.party.bag.clone(),
            input_keyboard: self.input_assignment.keyboard as u32,
            input_gamepads: self
                .input_assignment
                .gamepads
                .iter()
                .map(|&p| p as u32)
                .collect(),
        };
        save::store(&data);
        self.has_save = true;
    }

    /// Record the active level's defeated-enemy state into `level_progress`.
    fn capture_level_progress(&mut self) {
        if let Some(level) = &self.level {
            let id = self.reg.data.levels[self.current_level].id.clone();
            self.level_progress.insert(id, level.defeated_state());
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

    /// Which on-screen touch scheme fits the current scene: an analog joystick
    /// while walking the overworld, up/down only in the vertical battle menu, and
    /// a plain d-pad everywhere else (title, map, shop, dialogue). Read by the
    /// main loop before polling input.
    /// The live input-source → player mapping, read by the main loop each frame
    /// when it polls the controllers.
    pub fn input_assignment(&self) -> &InputAssignment {
        &self.input_assignment
    }

    pub fn touch_scheme(&self) -> TouchScheme {
        match self.scene {
            Scene::Level => TouchScheme::Joystick,
            Scene::Battle(_) => TouchScheme::UpDown,
            // The inventory is a menu screen → d-pad (the catch-all below).
            _ => TouchScheme::Dpad,
        }
    }

    pub fn update(&mut self, controllers: &Controllers, renderer: &mut Renderer, dt: f32) {
        self.time += dt;
        // Every screen but battle is single-player, so it reads the shared input
        // (keyboard + any gamepad). Battle hands each party member their own
        // gamepad, so it takes the whole controller set.
        let input = controllers.shared();
        // Take the scene out to sidestep borrow conflicts, put it back after.
        let scene = std::mem::replace(&mut self.scene, Scene::Title);
        self.scene = match scene {
            Scene::Title => {
                if input.pressed(Button::Confirm) {
                    // Resume straight into the level if the save restored one.
                    if self.level.is_some() {
                        Scene::Level
                    } else {
                        Scene::Map
                    }
                } else if input.pressed(Button::Menu) {
                    // Open the controls / input-mapping config.
                    let mut cfg = InputConfig::new(self.input_assignment.clone());
                    cfg.sync_gamepads(controllers.gamepad_count());
                    Scene::InputConfig(cfg)
                } else {
                    Scene::Title
                }
            }
            Scene::Map => self.update_map(input, renderer),
            Scene::Level => self.update_level(input, renderer, dt),
            Scene::Cutscene(mut cs) => match cs.update(input, &mut self.party, &self.reg, dt) {
                Some(CutsceneOutcome::Finished) => {
                    // A cutscene can recruit a new member, so persist afterwards.
                    self.save();
                    if self.level.is_some() {
                        Scene::Level
                    } else {
                        Scene::Map
                    }
                }
                None => Scene::Cutscene(cs),
            },
            Scene::Battle(mut battle) => {
                match battle.update(controllers, &mut self.rng, &self.reg, dt) {
                    Some(outcome) => {
                        battle.sync_party(&mut self.party);
                        self.finish_battle(outcome, renderer)
                    }
                    None => Scene::Battle(battle),
                }
            }
            Scene::Shop(mut shop) => match shop.update(input, &mut self.party, dt) {
                // Leaving a shop can have changed gold/equipment, so persist it.
                Some(ShopEvent::Exit) => {
                    self.save();
                    Scene::Level
                }
                None => Scene::Shop(shop),
            },
            Scene::Inventory(mut inv) => match inv.update(input, &mut self.party, &self.reg, dt) {
                // Closing can have re-equipped gear, so persist it.
                Some(InventoryEvent::Close) => {
                    self.save();
                    Scene::Level
                }
                None => Scene::Inventory(inv),
            },
            Scene::InputConfig(mut cfg) => {
                cfg.sync_gamepads(controllers.gamepad_count());
                match cfg.update(input) {
                    Some(InputConfigEvent::Close(assign)) => {
                        // Apply the new mapping and remember it across sessions.
                        self.input_assignment = assign;
                        self.save();
                        Scene::Title
                    }
                    None => Scene::InputConfig(cfg),
                }
            }
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
        if input.pressed(Button::Confirm) && self.unlocked(self.map_cursor) {
            self.current_level = self.map_cursor;
            // Restore this level's saved progress (beaten demons) if any.
            let level_id = self.reg.data.levels[self.current_level].id.clone();
            let defeated = self
                .level_progress
                .get(&level_id)
                .cloned()
                .unwrap_or_default();
            self.level = Some(Overworld::new(
                renderer,
                &mut self.cache,
                &self.reg,
                &self.party,
                self.current_level,
                &defeated,
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
                // Fold in beaten-demon progress while the level is still live,
                // then drop it *before* saving so the save records "on the map"
                // (no location) — resuming won't teleport back into the level.
                self.capture_level_progress();
                self.level = None;
                self.save();
                Scene::Map
            }
            Some(Event::EnterShop(id)) => {
                let Some(def) = self.reg.shop(&id) else {
                    log::warn!("entrance references unknown shop '{id}'");
                    return Scene::Level;
                };
                let shop = Shop::new(renderer, &mut self.cache, &self.reg, &self.party, def);
                Scene::Shop(shop)
            }
            Some(Event::OpenInventory) => Scene::Inventory(Inventory::new()),
            Some(Event::Battle(trigger)) => {
                // Members who fell in a previous fight rejoin this one with a
                // sliver of health instead of staying gone.
                self.party.revive_downed(5);
                let battle = Battle::new(
                    renderer,
                    &mut self.cache,
                    &self.reg,
                    &self.party,
                    &trigger.encounter,
                );
                // Boss encounters (e.g. the DEMON FORTRESS dragon) swap in the
                // dedicated boss theme; everything else uses the battle track.
                let is_boss = self
                    .reg
                    .encounter(&trigger.encounter)
                    .is_some_and(|e| e.boss);
                let track = if is_boss { Track::Boss } else { Track::Battle };
                self.audio.play_music_looping(track);
                self.pending = Some(trigger);
                Scene::Battle(battle)
            }
        }
    }

    /// Whether level `i` can be entered yet. Progression is linear: the first
    /// level is always open, and each later one unlocks only once the level
    /// before it is fully cleared.
    fn unlocked(&self, i: usize) -> bool {
        if i >= self.reg.data.levels.len() {
            return false;
        }
        i == 0 || self.cleared.get(i - 1).copied().unwrap_or(false)
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
            if best.is_none_or(|(b, _)| score < b) {
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
        let scene = match outcome {
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
        };
        // Persist the outcome: XP/levels/gold, live HP/MP, and which demons in
        // the level are now beaten (folded in from the live level).
        self.save();
        scene
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        match &mut self.scene {
            Scene::Title => Self::draw_title(&self.party, self.time, self.has_save, renderer),
            Scene::Map => self.draw_map(renderer),
            Scene::Level => {
                if let Some(level) = &self.level {
                    level.draw(renderer);
                }
            }
            Scene::Cutscene(cs) => cs.draw(renderer),
            Scene::Battle(battle) => battle.draw(renderer, &self.reg),
            Scene::Shop(shop) => shop.draw(renderer, &self.reg, &self.party),
            Scene::Inventory(inv) => inv.draw(renderer, &self.party, &self.reg),
            Scene::InputConfig(cfg) => cfg.draw(renderer),
            Scene::Report { win, lines, .. } => Self::draw_report(*win, lines, renderer),
        }
    }

    fn draw_title(party: &Party, time: f32, has_save: bool, r: &mut Renderer) {
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
            // A resumed session says CONTINUE; a fresh one says BEGIN.
            let prompt = if has_save {
                "PRESS ENTER TO CONTINUE"
            } else {
                "PRESS ENTER TO BEGIN"
            };
            r.draw_text_centered(
                prompt,
                VIRTUAL_W / 2.0,
                160.0,
                1.0,
                color::rgb(150, 150, 180),
            );
        }

        // Always-on hint for the input/controls config (Menu key).
        r.draw_text_centered(
            "PRESS MENU (SHIFT / START) FOR CONTROLS",
            VIRTUAL_W / 2.0,
            171.0,
            1.0,
            color::rgb(110, 110, 140),
        );
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
            let unlocked = self.unlocked(i);
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
            // Green = cleared, red = available, grey = still locked.
            let fill = if done {
                color::rgb(90, 200, 110)
            } else if unlocked {
                color::rgb(180, 90, 90)
            } else {
                color::rgb(70, 74, 84)
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
            } else if !unlocked {
                // A padlock hint for levels not yet reachable.
                r.draw_text_centered("X", p.0, p.1 - 4.0, 1.0, color::rgb(150, 155, 165));
            }
            // Label under the marker.
            let name_col = if selected {
                color::rgb(255, 240, 150)
            } else if unlocked {
                color::rgb(200, 200, 210)
            } else {
                color::rgb(130, 135, 145)
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
        // A locked selection can't be entered; say why. Otherwise the controls.
        if !self.unlocked(self.map_cursor) {
            r.draw_text_centered(
                "LOCKED - CLEAR THE PREVIOUS LEVEL FIRST",
                VIRTUAL_W / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgb(210, 170, 120),
            );
        } else if (self.time * 2.0) as i32 % 2 == 0 {
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
