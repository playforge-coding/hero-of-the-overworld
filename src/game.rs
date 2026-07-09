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
#[cfg(debug_assertions)]
use crate::devtools::{DevTools, DevToolsEvent};
use crate::input::{Button, Controllers, Input, InputAssignment, TouchScheme};
use crate::input_config::{InputConfig, InputConfigEvent};
use crate::inventory::{Inventory, InventoryEvent};
use crate::overworld::{Event, LevelProgress, Overworld, Trigger};
use crate::party::{ItemStack, Party, PartyMember};
use crate::renderer::{color, virtual_w, Renderer, VIRTUAL_H};
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
    /// **DEV-ONLY** developer menu, opened with F1 from the map: set the party's
    /// level, add any character, or fight any encounter. Only reachable in debug
    /// builds (the whole variant is compiled out of `--release`).
    #[cfg(debug_assertions)]
    DevTools(DevTools),
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
    /// Per-level in-progress state (beaten enemies, opened chests, slain mimics),
    /// keyed by level id. Persisted so quitting mid-level keeps the enemies you've
    /// already cleared and the treasure you've already looted.
    level_progress: HashMap<String, LevelProgress>,
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
    /// The level index to auto-advance into once the clear sequence (victory
    /// report, then any clear cutscene) finishes. Set when a level is freshly
    /// cleared, so beating the last foe carries the party straight into the next
    /// region instead of dropping them back on the map. `None` the rest of the time.
    pending_advance: Option<usize>,
    /// The story **chapter** the party is in (1-based). The world map only offers
    /// the current chapter's [levels](crate::data::LevelDef::chapter); it starts at
    /// 1 and ticks up when a chapter-advancing boss is faced (the DEMON KING, whose
    /// unwinnable fight hurls the party to the surface and into chapter 2). Persisted.
    chapter: u32,
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
            pending_advance: None,
            chapter: 1,
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
            // Skills aren't saved — they're rebuilt from the def. `from_def` taught
            // the level-1 kit; now that the saved level is set, teach everything the
            // learnset unlocks up to it so a reloaded hero keeps the moves they'd
            // earned.
            if let Some(def) = self.reg.character(&sm.def_id) {
                m.learn_skills_for_level(def);
            }
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
        self.party.items = data
            .items
            .into_iter()
            .map(|(id, count)| ItemStack { id, count })
            .collect();
        self.input_assignment = InputAssignment {
            keyboard: data.input_keyboard as usize,
            gamepads: data.input_gamepads.iter().map(|&p| p as usize).collect(),
        };
        self.chapter = data.chapter.max(1);
        self.played_cutscenes = data.played_cutscenes.into_iter().collect();
        // Merge the three per-level bool grids (enemies / chests / mimics), each
        // keyed by level id, back into one `LevelProgress` per level.
        let mut level_progress: HashMap<String, LevelProgress> = HashMap::new();
        for l in data.levels {
            level_progress.entry(l.id).or_default().enemies = l.screens;
        }
        for l in data.chest_levels {
            level_progress.entry(l.id).or_default().chests = l.screens;
        }
        for l in data.mimic_levels {
            level_progress.entry(l.id).or_default().mimics = l.screens;
        }
        self.level_progress = level_progress;

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
                let progress = self
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
                    &progress,
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
        // Split each level's `LevelProgress` back into three id-keyed grids for
        // the save file (enemies stay in `levels`; chests/mimics ride along in
        // their own trailing, back-compatible sections).
        let mut levels = Vec::new();
        let mut chest_levels = Vec::new();
        let mut mimic_levels = Vec::new();
        for (id, prog) in &self.level_progress {
            levels.push(SavedLevel {
                id: id.clone(),
                screens: prog.enemies.clone(),
            });
            chest_levels.push(SavedLevel {
                id: id.clone(),
                screens: prog.chests.clone(),
            });
            mimic_levels.push(SavedLevel {
                id: id.clone(),
                screens: prog.mimics.clone(),
            });
        }
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
            chapter: self.chapter,
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
            items: self
                .party
                .items
                .iter()
                .map(|s| (s.id.clone(), s.count))
                .collect(),
            chest_levels,
            mimic_levels,
        };
        save::store(&data);
        self.has_save = true;
    }

    /// Record the active level's progress (beaten enemies, opened chests, slain
    /// mimics) into `level_progress`.
    fn capture_level_progress(&mut self) {
        if let Some(level) = &self.level {
            let id = self.reg.data.levels[self.current_level].id.clone();
            self.level_progress.insert(id, level.progress());
        }
    }

    /// **DEV-ONLY.** Reset level `i` to its untouched state so it can be replayed
    /// fresh: clear its cleared flag, drop its defeated-enemy progress (so every
    /// foe respawns on re-entry), and forget its intro/clear cutscenes (so they
    /// play again). If that level happens to be loaded, drop it too. The caller
    /// saves afterwards. Compiled out of release builds along with its only caller.
    #[cfg(debug_assertions)]
    fn reset_level(&mut self, i: usize) {
        let Some(level) = self.reg.data.levels.get(i) else {
            return;
        };
        let id = level.id.clone();
        let cutscenes: Vec<String> = level
            .intro_cutscene
            .iter()
            .chain(level.clear_cutscene.iter())
            .cloned()
            .collect();
        if let Some(done) = self.cleared.get_mut(i) {
            *done = false;
        }
        self.level_progress.remove(&id);
        for cs in cutscenes {
            self.played_cutscenes.remove(&cs);
        }
        // If this level is the loaded one, drop it so a stale runtime can't write
        // its progress back on the next save/exit.
        if self.current_level == i {
            self.level = None;
        }
    }

    /// **DEV-ONLY.** The XP the party would earn by clearing every enemy in level
    /// `i` — the sum of each spawn's encounter, each enemy's XP **scaled to the
    /// party's current level** exactly as real battle spoils are (see
    /// [`crate::data::enemy_scale`]). The map's dev level-skip grants this so a
    /// skipped level still advances the party roughly as far as actually playing
    /// it would, keeping later-level testing realistic. (A single-scale snapshot,
    /// so it doesn't model levelling *mid-clear* — close enough for a dev tool.)
    #[cfg(debug_assertions)]
    fn level_clear_xp(&self, i: usize) -> i32 {
        let Some(level) = self.reg.data.levels.get(i) else {
            return 0;
        };
        let scale = crate::data::enemy_scale(self.party.level());
        let mut xp = 0;
        for screen in &level.screens {
            for spawn in &screen.spawns {
                let Some(enc) = self.reg.encounter(&spawn.encounter) else {
                    continue;
                };
                for eid in &enc.enemies {
                    if let Some(e) = self.reg.enemy(eid) {
                        xp += e.xp * scale / 100;
                    }
                }
            }
        }
        xp
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
                    // A clear cutscene is the tail of a clear sequence — carry on
                    // into the next level if one is queued.
                    self.after_clear_sequence(renderer)
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
            #[cfg(debug_assertions)]
            Scene::DevTools(mut dev) => match dev.update(input) {
                Some(DevToolsEvent::Close) => {
                    self.save();
                    Scene::Map
                }
                Some(DevToolsEvent::SetLevel(n)) => {
                    self.party.set_level(&self.reg, n);
                    self.save();
                    Scene::DevTools(dev)
                }
                Some(DevToolsEvent::AddMember(id)) => {
                    self.party.recruit(&self.reg, &id);
                    self.save();
                    Scene::DevTools(dev)
                }
                Some(DevToolsEvent::Fight(id)) => {
                    // Start a battle with no map trigger: finish_battle only
                    // touches level state when both a level and a trigger exist,
                    // so a dev fight still grants spoils and lands back on the map.
                    self.party.revive_downed(5);
                    let battle =
                        Battle::new(renderer, &mut self.cache, &self.reg, &self.party, &id);
                    let is_boss = self.reg.encounter(&id).is_some_and(|e| e.boss);
                    let track = if is_boss { Track::Boss } else { Track::Battle };
                    self.audio.play_music_looping(track);
                    self.pending = None;
                    Scene::Battle(battle)
                }
                None => Scene::DevTools(dev),
            },
            Scene::Report {
                win,
                lines,
                mut timer,
            } => {
                timer -= dt;
                if timer <= 0.0 && input.any_pressed() {
                    // A queued clear/recruit cutscene plays before returning; with
                    // none, fall through to the clear sequence (which auto-advances
                    // to the next level when one was just cleared).
                    match self.pending_cutscene.take() {
                        Some(cs) => Scene::Cutscene(cs),
                        None => self.after_clear_sequence(renderer),
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
        // DEV-ONLY level skipping: mark the highlighted level cleared so the next
        // one unlocks, letting a developer jump ahead (all the way to the
        // underworld) without playing through. To keep the skipped-ahead party
        // realistic for testing, also grant the XP that clearing the level's
        // enemies would have earned (scaled to the party's level, like real
        // spoils) — so a developer arrives at the next region roughly the level
        // they'd actually be. Only awarded the first time the level flips to
        // cleared, so mashing Tab doesn't stack XP. The entire block is compiled
        // out of release builds via `debug_assertions`, so a shipped game keeps
        // the normal linear gate and players cannot skip progression. See
        // [`input::dev_skip_pressed`].
        #[cfg(debug_assertions)]
        if crate::input::dev_skip_pressed() {
            let newly_cleared = !self.cleared.get(self.map_cursor).copied().unwrap_or(true);
            if let Some(done) = self.cleared.get_mut(self.map_cursor) {
                *done = true;
            }
            if newly_cleared {
                let xp = self.level_clear_xp(self.map_cursor);
                self.party.grant_xp(&self.reg, xp);
            }
            self.save();
            return Scene::Map;
        }
        // DEV-ONLY level reset: wipe the highlighted level back to its untouched
        // state — un-cleared, every enemy respawned, its intro/clear cutscenes
        // forgotten — so a developer can walk back in and replay it fresh after
        // tuning it. The inverse of the skip above; likewise compiled out of
        // release builds. See [`input::dev_reset_pressed`].
        #[cfg(debug_assertions)]
        if crate::input::dev_reset_pressed() {
            self.reset_level(self.map_cursor);
            self.save();
            return Scene::Map;
        }
        // DEV-ONLY developer menu: set the party's level, add any character, or
        // fight any encounter. Compiled out of release builds along with the
        // whole `DevTools` scene. See [`input::dev_menu_pressed`].
        #[cfg(debug_assertions)]
        if crate::input::dev_menu_pressed() {
            let characters = self
                .reg
                .data
                .characters
                .iter()
                .map(|c| (c.id.clone(), c.name.clone()))
                .collect();
            let encounters = self
                .reg
                .data
                .encounters
                .iter()
                .map(|e| e.id.clone())
                .collect();
            return Scene::DevTools(DevTools::new(self.party.level(), characters, encounters));
        }
        if input.pressed(Button::Confirm) && self.unlocked(self.map_cursor) {
            return self.enter_level(self.map_cursor, renderer);
        }
        Scene::Map
    }

    /// Load level `idx` as the active level and return the scene to show: its intro
    /// cutscene the first time it's entered, otherwise the level itself. Shared by
    /// picking a level on the map and auto-advancing after a clear.
    fn enter_level(&mut self, idx: usize, renderer: &mut Renderer) -> Scene {
        self.current_level = idx;
        self.map_cursor = idx;
        // Restore this level's saved progress (beaten demons, looted chests,
        // slain mimics) if any.
        let level_id = self.reg.data.levels[idx].id.clone();
        let progress = self
            .level_progress
            .get(&level_id)
            .cloned()
            .unwrap_or_default();
        self.level = Some(Overworld::new(
            renderer,
            &mut self.cache,
            &self.reg,
            &self.party,
            idx,
            &progress,
        ));
        // Play the level's intro cutscene the first time it's entered.
        if let Some(id) = self.reg.data.levels[idx].intro_cutscene.clone() {
            if let Some(cs) = self.build_cutscene(&id, renderer) {
                return Scene::Cutscene(cs);
            }
        }
        Scene::Level
    }

    /// The scene to show once a clear sequence (victory report → any clear
    /// cutscene) has fully played out. When a level was just cleared this
    /// **auto-advances** straight into the next region (persisting the jump);
    /// otherwise it's an ordinary return to the level (or the map if none is live).
    fn after_clear_sequence(&mut self, renderer: &mut Renderer) -> Scene {
        if let Some(next) = self.pending_advance.take() {
            let scene = self.enter_level(next, renderer);
            self.save();
            return scene;
        }
        if self.level.is_some() {
            Scene::Level
        } else {
            Scene::Map
        }
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
            Some(Event::OpenChest {
                gold,
                item,
                equipment,
            }) => {
                // Pour the chest's contents into the party and show a brief spoils
                // report, mirroring how battle loot is presented.
                let mut lines = Vec::new();
                if gold > 0 {
                    self.party.gold += gold;
                    lines.push(format!("FOUND {gold} GOLD"));
                }
                if let Some(id) = item {
                    self.party.add_item(&id, 1);
                    let name = self.reg.item(&id).map(|it| it.name.clone()).unwrap_or(id);
                    lines.push(format!("FOUND {name}"));
                }
                if let Some(id) = equipment {
                    let name = self
                        .reg
                        .equipment(&id)
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| id.clone());
                    self.party.bag.push(id);
                    lines.push(format!("FOUND {name}"));
                }
                if lines.is_empty() {
                    lines.push("THE CHEST IS EMPTY".to_string());
                }
                // The chest is already marked opened in the live level; persist it
                // (and the new loot) so it stays looted across save/reload.
                self.save();
                Scene::Report {
                    win: true,
                    lines,
                    timer: 0.8,
                }
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

    /// Whether level `i` can be entered yet. A level must belong to the party's
    /// current [chapter](Self::chapter); within a chapter, progression is linear —
    /// the first level of the chapter is always open, and each later one unlocks
    /// only once the previous level *of that same chapter* is fully cleared. Levels
    /// of a past or future chapter are always locked (the party has moved on from,
    /// or not yet reached, them).
    fn unlocked(&self, i: usize) -> bool {
        let Some(lv) = self.reg.data.levels.get(i) else {
            return false;
        };
        if lv.chapter != self.chapter {
            return false;
        }
        // Gate on the previous level of the same chapter; a chapter's first level
        // has none and is open from the start.
        match self.reg.data.levels[..i]
            .iter()
            .rposition(|l| l.chapter == lv.chapter)
        {
            Some(prev) => self.cleared.get(prev).copied().unwrap_or(false),
            None => true,
        }
    }

    /// Whether the party's current chapter has any levels at all. False once the
    /// party has advanced past the last authored chapter (the DEMON KING flings
    /// them into a chapter 2 that has no regions **yet**) — the map then shows a
    /// "to be continued" state rather than a playable region.
    fn chapter_has_levels(&self) -> bool {
        self.reg
            .data
            .levels
            .iter()
            .any(|l| l.chapter == self.chapter)
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
            // Only navigate between the current chapter's markers (the only ones the
            // map draws).
            if i == self.map_cursor || lv.chapter != self.chapter {
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
        // Names for the clear banner: the level just finished, and the next region
        // the party is being carried into (if any).
        let mut cleared_name: Option<String> = None;
        let mut next_name: Option<String> = None;
        if let (Some(level), Some(t)) = (&mut self.level, &trigger) {
            level.resolve_battle(t, won);
            if won && level.all_cleared() {
                self.cleared[self.current_level] = true;
                clear_cutscene = self.reg.data.levels[self.current_level]
                    .clear_cutscene
                    .clone();
                cleared_name = Some(self.reg.data.levels[self.current_level].name.clone());
                // Auto-advance: beating the last foe carries the party straight into
                // the next region (once the report and any clear cutscene finish),
                // rather than dropping them back on the map. Only within the same
                // chapter — a chapter boundary is crossed by story (a boss), not by
                // clearing — so the last region of a chapter simply ends the clear.
                let cur_chapter = self.reg.data.levels[self.current_level].chapter;
                let next = self.current_level + 1;
                if self
                    .reg
                    .data
                    .levels
                    .get(next)
                    .is_some_and(|l| l.chapter == cur_chapter)
                {
                    self.pending_advance = Some(next);
                    next_name = Some(self.reg.data.levels[next].name.clone());
                }
            }
        }
        // Queue the level-clear cutscene (e.g. a new ally joining) to play after
        // the victory report.
        if let Some(id) = clear_cutscene {
            self.pending_cutscene = self.build_cutscene(&id, renderer);
        }
        let scene = match outcome {
            BattleOutcome::Victory { xp, gold, drops } => {
                self.party.gold += gold;
                let leveled = self.party.grant_xp(&self.reg, xp);
                let mut lines = Vec::new();
                // Lead with the clear banner when the last foe just fell, naming
                // where the party is headed next.
                if let Some(name) = &cleared_name {
                    lines.push(format!("{name} CLEARED!"));
                    match &next_name {
                        Some(next) => lines.push(format!("ONWARD TO {next}")),
                        // The last built region of a chapter: the tale pauses here.
                        None => lines.push("TO BE CONTINUED...".to_string()),
                    }
                }
                lines.push(format!("GAINED {xp} XP  {gold} GOLD"));
                // Fold in any item drops the fallen enemies yielded.
                for id in &drops {
                    self.party.add_item(id, 1);
                    let name = self
                        .reg
                        .item(id)
                        .map(|it| it.name.clone())
                        .unwrap_or_else(|| id.clone());
                    lines.push(format!("FOUND {name}"));
                }
                // Report each level gained, and any skill it unlocked.
                for ev in &leveled {
                    let who = self
                        .party
                        .members
                        .get(ev.member)
                        .map(|m| m.name.clone())
                        .unwrap_or_default();
                    lines.push(format!("{who} REACHED LV {}", ev.level));
                    for id in &ev.learned {
                        let skill = self
                            .reg
                            .skill(id)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| id.clone());
                        lines.push(format!("{who} LEARNED {skill}"));
                    }
                }
                Scene::Report {
                    win: true,
                    lines,
                    timer: 0.8,
                }
            }
            BattleOutcome::Defeat => {
                // Does this encounter script a special defeat (the DEMON KING)? A
                // wipe here can be a story beat — a cutscene and/or a jump to the
                // next chapter — rather than the usual revive-at-camp. (A dev-menu
                // fight has no trigger, so it always takes the ordinary path.)
                let enc = trigger
                    .as_ref()
                    .and_then(|t| self.reg.encounter(&t.encounter));
                let defeat_cutscene = enc.and_then(|e| e.defeat_cutscene.clone());
                let advances_chapter = enc.is_some_and(|e| e.defeat_advances_chapter);

                // Whichever path, the party stands back up at full strength.
                self.party.full_heal();

                if advances_chapter {
                    // The unwinnable boss hurls the party back to the surface and the
                    // story turns over: leave the level (they land far from every
                    // region they'd unlocked) and tick the chapter, which re-gates the
                    // world map (see `unlocked`).
                    self.chapter += 1;
                    self.level = None;
                    self.pending_advance = None;
                    // Where the party washes up: the first region of the new chapter,
                    // if one is built. They walk straight into it once the launch
                    // cutscene ends; a chapter with no regions yet lands them on the
                    // (region-less) map — a "to be continued" cliffhanger.
                    let landing = self
                        .reg
                        .data
                        .levels
                        .iter()
                        .position(|l| l.chapter == self.chapter);
                    self.map_cursor = landing.unwrap_or(0);
                    // Play the launch cutscene first; after it, `after_clear_sequence`
                    // carries the party into the landing region (whose own intro then
                    // plays) via `pending_advance`.
                    if let Some(id) = &defeat_cutscene {
                        if let Some(cs) = self.build_cutscene(id, renderer) {
                            self.pending_advance = landing;
                            self.save();
                            return Scene::Cutscene(cs);
                        }
                    }
                    // No launch cutscene: step straight into the landing region, or
                    // land on the map if the new chapter has none yet.
                    let scene = match landing {
                        Some(idx) => self.enter_level(idx, renderer),
                        None => Scene::Map,
                    };
                    self.save();
                    return scene;
                }

                // Ordinary wipe: revive the party so exploration can continue.
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
            #[cfg(debug_assertions)]
            Scene::DevTools(dev) => dev.draw(renderer),
            Scene::Report { win, lines, .. } => Self::draw_report(*win, lines, renderer),
        }
    }

    fn draw_title(party: &Party, time: f32, has_save: bool, r: &mut Renderer) {
        r.set_clear_color(color::rgb(10, 10, 20));
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(14, 12, 26));
        r.draw_rect(0.0, 40.0, virtual_w(), 40.0, color::rgba(40, 30, 70, 255));

        r.draw_text_centered(
            "HERO OF THE OVERWORLD",
            virtual_w() / 2.0,
            30.0,
            1.6,
            color::rgb(255, 226, 120),
        );
        r.draw_text_centered(
            "a tiny extensible JRPG",
            virtual_w() / 2.0,
            52.0,
            1.0,
            color::rgb(180, 180, 210),
        );

        r.draw_text_centered(
            "TRAVEL THE MAP. CLEAR EACH LEVEL OF DEMONS.",
            virtual_w() / 2.0,
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
                virtual_w() / 2.0,
                160.0,
                1.0,
                color::rgb(150, 150, 180),
            );
        }

        // Always-on hint for the input/controls config (Menu key).
        r.draw_text_centered(
            "PRESS MENU (SHIFT / START) FOR CONTROLS",
            virtual_w() / 2.0,
            171.0,
            1.0,
            color::rgb(110, 110, 140),
        );
    }

    fn draw_map(&self, r: &mut Renderer) {
        r.set_clear_color(color::rgb(16, 22, 30));
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(20, 28, 36));
        r.draw_rect(0.0, 0.0, virtual_w(), 16.0, color::rgba(10, 14, 22, 220));
        r.draw_text(
            &format!("WORLD MAP · CH {}", self.chapter),
            6.0,
            3.0,
            1.0,
            color::rgb(220, 225, 200),
        );
        // Progress is scoped to the current chapter (the only regions on the map).
        let (cleared_count, chapter_total) = self.reg.data.levels.iter().enumerate().fold(
            (0usize, 0usize),
            |(done, total), (i, lv)| {
                if lv.chapter != self.chapter {
                    return (done, total);
                }
                (done + self.cleared[i] as usize, total + 1)
            },
        );
        let prog = format!("CLEARED {cleared_count}/{chapter_total}");
        let pw = r.text_width(&prog, 1.0);
        r.draw_text(
            &prog,
            virtual_w() - pw - 6.0,
            3.0,
            1.0,
            color::rgb(150, 210, 160),
        );
        // DEV-ONLY hint for the hidden level-skip / reset hotkeys (compiled out of
        // release). TAB marks the highlighted level cleared and grants its clear
        // XP; R resets it to replay.
        #[cfg(debug_assertions)]
        r.draw_text_centered(
            "DEV: TAB SKIPS (+XP) · R RESETS · F1 MENU",
            virtual_w() / 2.0,
            3.0,
            1.0,
            color::rgb(120, 130, 150),
        );

        let levels = &self.reg.data.levels;

        // The map only shows the party's **current chapter** — earlier chapters are
        // leagues behind them, later ones not yet reached. Faint path connecting this
        // chapter's regions in order (a travel route); links across a chapter boundary
        // are skipped so each chapter reads as its own map.
        for pair in levels.windows(2) {
            if pair[0].chapter != self.chapter || pair[1].chapter != self.chapter {
                continue;
            }
            let a = node_px(pair[0].node);
            let b = node_px(pair[1].node);
            draw_dotted_line(r, a, b, color::rgba(90, 110, 130, 150));
        }

        // Level markers (current chapter only).
        for (i, lv) in levels.iter().enumerate() {
            if lv.chapter != self.chapter {
                continue;
            }
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
        // Once the party has outrun every authored chapter, the map is a cliffhanger:
        // the regions they knew are out of reach and the next arc isn't built yet.
        if !self.chapter_has_levels() {
            r.draw_text_centered(
                &format!("CHAPTER {} - TO BE CONTINUED...", self.chapter),
                virtual_w() / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgb(230, 200, 140),
            );
        }
        // A locked selection can't be entered; say why. Otherwise the controls.
        else if !self.unlocked(self.map_cursor) {
            r.draw_text_centered(
                "LOCKED - CLEAR THE PREVIOUS LEVEL FIRST",
                virtual_w() / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgb(210, 170, 120),
            );
        } else if (self.time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "ARROWS: SELECT   ENTER: PLAY   ESC: TITLE",
                virtual_w() / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgb(160, 170, 190),
            );
        }
    }

    fn draw_report(win: bool, lines: &[String], r: &mut Renderer) {
        r.set_clear_color(color::rgb(8, 8, 16));
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(12, 10, 22));
        let (title, col) = if win {
            ("VICTORY", color::rgb(255, 230, 120))
        } else {
            ("DEFEAT", color::rgb(230, 120, 120))
        };
        r.draw_text_centered(title, virtual_w() / 2.0, 50.0, 2.0, col);
        for (i, line) in lines.iter().enumerate() {
            r.draw_text_centered(
                line,
                virtual_w() / 2.0,
                84.0 + i as f32 * 12.0,
                1.0,
                color::WHITE,
            );
        }
        r.draw_text_centered(
            "PRESS ENTER TO CONTINUE",
            virtual_w() / 2.0,
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
