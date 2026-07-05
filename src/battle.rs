//! Turn-based battle scene.
//!
//! Flow per round: every living unit takes a turn in initiative order (effective
//! speed plus a little randomness), and each turn resolves immediately. A unit
//! well above the field's average speed may earn a second turn in the round.
//!   1. Step to the next unit in the initiative-ordered turn list.
//!   2. A hero's turn opens the command menu; an enemy's turn auto-plans via its
//!      [`EnemyAi`]. Either way the chosen action plays out at once, with a
//!      little movement/impact animation and floating damage numbers.
//!   3. When the turn list is spent, status effects tick; then check for
//!      victory/defeat, otherwise start a new round.
//!
//! The scene is data-driven: heroes come from the [`Party`] and enemies from an
//! encounter in the [`Registry`], so more party members or new enemies work
//! with zero changes here.

use std::collections::HashMap;

use glam::Vec2;

use crate::data::{
    BattlerSprite, EnemyAi, EquipmentDef, Registry, SkillDef, SkillKind, Stats, StatusDef,
    TargetKind,
};
use crate::input::{Button, Controllers, Input};
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

// ---- Status effects ---------------------------------------------------------

/// A status condition currently afflicting a battler, with how many rounds it
/// has left. The behaviour (damage per turn, stat shifts, colour) lives in the
/// data-driven [`crate::data::StatusDef`] this `id` points at, so new effects are
/// content, not code.
struct ActiveStatus {
    id: String,
    remaining: i32,
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
    /// Equipped item ids (into the registry's `equipment`), for the gear panel.
    weapon: Option<String>,
    armor: Option<String>,
    /// Derived combat attributes (base + equipment), in percent.
    crit: i32,
    accuracy: i32,
    evasion: i32,
    sprite: BattlerSprite,
    texture: TextureHandle,
    idle: Anim,
    anim: Anim,
    defending: bool,
    /// Status conditions currently in effect (burn, slow, …). Drives per-round
    /// ticks and effective-stat adjustments.
    statuses: Vec<ActiveStatus>,
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

// ---- Action timing (attack / block timed hits) ------------------------------

// SMRPG-style timed hits: there is *no* on-screen indicator. The window is woven
// into the strike animation itself — you have to read the swing and tap Confirm
// as the blow connects. The weapon connects at `STRIKE_CONNECT` (the peak of the
// lunge); a tap within `STRIKE_PERFECT` of it is "on time" → the full bonus,
// within `STRIKE_GOOD` is a hair early/late → the lesser bonus, and anything
// further out (or no tap) misses. Tapping *before* the animation gets you nothing
// — an early panic tap simply whiffs.
const STRIKE_CONNECT: f32 = 0.2;
const STRIKE_PERFECT: f32 = 0.05;
const STRIKE_GOOD: f32 = 0.13;
/// When the blow actually lands — just after the timing window shuts, so the tap
/// is already known. Kept in step with the lunge's hold (see the timeline in
/// [`Battle::update_execute`]).
const STRIKE_APPLY: f32 = STRIKE_CONNECT + STRIKE_GOOD;

#[derive(Copy, Clone, PartialEq, Eq)]
enum TimingKind {
    /// A hero's own strike: a well-timed tap *adds* damage (+50% / +100%).
    Attack,
    /// A hero bracing for an incoming blow: a well-timed tap *subtracts* the
    /// enemy's damage (−50% / −100%).
    Block,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum TimingResult {
    Miss,
    Good,
    Perfect,
}

/// The timed-hit tracker for the current action. There is no bar and no marker:
/// the `presser` gets exactly **one** clean Confirm tap, recorded as the elapsed
/// *animation* time [`press_t`](Self::press_t), and how close that is to
/// [`STRIKE_CONNECT`] decides the [`TimingResult`] and its damage [`HitMod`].
/// Mashing the button ([`presses`](Self::presses) > 1) earns nothing — you can't
/// hammer your way to a perfect.
struct TimedHit {
    kind: TimingKind,
    /// The hero (battler index) whose controller drives the tap: the attacker for
    /// [`TimingKind::Attack`], the defender for [`TimingKind::Block`].
    presser: usize,
    /// Elapsed animation time of the *first* tap, for scoring (`None` until the
    /// presser taps — a tap only counts while the strike is playing out).
    press_t: Option<f32>,
    /// How many taps landed during the swing. Only a single, deliberate tap is
    /// eligible: zero is a plain miss, and two or more is button-mashing, which
    /// forfeits the bonus.
    presses: u32,
}

impl TimedHit {
    fn new(kind: TimingKind, presser: usize) -> Self {
        TimedHit {
            kind,
            presser,
            press_t: None,
            presses: 0,
        }
    }

    /// How well the tap landed, from its distance to the connect moment. Anything
    /// but a single clean tap misses — no tap, or mashing.
    fn result(&self) -> TimingResult {
        if self.presses != 1 {
            return TimingResult::Miss;
        }
        let off = (self.press_t.unwrap_or(f32::INFINITY) - STRIKE_CONNECT).abs();
        if off <= STRIKE_PERFECT {
            TimingResult::Perfect
        } else if off <= STRIKE_GOOD {
            TimingResult::Good
        } else {
            TimingResult::Miss
        }
    }

    /// The damage modifier this timing produces.
    fn hit_mod(&self) -> HitMod {
        match self.kind {
            TimingKind::Attack => HitMod {
                attack_mult: match self.result() {
                    TimingResult::Perfect => 2.0,
                    TimingResult::Good => 1.5,
                    TimingResult::Miss => 1.0,
                },
                block_reduce: 0.0,
            },
            TimingKind::Block => HitMod {
                attack_mult: 1.0,
                block_reduce: match self.result() {
                    TimingResult::Perfect => 1.0,
                    TimingResult::Good => 0.5,
                    TimingResult::Miss => 0.0,
                },
            },
        }
    }

