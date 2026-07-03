//! Turn-based battle scene.
//!
//! Flow per round:
//!   1. Command phase — the player picks an action for each living hero.
//!   2. Enemies auto-plan via their [`EnemyAi`].
//!   3. All actions are ordered by speed and executed one at a time with a
//!      little movement/impact animation and floating damage numbers.
//!   4. Check for victory/defeat; otherwise start a new round.
//!
//! The scene is data-driven: heroes come from the [`Party`] and enemies from an
//! encounter in the [`Registry`], so more party members or new enemies work
//! with zero changes here.

use glam::Vec2;

use crate::data::{BattlerSprite, EnemyAi, Registry, SkillDef, SkillKind, Stats, TargetKind};
use crate::input::{Button, Input};
use crate::party::Party;
use crate::renderer::{color, Color, Renderer, TextureHandle, VIRTUAL_H, VIRTUAL_W};
use crate::util::{Rng, TextureCache};

// ---- Runtime animation ------------------------------------------------------

#[derive(Clone)]
struct Anim {
    row: u32,
    first_col: u32,
    frames: u32,
    fps: f32,
    t: f32,
    looping: bool,
}

impl Anim {
    fn from_clip(c: &crate::data::AnimClip, looping: bool) -> Self {
        Anim {
            row: c.row,
            first_col: c.first_col,
            frames: c.frames.max(1),
            fps: c.fps.max(0.001),
            t: 0.0,
            looping,
        }
    }

    fn update(&mut self, dt: f32) {
        self.t += dt;
    }

    fn col(&self) -> u32 {
        let f = (self.t * self.fps) as u32;
        let f = if self.looping {
            f % self.frames
        } else {
            f.min(self.frames - 1)
        };
        self.first_col + f
    }

    /// Source pixel rect within the sheet.
    fn src(&self, sprite: &BattlerSprite) -> [f32; 4] {
        [
            (self.col() * sprite.frame_w) as f32,
            (self.row * sprite.frame_h) as f32,
            sprite.frame_w as f32,
            sprite.frame_h as f32,
        ]
    }
}

// ---- Battler ----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Hero,
    Enemy,
}

struct Battler {
    name: String,
    side: Side,
    party_index: Option<usize>,
    stats: Stats,
    hp: i32,
    max_hp: i32,
    mp: i32,
    max_mp: i32,
    skills: Vec<String>,
    sprite: BattlerSprite,
    texture: TextureHandle,
    idle: Anim,
    anim: Anim,
    defending: bool,
    ai: EnemyAi,
    xp: i32,
    gold: i32,
    home: Vec2,
    offset: Vec2,
    flash: f32,
    fade: f32, // 1 = solid; enemies fade to 0 when defeated
}

impl Battler {
    fn alive(&self) -> bool {
        self.hp > 0
    }

    fn present(&self) -> bool {
        // Enemies vanish once fully faded; heroes always stay on the field.
        self.alive() || (matches!(self.side, Side::Hero)) || self.fade > 0.01
    }

    fn pos(&self) -> Vec2 {
        self.home + self.offset
    }

    fn facing_dir(&self) -> f32 {
        match self.side {
            Side::Hero => 1.0,   // heroes on the left face right
            Side::Enemy => -1.0, // enemies on the right face left
        }
    }

    fn flip_x(&self) -> bool {
        // Sprite faces right if !faces_left. Enemies want to face left.
        let art_faces_right = !self.sprite.faces_left;
        match self.side {
            Side::Hero => !art_faces_right,
            Side::Enemy => art_faces_right,
        }
    }
}

// ---- Actions ----------------------------------------------------------------

#[derive(Clone)]
enum ActionKind {
    Attack,
    Skill(String),
    Defend,
}

#[derive(Clone)]
struct Action {
    actor: usize,
    kind: ActionKind,
    targets: Vec<usize>,
}

// ---- Floating text ----------------------------------------------------------

struct Popup {
    text: String,
    pos: Vec2,
    t: f32,
    color: Color,
}

// ---- Battle state -----------------------------------------------------------

pub enum BattleOutcome {
    Victory { xp: i32, gold: i32 },
    Defeat,
}

enum State {
    Intro(f32),
    Command(Command),
    Execute(Execute),
    Result { win: bool, timer: f32 },
}

struct Command {
    order: Vec<usize>, // living hero battler indices, in turn order
    current: usize,    // index into `order`
    planned: Vec<Action>,
    stage: Stage,
}

enum Stage {
    Root {
        cursor: usize,
    },
    Skill {
        cursor: usize,
    },
    Target {
        pending: Pending,
        cursor: usize,
        candidates: Vec<usize>,
    },
}

#[derive(Clone)]
struct Pending {
    kind: ActionKind,
    target: TargetKind,
}

struct Execute {
    queue: Vec<Action>,
    idx: usize,
    elapsed: f32,
    applied: bool,
    banner: String,
    popups: Vec<Popup>,
}

const ROOT_ITEMS: [&str; 3] = ["ATTACK", "SKILL", "DEFEND"];

pub struct Battle {
    battlers: Vec<Battler>,
    state: State,
    encounter_name: String,
}

impl Battle {
    pub fn new(
        renderer: &mut Renderer,
        cache: &mut TextureCache,
        reg: &Registry,
        party: &Party,
        encounter_id: &str,
    ) -> Self {
        let mut battlers = Vec::new();

        // Heroes on the left.
        let living: Vec<usize> = (0..party.members.len())
            .filter(|&i| party.members[i].is_alive())
            .collect();
        for (slot, &pi) in living.iter().enumerate() {
            let m = &party.members[pi];
            let texture = cache.get(renderer, &m.sprite.texture);
            let home = hero_home(slot);
            battlers.push(Battler {
                name: m.name.clone(),
                side: Side::Hero,
                party_index: Some(pi),
                stats: m.stats.clone(),
                hp: m.hp,
                max_hp: m.stats.max_hp,
                mp: m.mp,
                max_mp: m.stats.max_mp,
                skills: m.skills.clone(),
                idle: Anim::from_clip(&m.sprite.idle, true),
                anim: Anim::from_clip(&m.sprite.idle, true),
                sprite: m.sprite.clone(),
                texture,
                defending: false,
                ai: EnemyAi::Basic,
                xp: 0,
                gold: 0,
                home,
                offset: Vec2::ZERO,
                flash: 0.0,
                fade: 1.0,
            });
        }

        // Enemies on the right, from the encounter.
        let enc = reg
            .encounter(encounter_id)
            .unwrap_or_else(|| panic!("unknown encounter '{encounter_id}'"));
        let name = encounter_id.to_string();
        for (slot, eid) in enc.enemies.iter().enumerate() {
            let def = reg
                .enemy(eid)
                .unwrap_or_else(|| panic!("unknown enemy '{eid}'"));
            let texture = cache.get(renderer, &def.sprite.texture);
            let home = enemy_home(slot);
            battlers.push(Battler {
                name: def.name.clone(),
                side: Side::Enemy,
                party_index: None,
                stats: def.stats.clone(),
                hp: def.stats.max_hp,
                max_hp: def.stats.max_hp,
                mp: def.stats.max_mp,
                max_mp: def.stats.max_mp,
                skills: def.skills.clone(),
                idle: Anim::from_clip(&def.sprite.idle, true),
                anim: Anim::from_clip(&def.sprite.idle, true),
                sprite: def.sprite.clone(),
                texture,
                defending: false,
                ai: def.ai(),
                xp: def.xp,
                gold: def.gold,
                home,
                offset: Vec2::ZERO,
                flash: 0.0,
                fade: 1.0,
            });
        }

        Battle {
            battlers,
            state: State::Intro(0.6),
            encounter_name: name,
        }
    }

    /// Copy surviving hero HP/MP back into the persistent party.
    pub fn sync_party(&self, party: &mut Party) {
        for b in &self.battlers {
            if let Some(pi) = b.party_index {
                party.members[pi].hp = b.hp.max(0);
                party.members[pi].mp = b.mp;
            }
        }
    }

    // ---- Queries ------------------------------------------------------------

    fn living(&self, side: Side) -> Vec<usize> {
        self.battlers
            .iter()
            .enumerate()
            .filter(|(_, b)| b.side == side && b.alive())
            .map(|(i, _)| i)
            .collect()
    }

    fn heroes_alive(&self) -> bool {
        !self.living(Side::Hero).is_empty()
    }

    fn enemies_alive(&self) -> bool {
        !self.living(Side::Enemy).is_empty()
    }