    /// Label + colour for the floating feedback shown when the blow lands. A miss
    /// shows nothing — you simply don't earn the bonus, no scolding text.
    fn flash(&self) -> Option<(&'static str, Color)> {
        match (self.kind, self.result()) {
            (_, TimingResult::Miss) => None,
            (TimingKind::Attack, TimingResult::Good) => {
                Some(("GOOD  +50%", color::rgb(255, 226, 120)))
            }
            (TimingKind::Attack, TimingResult::Perfect) => {
                Some(("PERFECT  +100%", color::rgb(255, 150, 60)))
            }
            (TimingKind::Block, TimingResult::Good) => {
                Some(("BLOCK  -50%", color::rgb(140, 200, 255)))
            }
            (TimingKind::Block, TimingResult::Perfect) => {
                Some(("BLOCK  -100%", color::rgb(120, 240, 140)))
            }
        }
    }
}

/// A damage modifier from an action's timing: an offensive multiplier (the
/// attacker's own attack-timing) and a fraction of damage removed by a block (the
/// defender's block-timing). For any one action exactly one is non-neutral.
#[derive(Copy, Clone)]
struct HitMod {
    attack_mult: f32,
    block_reduce: f32,
}

impl HitMod {
    const NEUTRAL: HitMod = HitMod {
        attack_mult: 1.0,
        block_reduce: 0.0,
    };
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
    /// The single hero whose turn it is (battler index). Their action resolves
    /// immediately once chosen, rather than being batched with the rest.
    hero: usize,
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
    /// Whether the current action's start-up (banner + arming its timed hit) has
    /// run. Reset for each action in the queue.
    started: bool,
    /// The timed-hit tracker for the current action, if it has one. Records the
    /// presser's tap as the strike animation plays. `None` for actions with no
    /// timing (heals, defends, unblockable blows).
    timed: Option<TimedHit>,
    /// Damage modifier banked from `timed`, applied when the blow lands.
    hit_mod: HitMod,
    /// True for the tail-of-round bookkeeping step (an empty-queue execute) that
    /// resolves status ticks (burn damage, etc.) once before the next round.
    end_of_round: bool,
    /// Guards the one-shot status tick on an `end_of_round` execute.
    statuses_ticked: bool,
    banner: String,
    popups: Vec<Popup>,
}

const ROOT_ITEMS: [&str; 3] = ["ATTACK", "SKILL", "DEFEND"];

// Combat rolls. Hit chance rises with the attacker's accuracy and speed and
// falls with the target's evasion and speed; crit adds +50% damage.
const BASE_HIT: i32 = 92;
const MIN_HIT: i32 = 40;
const MAX_HIT: i32 = 99;
const BASE_CRIT: i32 = 5;
const MAX_CRIT: i32 = 90;

pub struct Battle {
    battlers: Vec<Battler>,
    state: State,
    encounter_name: String,
    /// This round's turn order: all living battlers (heroes and enemies) by
    /// speed, stepped through one at a time so each unit acts the moment its
    /// turn comes up.
    turn_order: Vec<usize>,
    turn_idx: usize,
    /// Icon texture for each equipped item id, resolved once at construction.
    icons: HashMap<String, TextureHandle>,
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
        let mut icons: HashMap<String, TextureHandle> = HashMap::new();

        // Heroes on the left.
        let living: Vec<usize> = (0..party.members.len())
            .filter(|&i| party.members[i].is_alive())
            .collect();
        for (slot, &pi) in living.iter().enumerate() {
            let m = &party.members[pi];
            let texture = cache.get(renderer, &m.sprite.texture);
            let home = hero_home(slot);
            // Fold equipment into the battler's stats and combat attributes.
            let eq = reg.equipped(&m.stats, m.weapon.as_deref(), m.armor.as_deref());
            load_item_icons(renderer, cache, reg, &mut icons, [&m.weapon, &m.armor]);
            battlers.push(Battler {
                name: m.name.clone(),
                side: Side::Hero,
                party_index: Some(pi),
                stats: eq.stats,
                hp: m.hp,
                max_hp: m.stats.max_hp,
                mp: m.mp,
                max_mp: m.stats.max_mp,
                skills: m.skills.clone(),
                weapon: m.weapon.clone(),
                armor: m.armor.clone(),
                crit: eq.crit,
                accuracy: eq.accuracy,
                evasion: eq.evasion,
                idle: Anim::from_clip(&m.sprite.idle, true),
                anim: Anim::from_clip(&m.sprite.idle, true),
                sprite: m.sprite.clone(),
                texture,
                defending: false,
                statuses: Vec::new(),
                ai: EnemyAi::Basic,
                xp: 0,
                gold: 0,
                home,
                offset: Vec2::ZERO,
                flash: 0.0,
                fade: 1.0,
            });
        }

        // Enemies on the right, from the encounter. Each is scaled to the party's
        // level so foes keep pace with the heroes' growth instead of becoming
        // one-shot fodder in the later regions (identity at party level 1).
        let enc = reg
            .encounter(encounter_id)
            .unwrap_or_else(|| panic!("unknown encounter '{encounter_id}'"));
        let name = encounter_id.to_string();
        let party_level = party.level();
        let scale = crate::data::enemy_scale(party_level);
        for (slot, eid) in enc.enemies.iter().enumerate() {
            let def = reg
                .enemy(eid)
                .unwrap_or_else(|| panic!("unknown enemy '{eid}'"));
            let texture = cache.get(renderer, &def.sprite.texture);
            let home = enemy_home(slot);
            let stats = def.stats.scaled_to(party_level);
            let eq = reg.equipped(&stats, def.weapon.as_deref(), def.armor.as_deref());
            load_item_icons(renderer, cache, reg, &mut icons, [&def.weapon, &def.armor]);
            battlers.push(Battler {
                name: def.name.clone(),
                side: Side::Enemy,
                party_index: None,
                stats: eq.stats,
                hp: stats.max_hp,
                max_hp: stats.max_hp,
                mp: stats.max_mp,
                max_mp: stats.max_mp,
                skills: def.skills.clone(),
                weapon: def.weapon.clone(),
                armor: def.armor.clone(),
                crit: eq.crit,
                accuracy: eq.accuracy,
                evasion: eq.evasion,
                idle: Anim::from_clip(&def.sprite.idle, true),
                anim: Anim::from_clip(&def.sprite.idle, true),
                sprite: def.sprite.clone(),
                texture,
                defending: false,
                statuses: Vec::new(),
                ai: def.ai(),
                // Rewards scale with the same factor, so tougher scaled foes are
                // worth proportionally more XP and gold.
                xp: def.xp * scale / 100,
                gold: def.gold * scale / 100,
                home,
                offset: Vec2::ZERO,
                flash: 0.0,
                fade: 1.0,
            });
        }

        Battle {
            battlers,
            state: State::Intro(0.6),
            turn_order: Vec::new(),
            turn_idx: 0,
            encounter_name: name,
            icons,
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

    /// A battler's effective combat stats: base+equipment with every active
    /// status's [`stat_mods`](crate::data::StatusDef::stat_mods) folded in. This
    /// is the single place status stat shifts (slow, weaken, …) take effect, so
    /// they apply everywhere — turn order, damage, hit rolls — and revert on their
    /// own once the status list no longer contains them.
    fn eff_stats(&self, reg: &Registry, i: usize) -> Stats {
        let b = &self.battlers[i];
        let mut s = b.stats.clone();
        for st in &b.statuses {
            if let Some(def) = reg.status(&st.id) {
                s.attack += def.stat_mods.attack;
                s.defense += def.stat_mods.defense;
                s.magic += def.stat_mods.magic;
                s.speed += def.stat_mods.speed;
            }
        }
        s
    }

    // ---- Update -------------------------------------------------------------

    pub fn update(
        &mut self,
        controllers: &Controllers,
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
                    self.state = self.begin_round(rng, reg);
                } else {
                    self.state = State::Intro(*timer);
                }
                None
            }
            State::Command(cmd) => {
                let next = self.update_command(cmd, controllers, reg);
                match next {
                    CommandResult::Stay => {
                        self.state = State::Command(std::mem::replace(cmd, dummy_command()));
                    }
                    CommandResult::Execute(exec) => self.state = State::Execute(exec),
                }
                None
            }
            State::Execute(exec) => {
                let done = self.update_execute(exec, controllers, rng, reg, dt);
                if done {
                    // An `end_of_round` execute finishes the round (statuses have
                    // ticked) and opens the next one; any other execute was a
                    // single unit's action, so hand off to whoever is next.
                    self.state = if exec.end_of_round {
                        self.begin_round(rng, reg)
                    } else {
                        self.after_action(rng, reg)
                    };
                } else {
                    self.state = State::Execute(std::mem::replace(exec, dummy_execute()));
                }
                None
            }
            State::Result { win, timer } => {
                *timer -= dt;
                if *timer <= 0.0 && controllers.shared().any_pressed() || *timer <= -3.0 {
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

    /// Open a fresh round: clear defends, rebuild the speed-ordered turn list of
    /// every living battler, and hand control to whoever moves first.
    fn begin_round(&mut self, rng: &mut Rng, reg: &Registry) -> State {
        // A tail-of-round status tick may have wiped a side, so check before
        // opening a new round.
        if let Some(win) = self.battle_over() {
            return State::Result { win, timer: 1.6 };
        }
        for b in &mut self.battlers {
            b.defending = false;
        }
        // Turn order is speed-based with a dash of randomness: each living unit
        // rolls an initiative of its effective speed plus a small jitter, so the
        // swift usually act first but the exact order shifts round to round and
        // near-equal units trade the lead. Rolls are done once per unit here (not
        // inside the sort) so every entry keeps a single stable initiative.
        let living: Vec<usize> = (0..self.battlers.len())
            .filter(|&i| self.battlers[i].alive())
            .collect();
        // "Relatively fast" is measured against the field's average speed: the
        // further a unit sits above it, the better its odds of a bonus turn.
        let avg = living
            .iter()
            .map(|&i| self.eff_stats(reg, i).speed)
            .sum::<i32>() as f32
            / living.len().max(1) as f32;

        let mut init: Vec<(usize, i32)> = Vec::new();
        for &i in &living {
            let speed = self.eff_stats(reg, i).speed;
            init.push((i, speed + rng.range(0, 3)));
            // A unit well above the average gets a chance at a second turn, rolled
            // its own initiative so it slots back into the order by speed. The
            // edge scales with how far above average it is, capped so even a
            // blazing-fast unit doesn't get a guaranteed double.
            let edge = speed as f32 - avg;
            if edge > 0.0 && rng.chance((edge / avg).min(0.6)) {
                init.push((i, speed + rng.range(0, 3)));
            }
        }
        init.sort_by_key(|&(_, roll)| -roll);
        self.turn_order = init.into_iter().map(|(i, _)| i).collect();
        self.turn_idx = 0;
        self.advance(rng, reg)
    }

    /// Win (`Some(true)`) if all enemies are down, loss (`Some(false)`) if all
    /// heroes are, else `None` — the fight goes on.
    fn battle_over(&self) -> Option<bool> {
        if !self.enemies_alive() {
            Some(true)
        } else if !self.heroes_alive() {
            Some(false)
        } else {
            None
        }
    }

    /// Resolve the state after one unit's action: end the fight if a side is
    /// wiped, otherwise pass the turn to the next unit.
    fn after_action(&mut self, rng: &mut Rng, reg: &Registry) -> State {
        if let Some(win) = self.battle_over() {
            return State::Result { win, timer: 1.6 };
        }
        self.advance(rng, reg)
    }

    /// Step to the next living unit in the turn order. A hero gets the command
    /// menu; an enemy plans and acts at once. When the order is spent, run the
    /// end-of-round status tick.
    fn advance(&mut self, rng: &mut Rng, reg: &Registry) -> State {
        while self.turn_idx < self.turn_order.len() {
            let actor = self.turn_order[self.turn_idx];
            self.turn_idx += 1;
            if !self.battlers[actor].alive() {
                continue; // fell earlier this round — skip its turn
            }
            return match self.battlers[actor].side {
                Side::Hero => State::Command(Command {
                    hero: actor,
                    stage: Stage::Root { cursor: 0 },
                }),
                Side::Enemy => {
                    let action = self.plan_enemy(actor, rng, reg);
                    State::Execute(self.action_exec(action))
                }
            };
        }
        State::Execute(self.end_of_round_exec())
    }

    /// An execute that plays out a single unit's action.
    fn action_exec(&self, action: Action) -> Execute {
        Execute {
            queue: vec![action],
            idx: 0,
            elapsed: 0.0,
            applied: false,
            started: false,
            timed: None,
            hit_mod: HitMod::NEUTRAL,
            end_of_round: false,
            statuses_ticked: false,
            banner: String::new(),
            popups: Vec::new(),
        }
    }

    /// An empty-queue execute that only ticks end-of-round statuses.
    fn end_of_round_exec(&self) -> Execute {
        Execute {
            queue: Vec::new(),
            idx: 0,
            elapsed: 0.0,
            applied: false,
            started: false,
            timed: None,
            hit_mod: HitMod::NEUTRAL,
            end_of_round: true,
            statuses_ticked: false,
            banner: String::new(),
            popups: Vec::new(),
        }
    }

    fn update_command(
        &mut self,
        cmd: &mut Command,
        controllers: &Controllers,
        reg: &Registry,
    ) -> CommandResult {
        let hero = cmd.hero;
        // Each hero is commanded by the gamepad assigned to their party slot;
        // with one controller (or the keyboard) `player` falls back to the shared
        // input, so a lone player still commands every hero in turn.
        let input = controllers.player(self.battlers[hero].party_index.unwrap_or(0));

        match &mut cmd.stage {
            Stage::Root { cursor } => {
                menu_move(cursor, ROOT_ITEMS.len(), input);
                // No "back": the previous unit has already acted this round, so a
                // committed turn can't be taken back.
                if input.pressed(Button::Confirm) {
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
                            return CommandResult::Execute(self.action_exec(Action {
                                actor: hero,
                                kind: ActionKind::Defend,
                                targets: vec![],
                            }));
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
                                return CommandResult::Execute(self.action_exec(Action {
                                    actor: hero,
                                    kind,
                                    targets,
                                }));
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
                    return CommandResult::Execute(self.action_exec(Action {
                        actor: hero,
                        kind: pending.kind.clone(),
                        targets,
                    }));
                }
                CommandResult::Stay
            }
        }
    }

    fn plan_enemy(&self, enemy: usize, rng: &mut Rng, reg: &Registry) -> Action {
        let b = &self.battlers[enemy];
        // Random AI may use a skill; Basic always attacks.
        let use_skill = matches!(b.ai, EnemyAi::Random) && !b.skills.is_empty();
        // Deterministic-ish selection without borrowing the rng here: cast on even
        // HP, and rotate which known skill by HP so multi-skill foes (e.g. a demon
        // with FIREBALL and CLAW) mix it up. Falls through to the first affordable
        // one with a valid target.
        if use_skill && (b.hp % 2 == 0) {
            let n = b.skills.len();
            let start = (b.hp as usize / 2) % n;
            for k in 0..n {
                let Some(def) = reg.skill(&b.skills[(start + k) % n]) else {
                    continue;
                };
                if b.mp >= def.mp_cost {
                    let targets = self.pick_targets(enemy, def.target, rng);
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
        let targets = self.pick_targets(enemy, TargetKind::OneEnemy, rng);
        Action {
            actor: enemy,
            kind: ActionKind::Attack,
            targets,
        }
    }

    fn pick_targets(&self, actor: usize, target: TargetKind, rng: &mut Rng) -> Vec<usize> {
        let cands = self.candidates(actor, target);
        match target {
            TargetKind::AllEnemies | TargetKind::AllAllies => cands,
            TargetKind::SelfOnly => vec![actor],
            // Single target: pick a random living candidate so foes spread their
            // attacks around instead of always focusing the frontmost hero.
            _ => match cands.len() {
                0 => vec![],
                n => vec![cands[rng.range(0, n as i32 - 1) as usize]],
            },
        }
    }

    fn update_execute(
        &mut self,
        exec: &mut Execute,
        controllers: &Controllers,
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
            // The action (if any) has resolved. At the tail of a round, tick
            // status effects once (burn damage, regen, …). Then wait for the
            // damage numbers to clear before handing off.
            if exec.end_of_round && !exec.statuses_ticked {
                exec.statuses_ticked = true;
                self.tick_statuses(reg, &mut exec.popups);
            }
            return exec.popups.is_empty();
        }

        // Skip actions whose actor died before acting.
        if !self.battlers[exec.queue[exec.idx].actor].alive() {
            self.next_action(exec);
            return false;
        }

        let action = exec.queue[exec.idx].clone();
        let actor = action.actor;

        // Action start-up: announce it and, if it warrants one, arm the timed hit
        // (a hero's attack, or a blockable enemy blow on a hero). No prompt or bar
        // is shown — the window lives inside the swing below.
        if !exec.started {
            exec.started = true;
            exec.banner = self.action_banner(&action, reg);
            exec.timed = self.make_timed(&action, reg);
            exec.hit_mod = HitMod::NEUTRAL;
        }

        // Movement + impact timeline: 0.0 windup, STRIKE_CONNECT peak/connect,
        // STRIKE_APPLY the blow lands, 0.55 hold, 0.75 return, 0.9 end. The timed
        // hit is woven in: the presser must tap Confirm as the swing connects, and
        // the blow lands scaled by the `hit_mod` the tap earned.
        if exec.elapsed == 0.0 {
            let clip = self.battlers[actor].sprite.attack.clone();
            self.battlers[actor].anim = Anim::from_clip(&clip, false);
        }
        exec.elapsed += dt;
        let t = exec.elapsed;

        // Tally the presser's taps as they happen in the swing: the first fixes
        // the timing, and any beyond it mark this as mashing (which forfeits the
        // bonus in `TimedHit::result`).
        if let Some(th) = &mut exec.timed {
            let pi = self.battlers[th.presser].party_index.unwrap_or(0);
            if controllers.player(pi).pressed(Button::Confirm) {
                th.press_t.get_or_insert(t);
                th.presses += 1;
            }
        }

        let dir = self.battlers[actor].facing_dir();
        let lunge = 18.0;
        self.battlers[actor].offset.x = if t < STRIKE_CONNECT {
            dir * lunge * (t / STRIKE_CONNECT)
        } else if t < 0.55 {
            dir * lunge
        } else if t < 0.75 {
            dir * lunge * (1.0 - (t - 0.55) / 0.2)
        } else {
            0.0
        };

        if t >= STRIKE_APPLY && !exec.applied {
            exec.applied = true;
            // The timing window has closed: lock in the tap's modifier and float
            // its label as after-the-fact feedback (a miss shows nothing).
            if let Some(th) = &exec.timed {
                exec.hit_mod = th.hit_mod();
                if let Some((label, col)) = th.flash() {
                    let shown = action
                        .targets
                        .iter()
                        .copied()
                        .find(|&x| self.battlers[x].alive())
                        .unwrap_or(actor);
                    let pos = self.battlers[shown].pos() + Vec2::new(0.0, -28.0);
                    exec.popups.push(Popup {
                        text: label.to_string(),
                        pos,
                        t: 0.0,
                        color: col,
                    });
                }
            }
            self.apply_action(&action, exec.hit_mod, rng, reg, &mut exec.popups);
        }

        if t >= 0.9 {
            // Restore idle and move to the next action.
            let idle = self.battlers[actor].idle.clone();
            self.battlers[actor].anim = idle;
            self.battlers[actor].offset = Vec2::ZERO;
            self.next_action(exec);
        }

        false
    }

    /// Advance the execute queue to the next action, clearing per-action state.
    fn next_action(&self, exec: &mut Execute) {
        exec.idx += 1;
        exec.elapsed = 0.0;
        exec.applied = false;
        exec.started = false;
        exec.timed = None;
        exec.hit_mod = HitMod::NEUTRAL;
    }

    /// Arm the timed hit for an action, if it warrants one. A hero's damaging
    /// action gets an *attack* window (a well-timed tap adds damage); a *blockable*
    /// enemy blow on a hero gets a *block* window (a well-timed tap subtracts
    /// damage). Heals, defends, and unblockable attacks get none.
    fn make_timed(&self, action: &Action, reg: &Registry) -> Option<TimedHit> {
        // Whether the action deals damage, and (if it lands on a hero) whether it
        // ignores blocks.
        let (damages, unblockable) = match &action.kind {
            ActionKind::Attack => (true, false), // a plain swing is blockable
            ActionKind::Defend => (false, false),
            ActionKind::Skill(id) => match reg.skill(id) {
                Some(def) => (!matches!(def.kind, SkillKind::Heal), def.is_unblockable()),
                None => (false, false),
            },
        };
        if !damages || !action.targets.iter().any(|&t| self.battlers[t].alive()) {
            return None;
        }

        match self.battlers[action.actor].side {
            // Hero striking foes: time the swing for bonus damage.
            Side::Hero => Some(TimedHit::new(TimingKind::Attack, action.actor)),
            // Enemy striking heroes: brace to block, unless the blow is unblockable.
            Side::Enemy if !unblockable => {
                let defender =
                    action.targets.iter().copied().find(|&t| {
                        self.battlers[t].alive() && self.battlers[t].side == Side::Hero
                    })?;
                Some(TimedHit::new(TimingKind::Block, defender))
            }
            Side::Enemy => None,
        }
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
        hit_mod: HitMod,
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
                let atk = self.eff_stats(reg, actor).attack;
                for &tgt in &action.targets {
                    if self.battlers[tgt].alive() {
                        self.strike(actor, tgt, atk, 100, hit_mod, reg, rng, popups);
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
                        let mag = self.eff_stats(reg, actor).magic;
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
                        let atk = self.eff_stats(reg, actor).attack;
                        for &tgt in &targets {
                            if self.battlers[tgt].alive()
                                && self
                                    .strike(actor, tgt, atk, def.power, hit_mod, reg, rng, popups)
                            {
                                self.inflict_all(reg, tgt, &def.inflicts, popups);
                            }
                        }
                    }
                    SkillKind::Magical => {
                        let mag = self.eff_stats(reg, actor).magic;
                        for &tgt in &targets {
                            if self.battlers[tgt].alive()
                                && self
                                    .strike(actor, tgt, mag, def.power, hit_mod, reg, rng, popups)
                            {
                                self.inflict_all(reg, tgt, &def.inflicts, popups);
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

    /// Resolve one hit from `actor` onto `target`. Returns `true` if it connected
    /// (so the caller can apply on-hit riders like status effects), `false` on a
    /// miss.
    #[allow(clippy::too_many_arguments)]
    fn strike(
        &mut self,
        actor: usize,
        target: usize,
        offense: i32,
        power: i32,
        hit_mod: HitMod,
        reg: &Registry,
        rng: &mut Rng,
        popups: &mut Vec<Popup>,
    ) -> bool {
        // Read attacker/target attributes up front, before borrowing the target.
        // Speed and defense come from effective stats so statuses (slow, …) count.
        let atk_acc = self.battlers[actor].accuracy;
        let atk_crit = self.battlers[actor].crit;
        let atk_spd = self.eff_stats(reg, actor).speed;
        let tgt = self.eff_stats(reg, target);
        let tgt_eva = self.battlers[target].evasion;
        let tgt_spd = tgt.speed;
        let defending = self.battlers[target].defending;
        let defense = tgt.defense;

        // Hit or miss: accuracy and being faster help you land; the target's
        // evasion and speed help it dodge.
        let hit = hit_chance(atk_acc, atk_spd, tgt_eva, tgt_spd);
        if !rng.chance(hit as f32 / 100.0) {
            let b = &mut self.battlers[target];
            popups.push(Popup {
                text: "MISS".to_string(),
                pos: b.pos() + Vec2::new(0.0, -6.0),
                t: 0.0,
                color: color::rgb(170, 170, 185),
            });
            return false;
        }

        // Base damage.
        let mut dmg = (offense * power / 100) - defense / 2;
        if dmg < 1 {
            dmg = 1;
        }

        // Critical hit: +50% damage.
        let is_crit = rng.chance(crit_chance(atk_crit) as f32 / 100.0);
        if is_crit {
            dmg = (dmg * 3 / 2).max(1);
        }

        // Random spread, then the defending reduction.
        let variance = rng.range(88, 112);
        dmg = (dmg * variance / 100).max(1);
        if defending {
            dmg = (dmg * 2 / 3).max(1);
        }

        // Attack-timing bonus (the attacker's well-timed swing does more).
        if hit_mod.attack_mult != 1.0 {
            dmg = ((dmg as f32 * hit_mod.attack_mult).round() as i32).max(1);
        }
        // Block-timing reduction (the defender's well-timed block does less to
        // them — a perfect block subtracts all of it).
        let blocked = hit_mod.block_reduce > 0.0;
        if blocked {
            dmg = (dmg as f32 * (1.0 - hit_mod.block_reduce)).round().max(0.0) as i32;
        }

        let b = &mut self.battlers[target];
        b.hp = (b.hp - dmg).max(0);
        b.flash = if is_crit { 0.45 } else { 0.3 };
        let (text, color) = if blocked && dmg == 0 {
            ("BLOCK".to_string(), color::rgb(140, 200, 255))
        } else if blocked {
            (format!("{dmg}"), color::rgb(140, 200, 255))
        } else if is_crit {
            (format!("{dmg}!"), color::rgb(255, 150, 60))
        } else {
            (format!("{dmg}"), color::rgb(255, 226, 120))
        };
        popups.push(Popup {
            text,
            pos: b.pos() + Vec2::new(0.0, -6.0),
            t: 0.0,
            color,
        });
        true
    }

    // ---- Status effects -----------------------------------------------------

    /// Apply each status id in `ids` to `target` (used as an on-hit rider). See
    /// [`Self::apply_status`].
    fn inflict_all(
        &mut self,
        reg: &Registry,
        target: usize,
        ids: &[String],
        popups: &mut Vec<Popup>,
    ) {
        for id in ids {
            self.apply_status(reg, target, id, popups);
        }
    }

    /// Attach status `id` to `target` (or refresh its duration if already
    /// present) and float its name so the player sees it land. Unknown ids and
    /// dead targets are ignored.
    fn apply_status(&mut self, reg: &Registry, target: usize, id: &str, popups: &mut Vec<Popup>) {
        let Some(def) = reg.status(id) else { return };
        if def.duration <= 0 || !self.battlers[target].alive() {
            return;
        }
        let b = &mut self.battlers[target];
        match b.statuses.iter_mut().find(|s| s.id == id) {
            // Re-applying refreshes to the longer of the two remaining counts.
            Some(s) => s.remaining = def.duration.max(s.remaining),
            None => b.statuses.push(ActiveStatus {
                id: id.to_string(),
                remaining: def.duration,
            }),
        }
        let pos = b.pos() + Vec2::new(0.0, -16.0);
        let color = status_color(def);
        popups.push(Popup {
            text: def.name.clone(),
            pos,
            t: 0.0,
            color,
        });
    }

    /// End-of-round resolution for every active status: deal (or heal) its
    /// per-turn HP, float a number, then count it down and drop the expired ones.
    fn tick_statuses(&mut self, reg: &Registry, popups: &mut Vec<Popup>) {
        for i in 0..self.battlers.len() {
            if !self.battlers[i].alive() {
                self.battlers[i].statuses.clear();
                continue;
            }
            // Sum this round's HP change and remember a colour, decrementing each
            // status and keeping only those with rounds left.
            let taken = std::mem::take(&mut self.battlers[i].statuses);
            let mut kept = Vec::with_capacity(taken.len());
            let mut delta = 0;
            let mut color = color::rgb(255, 170, 80);
            for st in taken {
                if let Some(def) = reg.status(&st.id) {
                    delta += def.damage_per_turn;
                    if def.damage_per_turn != 0 {
                        color = status_color(def);
                    }
                }
                let remaining = st.remaining - 1;
                if remaining > 0 {
                    kept.push(ActiveStatus {
                        id: st.id,
                        remaining,
                    });
                }
            }
            self.battlers[i].statuses = kept;

            if delta == 0 {
                continue;
            }
            let b = &mut self.battlers[i];
            b.flash = 0.3;
            let (text, popup_color) = if delta > 0 {
                b.hp = (b.hp - delta).max(0);
                (format!("{delta}"), color)
            } else {
                let before = b.hp;
                b.hp = (b.hp - delta).min(b.max_hp); // delta<0 heals
                (format!("+{}", b.hp - before), color::rgb(120, 240, 140))
            };
            popups.push(Popup {
                text,
                pos: b.pos() + Vec2::new(0.0, -6.0),
                t: 0.0,
                color: popup_color,
            });
        }
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

        // State tint (hit flash / KO desaturation) multiplied by the sprite's
        // own recolour so reskinned characters keep their palette.
        let (sr, sg, sb) = b
            .sprite
            .tint
            .map(|(r, g, b)| (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            .unwrap_or((1.0, 1.0, 1.0));
        let (mr, mg, mb) = if b.flash > 0.0 {
            let k = (b.flash / 0.3).clamp(0.0, 1.0);
            (1.0, 1.0 - k * 0.6, 1.0 - k * 0.6)
        } else if !b.alive() {
            (0.5, 0.5, 0.6)
        } else {
            (1.0, 1.0, 1.0)
        };
        let tint = [mr * sr, mg * sg, mb * sb, b.fade];
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
                [62.0, y + 1.0, 44.0, 4.0],
                b.hp,
                b.max_hp,
                color::rgb(80, 210, 90),
            );
            // MP bar.
            bar(
                r,
                [62.0, y + 6.0, 44.0, 3.0],
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
        let hero = cmd.hero;
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
                menu_box(r, 6.0, 16.0, 90.0, &ROOT_ITEMS, *cursor);
                self.draw_gear_panel(r, reg, hero);
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
                self.draw_skill_info(r, reg, hero, *cursor);
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

    /// The acting hero's equipped weapon and armor, with icons, bonuses, and
    /// descriptions — shown while choosing a command.
    fn draw_gear_panel(&self, r: &mut Renderer, reg: &Registry, hero: usize) {
        let b = &self.battlers[hero];
        // Swapped down from the top: the panel now sits at the bottom-left, just
        // above the party panel (whose height tracks the party size).
        let party_h = 8.0
            + self
                .battlers
                .iter()
                .filter(|bt| bt.side == Side::Hero)
                .count() as f32
                * 16.0;
        // Size the panel to its widest row so a gear-heavy weapon (e.g. Gareth's
        // SCOUT'S EDGE, which carries four stat mods) never spills past the box.
        // Text in each row starts 24px in (icon + gap); measure the name+mods
        // line and the description line, take the wider, and pad the right edge.
        let row_extent = |id: &Option<String>| -> f32 {
            match id.as_deref().and_then(|id| reg.equipment(id)) {
                Some(item) => {
                    let mods = mods_summary(item);
                    let mut line = r.text_width(&item.name, 1.0);
                    if !mods.is_empty() {
                        line += 6.0 + r.text_width(&mods, 1.0);
                    }
                    24.0 + line.max(r.text_width(&item.description, 1.0))
                }
                None => 0.0,
            }
        };
        let content = row_extent(&b.weapon).max(row_extent(&b.armor));
        let (w, h) = ((content + 4.0).max(196.0), 52.0);
        let x = 6.0;
        let y = VIRTUAL_H - party_h - h - 2.0;
        r.draw_rect(x, y, w, h, color::rgba(12, 14, 28, 232));
        r.draw_rect_outline(x, y, w, h, 1.0, color::rgba(80, 90, 140, 255));
        r.draw_text(
            "EQUIPMENT",
            x + 4.0,
            y + 3.0,
            1.0,
            color::rgb(170, 180, 210),
        );
        self.draw_gear_row(r, reg, "WPN", &b.weapon, x + 4.0, y + 14.0);
        self.draw_gear_row(r, reg, "ARM", &b.armor, x + 4.0, y + 33.0);
    }

    fn draw_gear_row(
        &self,
        r: &mut Renderer,
        reg: &Registry,
        label: &str,
        id: &Option<String>,
        x: f32,
        y: f32,
    ) {
        // Icon slot background.
        r.draw_rect(x, y, 16.0, 16.0, color::rgba(20, 24, 44, 255));
        r.draw_rect_outline(x, y, 16.0, 16.0, 0.5, color::rgba(90, 100, 150, 200));
        let tx = x + 20.0;
        match id
            .as_deref()
            .and_then(|id| reg.equipment(id).map(|it| (id, it)))
        {
            Some((id, item)) => {
                if let Some(&tex) = self.icons.get(id) {
                    r.draw_texture(tex, x, y, 16.0, 16.0, color::WHITE);
                }
                r.draw_text(&item.name, tx, y, 1.0, color::WHITE);
                let mods = mods_summary(item);
                if !mods.is_empty() {
                    let nw = r.text_width(&item.name, 1.0);
                    r.draw_text(&mods, tx + nw + 6.0, y, 1.0, color::rgb(150, 220, 160));
                }
                r.draw_text(
                    &item.description,
                    tx,
                    y + 9.0,
                    1.0,
                    color::rgb(190, 190, 205),
                );
            }
            None => {
                r.draw_text(
                    &format!("{label}: (none)"),
                    tx,
                    y + 4.0,
                    1.0,
                    color::rgb(120, 120, 140),
                );
            }
        }
    }

    /// The highlighted skill's name and description, shown in the skill menu.
    fn draw_skill_info(&self, r: &mut Renderer, reg: &Registry, hero: usize, cursor: usize) {
        let skills = &self.battlers[hero].skills;
        if cursor >= skills.len() {
            return; // "BACK" is highlighted
        }
        let Some(def) = reg.skill(&skills[cursor]) else {
            return;
        };
        let (x, y, w, h) = (6.0, 16.0, 138.0, 42.0);
        r.draw_rect(x, y, w, h, color::rgba(12, 14, 28, 232));
        r.draw_rect_outline(x, y, w, h, 1.0, color::rgba(90, 110, 170, 255));
        r.draw_text(&def.name, x + 4.0, y + 3.0, 1.0, color::rgb(255, 226, 120));
        for (i, line) in wrap_text(&def.description, 26).iter().take(3).enumerate() {
            r.draw_text(
                line,
                x + 4.0,
                y + 13.0 + i as f32 * 9.0,
                1.0,
                color::rgb(200, 200, 215),
            );
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
        hero: 0,
        stage: Stage::Root { cursor: 0 },
    }
}

fn dummy_execute() -> Execute {
    Execute {
        queue: vec![],
        idx: 0,
        elapsed: 0.0,
        applied: false,
        started: false,
        timed: None,
        hit_mod: HitMod::NEUTRAL,
        end_of_round: false,
        statuses_ticked: false,
        banner: String::new(),
        popups: vec![],
    }
}

fn needs_cursor(t: TargetKind) -> bool {
    matches!(t, TargetKind::OneEnemy | TargetKind::OneAlly)
}

/// A status's popup colour: its configured `tint`, or a warm orange default.
fn status_color(def: &StatusDef) -> Color {
    def.tint
        .map(|(r, g, b)| color::rgb(r, g, b))
        .unwrap_or_else(|| color::rgb(255, 170, 80))
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

/// Chance (percent) that an attack lands, given the attacker's accuracy/speed
/// and the target's evasion/speed. Clamped so nothing is a guaranteed hit or a
/// hopeless one.
fn hit_chance(atk_acc: i32, atk_spd: i32, tgt_eva: i32, tgt_spd: i32) -> i32 {
    (BASE_HIT + atk_acc - tgt_eva + (atk_spd - tgt_spd)).clamp(MIN_HIT, MAX_HIT)
}

/// Chance (percent) that a landed hit is a critical (deals +50% damage).
fn crit_chance(atk_crit: i32) -> i32 {
    (BASE_CRIT + atk_crit).clamp(0, MAX_CRIT)
}

/// A compact "+6 ATK  +5% CRIT" line summarising an item's bonuses.
fn mods_summary(item: &EquipmentDef) -> String {
    let m = &item.mods;
    let mut parts = Vec::new();
    if m.attack != 0 {
        parts.push(format!("{:+} ATK", m.attack));
    }
    if m.defense != 0 {
        parts.push(format!("{:+} DEF", m.defense));
    }
    if m.magic != 0 {
        parts.push(format!("{:+} MAG", m.magic));
    }
    if m.speed != 0 {
        parts.push(format!("{:+} SPD", m.speed));
    }
    if item.crit != 0 {
        parts.push(format!("{:+}% CRIT", item.crit));
    }
    if item.accuracy != 0 {
        parts.push(format!("{:+}% ACC", item.accuracy));
    }
    if item.evasion != 0 {
        parts.push(format!("{:+}% EVA", item.evasion));
    }
    parts.join(" ")
}

/// Greedy word-wrap to at most `max` characters per line (fixed-width font).
fn wrap_text(text: &str, max: usize) -> Vec<String> {
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

/// Resolve and cache the icon texture for each given equipped item id.
fn load_item_icons(
    renderer: &mut Renderer,
    cache: &mut TextureCache,
    reg: &Registry,
    icons: &mut HashMap<String, TextureHandle>,
    ids: [&Option<String>; 2],
) {
    for id in ids.into_iter().flatten() {
        if !icons.contains_key(id) {
            if let Some(item) = reg.equipment(id) {
                let h = cache.get(renderer, &item.icon);
                icons.insert(id.clone(), h);
            }
        }
    }
}

fn hero_home(slot: usize) -> Vec2 {
    // Diagonal column on the left, facing right.
    Vec2::new(70.0 - slot as f32 * 8.0, 92.0 + slot as f32 * 24.0)
}

fn enemy_home(slot: usize) -> Vec2 {
    Vec2::new(228.0 + (slot % 2) as f32 * 20.0, 74.0 + slot as f32 * 26.0)
}

fn bar(r: &mut Renderer, rect: [f32; 4], value: i32, max: i32, fill: Color) {
    let [x, y, w, h] = rect;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_chance_baseline_and_clamps() {
        // Equal, unequipped combatants sit at the baseline.
        assert_eq!(hit_chance(0, 10, 0, 10), BASE_HIT);
        // Accuracy raises it and speed advantage helps, but it never hits 100%.
        assert_eq!(hit_chance(50, 20, 0, 5), MAX_HIT);
        // Overwhelming evasion can't drop it below the floor.
        assert_eq!(hit_chance(0, 5, 80, 30), MIN_HIT);
    }

    #[test]
    fn hit_chance_reacts_to_evasion_and_speed() {
        let base = hit_chance(0, 10, 0, 10);
        assert!(hit_chance(0, 10, 8, 10) < base, "evasion lowers hit chance");
        assert!(
            hit_chance(6, 10, 0, 10) > base,
            "accuracy raises hit chance"
        );
        assert!(
            hit_chance(0, 14, 0, 9) > base,
            "being faster raises hit chance"
        );
    }

    #[test]
    fn crit_chance_baseline_and_clamps() {
        assert_eq!(crit_chance(0), BASE_CRIT);
        assert!(crit_chance(6) > BASE_CRIT, "weapon crit adds to the rate");
        assert_eq!(crit_chance(1000), MAX_CRIT, "crit is capped");
        assert_eq!(crit_chance(-1000), 0, "crit never goes negative");
    }

    /// A single clean tap `off` seconds away from the connect, with its result.
    fn tap_at(kind: TimingKind, off: f32) -> TimingResult {
        let mut t = TimedHit::new(kind, 0);
        t.press_t = Some(STRIKE_CONNECT + off);
        t.presses = 1;
        t.result()
    }

    #[test]
    fn attack_timing_scales_damage_by_lateness() {
        // Dead on the connect → perfect, double damage.
        assert_eq!(tap_at(TimingKind::Attack, 0.0), TimingResult::Perfect);
        // A hair off (inside GOOD but outside PERFECT) → good, +50%.
        assert_eq!(
            tap_at(TimingKind::Attack, STRIKE_PERFECT + 0.02),
            TimingResult::Good
        );
        // Well before the connect → miss, no bonus. An early panic tap whiffs.
        assert_eq!(
            tap_at(TimingKind::Attack, -(STRIKE_GOOD + 0.05)),
            TimingResult::Miss
        );

        // The multipliers those results carry.
        let m = |r| timed_for(TimingKind::Attack, r).hit_mod().attack_mult;
        assert_eq!(m(TimingResult::Perfect), 2.0);
        assert_eq!(m(TimingResult::Good), 1.5);
        assert_eq!(m(TimingResult::Miss), 1.0);
    }

    #[test]
    fn block_timing_subtracts_incoming_damage() {
        assert_eq!(tap_at(TimingKind::Block, 0.0), TimingResult::Perfect);
        let reduce = |r| {
            let m = timed_for(TimingKind::Block, r).hit_mod();
            assert_eq!(m.attack_mult, 1.0, "block never boosts offense");
            m.block_reduce
        };
        assert_eq!(
            reduce(TimingResult::Perfect),
            1.0,
            "a perfect block negates it"
        );
        assert_eq!(reduce(TimingResult::Good), 0.5, "a late block halves it");
        assert_eq!(
            reduce(TimingResult::Miss),
            0.0,
            "a missed block does nothing"
        );
    }

    /// A single-tap `TimedHit` that yields the given result, for hit_mod checks.
    fn timed_for(kind: TimingKind, r: TimingResult) -> TimedHit {
        let mut t = TimedHit::new(kind, 0);
        match r {
            TimingResult::Perfect => {
                t.press_t = Some(STRIKE_CONNECT);
                t.presses = 1;
            }
            TimingResult::Good => {
                t.press_t = Some(STRIKE_CONNECT + STRIKE_PERFECT + 0.02);
                t.presses = 1;
            }
            TimingResult::Miss => {} // no tap
        }
        t
    }

    #[test]
    fn untapped_timing_misses() {
        // No tap at all during the swing → miss, no bonus.
        let t = TimedHit::new(TimingKind::Attack, 0);
        assert_eq!(t.result(), TimingResult::Miss);
        assert_eq!(t.hit_mod().attack_mult, 1.0);
    }

    #[test]
    fn mashing_forfeits_the_bonus() {
        // Multiple taps — even with one landing dead on the connect — earn
        // nothing, so hammering the button can't fish for a perfect.
        let mut t = TimedHit::new(TimingKind::Attack, 0);
        t.press_t = Some(STRIKE_CONNECT); // first tap was perfectly timed…
        t.presses = 4; // …but it was part of a mash
        assert_eq!(t.result(), TimingResult::Miss);
        assert_eq!(t.hit_mod().attack_mult, 1.0);
    }

    #[test]
    fn declared_unblockable_attacks_skip_the_block_window() {
        let reg = crate::data::Registry::load();
        for id in [
            "fireball",
            "flame_breath",
            "lance_charge",
            "firebolt",
            "frost",
        ] {
            assert!(
                reg.skill(id).expect(id).is_unblockable(),
                "{id} should be unblockable"
            );
        }
        // TRAMPLE is a heavy but brace-able stomp — blockable, unlike the piercing
        // LANCE CHARGE — so the party keeps timing counterplay against a knight pack.
        for id in ["claw", "tail_swipe", "reap", "trample"] {
            assert!(
                !reg.skill(id).expect(id).is_unblockable(),
                "{id} should be blockable"
            );
        }
    }
}