    /// Living battlers matching a skill's target kind, from `actor`'s view.
    fn candidates(&self, actor: usize, target: TargetKind) -> Vec<usize> {
        let side = self.battlers[actor].side;
        let foes = if side == Side::Hero {
            Side::Enemy
        } else {
            Side::Hero
        };
        match target {
            TargetKind::OneEnemy | TargetKind::AllEnemies => self.living(foes),
            TargetKind::OneAlly | TargetKind::AllAllies => self.living(side),
            TargetKind::SelfOnly => vec![actor],
        }
    }

    // ---- Update -------------------------------------------------------------

    pub fn update(
        &mut self,
        input: &Input,
        rng: &mut Rng,
        reg: &Registry,
        dt: f32,
    ) -> Option<BattleOutcome> {
        // Advance per-battler visual timers.
        for b in &mut self.battlers {
            b.idle.update(dt);
            b.anim.update(dt);
            if b.flash > 0.0 {
                b.flash = (b.flash - dt).max(0.0);
            }
            if !b.alive() && b.side == Side::Enemy && b.fade > 0.0 {
                b.fade = (b.fade - dt * 2.0).max(0.0);
            }
        }

        // Take the state out so we can freely borrow self.
        let mut state = std::mem::replace(&mut self.state, State::Intro(0.0));
        let outcome = match &mut state {
            State::Intro(timer) => {
                *timer -= dt;
                if *timer <= 0.0 {
                    self.state = self.begin_command();
                } else {
                    self.state = State::Intro(*timer);
                }
                None
            }
            State::Command(cmd) => {
                let next = self.update_command(cmd, input, reg);
                match next {
                    CommandResult::Stay => {
                        self.state = State::Command(std::mem::replace(cmd, dummy_command()));
                    }
                    CommandResult::Execute(exec) => self.state = State::Execute(exec),
                }
                None
            }
            State::Execute(exec) => {
                let done = self.update_execute(exec, rng, reg, dt);
                if done {
                    if !self.enemies_alive() {
                        self.state = State::Result {
                            win: true,
                            timer: 1.6,
                        };
                    } else if !self.heroes_alive() {
                        self.state = State::Result {
                            win: false,
                            timer: 1.6,
                        };
                    } else {
                        self.state = self.begin_command();
                    }
                } else {
                    self.state = State::Execute(std::mem::replace(exec, dummy_execute()));
                }
                None
            }
            State::Result { win, timer } => {
                *timer -= dt;
                if *timer <= 0.0 && input.any_pressed() || *timer <= -3.0 {
                    if *win {
                        let (xp, gold) = self.spoils();
                        Some(BattleOutcome::Victory { xp, gold })
                    } else {
                        Some(BattleOutcome::Defeat)
                    }
                } else {
                    self.state = State::Result {
                        win: *win,
                        timer: *timer,
                    };
                    None
                }
            }
        };
        outcome
    }

    fn spoils(&self) -> (i32, i32) {
        self.battlers
            .iter()
            .filter(|b| b.side == Side::Enemy)
            .fold((0, 0), |(xp, gold), b| (xp + b.xp, gold + b.gold))
    }

    fn begin_command(&mut self) -> State {
        for b in &mut self.battlers {
            b.defending = false;
        }
        let mut order = self.living(Side::Hero);
        order.sort_by_key(|&i| -self.battlers[i].stats.speed);
        State::Command(Command {
            order,
            current: 0,
            planned: Vec::new(),
            stage: Stage::Root { cursor: 0 },
        })
    }

    fn update_command(
        &mut self,
        cmd: &mut Command,
        input: &Input,
        reg: &Registry,
    ) -> CommandResult {
        if cmd.current >= cmd.order.len() {
            // All heroes have chosen: add enemy actions and build the queue.
            return CommandResult::Execute(self.build_execution(cmd, reg));
        }
        let hero = cmd.order[cmd.current];

        match &mut cmd.stage {
            Stage::Root { cursor } => {
                menu_move(cursor, ROOT_ITEMS.len(), input);
                if input.pressed(Button::Cancel) && cmd.current > 0 {
                    // Go back and re-plan the previous hero.
                    cmd.current -= 1;
                    cmd.planned.pop();
                    cmd.stage = Stage::Root { cursor: 0 };
                } else if input.pressed(Button::Confirm) {
                    match *cursor {
                        0 => {
                            let cands = self.candidates(hero, TargetKind::OneEnemy);
                            cmd.stage = Stage::Target {
                                pending: Pending {
                                    kind: ActionKind::Attack,
                                    target: TargetKind::OneEnemy,
                                },
                                cursor: 0,
                                candidates: cands,
                            };
                        }
                        1 => cmd.stage = Stage::Skill { cursor: 0 },
                        _ => {
                            cmd.planned.push(Action {
                                actor: hero,
                                kind: ActionKind::Defend,
                                targets: vec![],
                            });
                            cmd.current += 1;
                            cmd.stage = Stage::Root { cursor: 0 };
                        }
                    }
                }
                CommandResult::Stay
            }
            Stage::Skill { cursor } => {
                let skills = &self.battlers[hero].skills;
                let count = skills.len() + 1; // + BACK
                menu_move(cursor, count, input);
                if input.pressed(Button::Cancel) {
                    cmd.stage = Stage::Root { cursor: 1 };
                } else if input.pressed(Button::Confirm) {
                    if *cursor >= skills.len() {
                        cmd.stage = Stage::Root { cursor: 1 };
                    } else if let Some(def) = reg.skill(&skills[*cursor]) {
                        if self.battlers[hero].mp < def.mp_cost {
                            // Not enough MP: ignore for now (stay on menu).
                        } else {
                            let target = def.target;
                            let kind = ActionKind::Skill(def.id.clone());
                            if needs_cursor(target) {
                                let cands = self.candidates(hero, target);
                                cmd.stage = Stage::Target {
                                    pending: Pending { kind, target },
                                    cursor: 0,
                                    candidates: cands,
                                };
                            } else {
                                let targets = self.candidates(hero, target);
                                cmd.planned.push(Action {
                                    actor: hero,
                                    kind,
                                    targets,
                                });
                                cmd.current += 1;
                                cmd.stage = Stage::Root { cursor: 0 };
                            }
                        }
                    }
                }
                CommandResult::Stay
            }
            Stage::Target {
                pending,
                cursor,
                candidates,
            } => {
                if candidates.is_empty() {
                    cmd.stage = Stage::Root { cursor: 0 };
                    return CommandResult::Stay;
                }
                if input.pressed(Button::Up) || input.pressed(Button::Left) {
                    *cursor = (*cursor + candidates.len() - 1) % candidates.len();
                }
                if input.pressed(Button::Down) || input.pressed(Button::Right) {
                    *cursor = (*cursor + 1) % candidates.len();
                }
                if input.pressed(Button::Cancel) {
                    cmd.stage = Stage::Root { cursor: 0 };
                } else if input.pressed(Button::Confirm) {
                    let targets = match pending.target {
                        TargetKind::AllEnemies | TargetKind::AllAllies => candidates.clone(),
                        _ => vec![candidates[*cursor]],
                    };
                    cmd.planned.push(Action {
                        actor: hero,
                        kind: pending.kind.clone(),
                        targets,
                    });
                    cmd.current += 1;
                    cmd.stage = Stage::Root { cursor: 0 };
                }
                CommandResult::Stay
            }
        }
    }

    fn build_execution(&mut self, cmd: &mut Command, reg: &Registry) -> Execute {
        let mut queue = std::mem::take(&mut cmd.planned);

        // Enemy AI plans.
        for &e in &self.living(Side::Enemy) {
            let action = self.plan_enemy(e, reg);
            queue.push(action);
        }

        // Order by speed (desc). Stable enough for a basic JRPG.
        queue.sort_by_key(|a| -self.battlers[a.actor].stats.speed);

        Execute {
            queue,
            idx: 0,
            elapsed: 0.0,
            applied: false,
            banner: String::new(),
            popups: Vec::new(),
        }
    }

    fn plan_enemy(&self, enemy: usize, reg: &Registry) -> Action {
        let b = &self.battlers[enemy];
        // Random AI may use a skill; Basic always attacks.
        let use_skill = matches!(b.ai, EnemyAi::Random) && !b.skills.is_empty();
        // Deterministic-ish selection without borrowing the rng here: pick by hp.
        if use_skill && (b.hp % 2 == 0) {
            if let Some(def) = reg.skill(&b.skills[0]) {
                if b.mp >= def.mp_cost {
                    let targets = self.pick_targets(enemy, def.target);
                    if !targets.is_empty() {
                        return Action {
                            actor: enemy,
                            kind: ActionKind::Skill(def.id.clone()),
                            targets,
                        };
                    }
                }
            }
        }
        let targets = self.pick_targets(enemy, TargetKind::OneEnemy);
        Action {
            actor: enemy,
            kind: ActionKind::Attack,
            targets,
        }
    }

    fn pick_targets(&self, actor: usize, target: TargetKind) -> Vec<usize> {
        let cands = self.candidates(actor, target);
        match target {
            TargetKind::AllEnemies | TargetKind::AllAllies => cands,
            TargetKind::SelfOnly => vec![actor],
            _ => cands.into_iter().take(1).collect(),
        }
    }

    fn update_execute(
        &mut self,
        exec: &mut Execute,
        rng: &mut Rng,
        reg: &Registry,
        dt: f32,
    ) -> bool {
        // Advance popups regardless of state.
        for p in &mut exec.popups {
            p.t += dt;
            p.pos.y -= dt * 14.0;
        }
        exec.popups.retain(|p| p.t < 0.9);

        if exec.idx >= exec.queue.len() {
            // Wait for popups to clear, then finish the round.
            return exec.popups.is_empty();
        }

        // Skip actions whose actor died before acting.
        if !self.battlers[exec.queue[exec.idx].actor].alive() {
            exec.idx += 1;
            exec.elapsed = 0.0;
            exec.applied = false;
            return false;
        }

        exec.elapsed += dt;
        let action = exec.queue[exec.idx].clone();
        let actor = action.actor;
        let dir = self.battlers[actor].facing_dir();

        // Timeline: 0.0 windup, 0.2 impact, 0.55 hold, 0.75 return, 0.9 end.
        let t = exec.elapsed;
        let lunge = 18.0;
        self.battlers[actor].offset.x = if t < 0.2 {
            dir * lunge * (t / 0.2)
        } else if t < 0.55 {
            dir * lunge
        } else if t < 0.75 {
            dir * lunge * (1.0 - (t - 0.55) / 0.2)
        } else {
            0.0
        };

        if t < 0.05 && !exec.applied {
            // Start of action: set banner + play attack anim.
            exec.banner = self.action_banner(&action, reg);
            let clip = self.battlers[actor].sprite.attack.clone();
            self.battlers[actor].anim = Anim::from_clip(&clip, false);
        }

        if t >= 0.2 && !exec.applied {
            exec.applied = true;
            self.apply_action(&action, rng, reg, &mut exec.popups);
        }

        if t >= 0.9 {
            // Restore idle and move to the next action.
            let idle = self.battlers[actor].idle.clone();
            self.battlers[actor].anim = idle;
            self.battlers[actor].offset = Vec2::ZERO;
            exec.idx += 1;
            exec.elapsed = 0.0;
            exec.applied = false;
        }

        false
    }

    fn action_banner(&self, action: &Action, reg: &Registry) -> String {
        let name = &self.battlers[action.actor].name;
        match &action.kind {
            ActionKind::Attack => format!("{name} ATTACKS"),
            ActionKind::Defend => format!("{name} DEFENDS"),
            ActionKind::Skill(id) => {
                let sk = reg.skill(id).map(|s| s.name.as_str()).unwrap_or("SKILL");
                format!("{name}: {sk}")
            }
        }
    }

    fn apply_action(
        &mut self,
        action: &Action,
        rng: &mut Rng,
        reg: &Registry,
        popups: &mut Vec<Popup>,
    ) {
        let actor = action.actor;
        match &action.kind {
            ActionKind::Defend => {
                self.battlers[actor].defending = true;
            }
            ActionKind::Attack => {
                let atk = self.battlers[actor].stats.attack;
                for &tgt in &action.targets {
                    if self.battlers[tgt].alive() {
                        self.strike(tgt, atk, 100, SkillKind::Physical, rng, popups);
                    }
                }
            }
            ActionKind::Skill(id) => {
                let Some(def) = reg.skill(id) else { return };
                let def: SkillDef = def.clone();
                self.battlers[actor].mp = (self.battlers[actor].mp - def.mp_cost).max(0);
                // Retarget dead single-targets to a living one if possible.
                let targets = self.resolve_live_targets(actor, &action.targets, def.target);
                match def.kind {
                    SkillKind::Heal => {
                        let mag = self.battlers[actor].stats.magic;
                        for &tgt in &targets {
                            let heal = (mag * def.power / 100).max(1);
                            let b = &mut self.battlers[tgt];
                            let before = b.hp;
                            b.hp = (b.hp + heal).min(b.max_hp);
                            let gained = b.hp - before;
                            b.flash = 0.25;
                            popups.push(Popup {
                                text: format!("+{gained}"),
                                pos: b.pos() + Vec2::new(0.0, -6.0),
                                t: 0.0,
                                color: color::rgb(120, 240, 140),
                            });
                        }
                    }
                    SkillKind::Physical => {
                        let atk = self.battlers[actor].stats.attack;
                        for &tgt in &targets {
                            if self.battlers[tgt].alive() {
                                self.strike(tgt, atk, def.power, SkillKind::Physical, rng, popups);
                            }
                        }
                    }
                    SkillKind::Magical => {
                        let mag = self.battlers[actor].stats.magic;
                        for &tgt in &targets {
                            if self.battlers[tgt].alive() {
                                self.strike(tgt, mag, def.power, SkillKind::Magical, rng, popups);
                            }
                        }
                    }
                }
            }
        }
    }

    fn resolve_live_targets(
        &self,
        actor: usize,
        targets: &[usize],
        target_kind: TargetKind,
    ) -> Vec<usize> {
        let alive: Vec<usize> = targets
            .iter()
            .copied()
            .filter(|&t| self.battlers[t].alive())
            .collect();
        if !alive.is_empty() {
            return alive;
        }
        // Everything originally targeted is gone; pick fresh candidates.
        match target_kind {
            TargetKind::OneEnemy | TargetKind::OneAlly => self
                .candidates(actor, target_kind)
                .into_iter()
                .take(1)
                .collect(),
            _ => self.candidates(actor, target_kind),
        }
    }

    fn strike(
        &mut self,
        target: usize,
        offense: i32,
        power: i32,
        _kind: SkillKind,
        rng: &mut Rng,
        popups: &mut Vec<Popup>,
    ) {
        let defending = self.battlers[target].defending;
        let defense = self.battlers[target].stats.defense;
        let mut dmg = (offense * power / 100) - defense / 2;
        if dmg < 1 {
            dmg = 1;
        }
        let variance = rng.range(88, 112);
        dmg = (dmg * variance / 100).max(1);
        if defending {
            dmg = (dmg * 2 / 3).max(1);
        }

        let b = &mut self.battlers[target];
        b.hp = (b.hp - dmg).max(0);
        b.flash = 0.3;
        popups.push(Popup {
            text: format!("{dmg}"),
            pos: b.pos() + Vec2::new(0.0, -6.0),
            t: 0.0,
            color: color::rgb(255, 226, 120),
        });
    }

    // ---- Rendering ----------------------------------------------------------

    pub fn draw(&mut self, r: &mut Renderer, reg: &Registry) {
        draw_background(r);

        // Draw battlers back-to-front (enemies first so heroes overlap nicely).
        let mut order: Vec<usize> = (0..self.battlers.len()).collect();
        order.sort_by(|&a, &b| {
            self.battlers[a]
                .pos()
                .y
                .partial_cmp(&self.battlers[b].pos().y)
                .unwrap()
        });
        for &i in &order {
            self.draw_battler(r, i);
        }

        // UI panels.
        self.draw_party_panel(r);

        match &self.state {
            State::Command(cmd) => self.draw_command(r, cmd, reg),
            State::Execute(exec) => {
                if !exec.banner.is_empty() {
                    draw_banner(r, &exec.banner);
                }
                for p in &exec.popups {
                    let alpha = (1.0 - p.t / 0.9).clamp(0.0, 1.0);
                    let c = [p.color[0], p.color[1], p.color[2], alpha];
                    r.draw_text_centered(&p.text, p.pos.x, p.pos.y, 1.0, c);
                }
            }
            State::Result { win, .. } => {
                let (msg, col) = if *win {
                    ("VICTORY!", color::rgb(255, 230, 120))
                } else {
                    ("DEFEAT...", color::rgb(230, 120, 120))
                };
                draw_banner(r, msg);
                r.draw_text_centered(
                    "PRESS ENTER",
                    VIRTUAL_W / 2.0,
                    VIRTUAL_H / 2.0 + 12.0,
                    1.0,
                    col,
                );
            }
            State::Intro(_) => {
                r.draw_text_centered(
                    &format!("{} APPEARS!", self.encounter_name.to_uppercase()),
                    VIRTUAL_W / 2.0,
                    24.0,
                    1.0,
                    color::WHITE,
                );
            }
        }
    }

    fn draw_battler(&self, r: &mut Renderer, i: usize) {
        let b = &self.battlers[i];
        if !b.present() {
            return;
        }
        let pos = b.pos();
        let (dw, dh) = (b.sprite.draw_w, b.sprite.draw_h);
        let dest = [pos.x - dw / 2.0, pos.y - dh, dw, dh];
        let src = b.anim.src(&b.sprite);

        // Shadow.
        r.draw_rect(
            pos.x - dw * 0.28,
            pos.y - 3.0,
            dw * 0.56,
            5.0,
            color::rgba(0, 0, 0, 90),
        );

        let tint = if b.flash > 0.0 {
            let k = (b.flash / 0.3).clamp(0.0, 1.0);
            [1.0, 1.0 - k * 0.6, 1.0 - k * 0.6, b.fade]
        } else if !b.alive() {
            [0.5, 0.5, 0.6, b.fade]
        } else {
            [1.0, 1.0, 1.0, b.fade]
        };
        r.draw_sprite(b.texture, dest, src, b.flip_x(), tint);
    }

    fn draw_party_panel(&self, r: &mut Renderer) {
        let heroes: Vec<usize> = self
            .battlers
            .iter()
            .enumerate()
            .filter(|(_, b)| b.side == Side::Hero)
            .map(|(i, _)| i)
            .collect();

        let panel_h = 8.0 + heroes.len() as f32 * 16.0;
        let y0 = VIRTUAL_H - panel_h;
        r.draw_rect(0.0, y0, 150.0, panel_h, color::rgba(12, 14, 28, 220));
        r.draw_rect_outline(0.0, y0, 150.0, panel_h, 1.0, color::rgba(80, 90, 140, 255));

        for (row, &i) in heroes.iter().enumerate() {
            let b = &self.battlers[i];
            let y = y0 + 5.0 + row as f32 * 16.0;
            let name_col = if b.alive() {
                color::WHITE
            } else {
                color::rgb(150, 90, 90)
            };
            r.draw_text(&b.name, 6.0, y, 1.0, name_col);
            // HP bar.
            bar(
                r,
                62.0,
                y + 1.0,
                44.0,
                4.0,
                b.hp,
                b.max_hp,
                color::rgb(80, 210, 90),
            );
            // MP bar.
            bar(
                r,
                62.0,
                y + 6.0,
                44.0,
                3.0,
                b.mp,
                b.max_mp,
                color::rgb(90, 150, 240),
            );
            r.draw_text(
                &format!("{}", b.hp.max(0)),
                110.0,
                y,
                1.0,
                color::rgb(200, 220, 200),
            );
        }
    }

    fn draw_command(&self, r: &mut Renderer, cmd: &Command, reg: &Registry) {
        if cmd.current >= cmd.order.len() {
            return;
        }
        let hero = cmd.order[cmd.current];
        let hero_name = &self.battlers[hero].name;

        // Prompt.
        r.draw_text(
            &format!("{hero_name}'S TURN"),
            6.0,
            6.0,
            1.0,
            color::rgb(230, 230, 160),
        );

        match &cmd.stage {
            Stage::Root { cursor } => {
                menu_box(r, 160.0, 118.0, 90.0, &ROOT_ITEMS, *cursor);
            }
            Stage::Skill { cursor } => {
                let mut items: Vec<String> = self.battlers[hero]
                    .skills
                    .iter()
                    .map(|id| {
                        let s = reg.skill(id);
                        match s {
                            Some(s) => format!("{}  {}MP", s.name, s.mp_cost),
                            None => id.clone(),
                        }
                    })
                    .collect();
                items.push("BACK".to_string());
                let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
                menu_box(r, 150.0, 96.0, 120.0, &refs, *cursor);
            }
            Stage::Target {
                candidates, cursor, ..
            } => {
                r.draw_text("SELECT TARGET", 6.0, 18.0, 1.0, color::rgb(255, 210, 120));
                if let Some(&tgt) = candidates.get(*cursor) {
                    let p = self.battlers[tgt].pos();
                    let dh = self.battlers[tgt].sprite.draw_h;
                    // Blinking cursor above the target.
                    r.draw_text_centered("v", p.x, p.y - dh - 10.0, 1.0, color::rgb(255, 240, 120));
                    r.draw_text_centered(
                        &self.battlers[tgt].name,
                        p.x,
                        p.y - dh - 20.0,
                        1.0,
                        color::WHITE,
                    );
                }
            }
        }
    }
}

// ---- Free helpers -----------------------------------------------------------

enum CommandResult {
    Stay,
    Execute(Execute),
}

fn dummy_command() -> Command {
    Command {
        order: vec![],
        current: 0,
        planned: vec![],
        stage: Stage::Root { cursor: 0 },
    }
}

fn dummy_execute() -> Execute {
    Execute {
        queue: vec![],
        idx: 0,
        elapsed: 0.0,
        applied: false,
        banner: String::new(),
        popups: vec![],
    }
}

fn needs_cursor(t: TargetKind) -> bool {
    matches!(t, TargetKind::OneEnemy | TargetKind::OneAlly)
}

fn menu_move(cursor: &mut usize, len: usize, input: &Input) {
    if len == 0 {
        return;
    }
    if input.pressed(Button::Up) {
        *cursor = (*cursor + len - 1) % len;
    }
    if input.pressed(Button::Down) {
        *cursor = (*cursor + 1) % len;
    }
}

fn hero_home(slot: usize) -> Vec2 {
    // Diagonal column on the left, facing right.
    Vec2::new(70.0 - slot as f32 * 8.0, 92.0 + slot as f32 * 24.0)
}

fn enemy_home(slot: usize) -> Vec2 {
    Vec2::new(228.0 + (slot % 2) as f32 * 20.0, 74.0 + slot as f32 * 26.0)
}

fn bar(r: &mut Renderer, x: f32, y: f32, w: f32, h: f32, value: i32, max: i32, fill: Color) {
    r.draw_rect(x, y, w, h, color::rgba(0, 0, 0, 180));
    let frac = if max > 0 {
        (value.max(0) as f32 / max as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    r.draw_rect(x, y, w * frac, h, fill);
    r.draw_rect_outline(x, y, w, h, 0.5, color::rgba(200, 200, 220, 200));
}

fn menu_box(r: &mut Renderer, x: f32, y: f32, w: f32, items: &[&str], cursor: usize) {
    let h = 6.0 + items.len() as f32 * 11.0;
    r.draw_rect(x, y, w, h, color::rgba(14, 16, 32, 235));
    r.draw_rect_outline(x, y, w, h, 1.0, color::rgba(90, 110, 170, 255));
    for (i, item) in items.iter().enumerate() {
        let iy = y + 4.0 + i as f32 * 11.0;
        if i == cursor {
            r.draw_rect(
                x + 2.0,
                iy - 1.0,
                w - 4.0,
                10.0,
                color::rgba(60, 80, 150, 220),
            );
            r.draw_text(">", x + 3.0, iy, 1.0, color::rgb(255, 240, 150));
        }
        r.draw_text(item, x + 11.0, iy, 1.0, color::WHITE);
    }
}

fn draw_banner(r: &mut Renderer, text: &str) {
    let w = r.text_width(text, 1.0) + 16.0;
    let x = (VIRTUAL_W - w) / 2.0;
    r.draw_rect(x, 10.0, w, 16.0, color::rgba(10, 12, 26, 230));
    r.draw_rect_outline(x, 10.0, w, 16.0, 1.0, color::rgba(90, 110, 170, 255));
    r.draw_text_centered(text, VIRTUAL_W / 2.0, 14.0, 1.0, color::WHITE);
}

fn draw_background(r: &mut Renderer) {
    // Simple two-tone sky over ground.
    r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(28, 24, 48));
    r.draw_rect(0.0, 60.0, VIRTUAL_W, 60.0, color::rgb(40, 34, 66));
    r.draw_rect(
        0.0,
        118.0,
        VIRTUAL_W,
        VIRTUAL_H - 118.0,
        color::rgb(46, 40, 40),
    );
    // A faint horizon band.
    r.draw_rect(0.0, 116.0, VIRTUAL_W, 3.0, color::rgba(90, 70, 100, 160));
}
