//! The persistent party.
//!
//! The party is a plain `Vec` of [`PartyMember`], so supporting more than one
//! hero — recruiting new characters mid-game, swapping the active line-up — is
//! just pushing/removing entries. Battle code iterates whatever is here, so a
//! second or third character assisting in battle needs no special-casing.

use crate::data::{BattlerSprite, CharacterDef, EquipSlot, ItemEffect, Registry, SkillKind, Stats};

/// One level a member gained from an XP award, together with any skills that
/// level unlocked — enough for the battle-victory report to announce both the
/// level and every newly learned move.
pub struct LevelUp {
    /// Index into [`Party::members`].
    pub member: usize,
    /// The level reached.
    pub level: i32,
    /// Skill ids newly learned on reaching this level (empty if none).
    pub learned: Vec<String>,
}

/// Outcome of using a healing move or item from the overworld (the inventory
/// screen), enough for the UI to report what happened without re-deriving it.
pub enum FieldUse {
    /// It worked: this much HP and MP was restored to the target (either may be 0).
    Restored { hp: i32, mp: i32 },
    /// Nothing to do (target already full, or the move/item isn't restorative), so
    /// no MP or item was spent.
    NoEffect,
    /// A move the caster couldn't afford — not enough MP. Nothing was spent.
    NotEnoughMp,
}

/// A stack of one consumable item and how many the party carries. Items stack
/// (you hold several potions), so — unlike the equipment [bag](Party::bag) — the
/// item stash is a counted multiset keyed by item id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: u32,
}

/// A recruited character. Carries live HP/MP between battles.
#[derive(Clone, Debug)]
pub struct PartyMember {
    pub def_id: String,
    pub name: String,
    pub stats: Stats,
    pub hp: i32,
    pub mp: i32,
    pub skills: Vec<String>,
    pub sprite: BattlerSprite,
    /// Equipped weapon / armor ids (into the registry's `equipment`).
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub level: i32,
    pub xp: i32,
}

impl PartyMember {
    /// Build a fresh member from a character definition at full health, knowing
    /// its starting skills plus anything its learnset already unlocks at level 1.
    pub fn from_def(reg: &Registry, id: &str) -> Option<Self> {
        let def = reg.character(id)?;
        let mut m = Self {
            def_id: def.id.clone(),
            name: def.name.clone(),
            stats: def.stats.clone(),
            hp: def.stats.max_hp,
            mp: def.stats.max_mp,
            skills: def.skills.clone(),
            sprite: def.sprite.clone(),
            weapon: def.weapon.clone(),
            armor: def.armor.clone(),
            level: 1,
            xp: 0,
        };
        m.learn_skills_for_level(def);
        Some(m)
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Learn every skill `def`'s learnset unlocks at or below this member's
    /// current level that they don't already know, returning the ids newly
    /// learned. Idempotent (already-known skills are skipped), so it's safe to
    /// call after any level change — a fresh level-up, a mid-game recruit brought
    /// up to party level, or a save reloaded at a higher level.
    pub fn learn_skills_for_level(&mut self, def: &CharacterDef) -> Vec<String> {
        let mut learned = Vec::new();
        for entry in &def.learnset {
            if entry.level <= self.level && !self.skills.contains(&entry.skill) {
                self.skills.push(entry.skill.clone());
                learned.push(entry.skill.clone());
            }
        }
        learned
    }

    /// Apply a consumable item's **restorative** effect to this member outside of
    /// battle (from the inventory screen). Returns the HP and MP actually restored
    /// (either may be 0), so a full-health target refuses the item rather than
    /// wasting it. Damage and status effects are ignored here — they only mean
    /// something in a fight (see [`ItemEffect::usable_in_field`]); a heal on a
    /// downed member revives them.
    fn apply_field_item(&mut self, effect: &ItemEffect) -> (i32, i32) {
        let mut hp = 0;
        let mut mp = 0;
        if effect.heal > 0 && self.hp < self.stats.max_hp {
            let before = self.hp;
            self.hp = (self.hp + effect.heal).min(self.stats.max_hp);
            hp = self.hp - before;
        }
        if effect.restore_mp > 0 && self.mp < self.stats.max_mp {
            let before = self.mp;
            self.mp = (self.mp + effect.restore_mp).min(self.stats.max_mp);
            mp = self.mp - before;
        }
        (hp, mp)
    }

    /// Apply one level's worth of growth: bump the level, add the modest,
    /// extensible stat gains, and refill HP/MP. Speed is intentionally left
    /// unchanged — enemies don't gain speed with level either, so leveling it
    /// would skew turn order. This is the single source of the growth curve,
    /// shared by [`Party::grant_xp`] (earning a level in battle) and
    /// [`Party::recruit`] (bringing a new hero up to the party's level). Returns
    /// any skills the new level unlocks (see [`learn_skills_for_level`]).
    ///
    /// [`learn_skills_for_level`]: Self::learn_skills_for_level
    fn level_up(&mut self, def: &CharacterDef) -> Vec<String> {
        self.level += 1;
        self.stats.max_hp += 12;
        self.stats.max_mp += 3;
        self.stats.attack += 2;
        self.stats.defense += 1;
        self.stats.magic += 1;
        self.hp = self.stats.max_hp;
        self.mp = self.stats.max_mp;
        self.learn_skills_for_level(def)
    }
}

/// The whole active party. Extensible: add characters as they are recruited.
#[derive(Clone, Debug, Default)]
pub struct Party {
    pub members: Vec<PartyMember>,
    pub gold: i32,
    /// Owned but **unequipped** equipment (ids into the registry's `equipment`),
    /// a multiset — duplicates allowed. Gear bought at a shop that displaces an
    /// equipped item lands here, and the inventory screen equips/unequips between
    /// this bag and party members. Persisted in the save file.
    pub bag: Vec<String>,
    /// Owned consumable items and their counts (ids into the registry's `items`).
    /// Filled by shop purchases and monster drops, spent when used in battle or
    /// from the inventory screen. Persisted in the save file.
    pub items: Vec<ItemStack>,
}

impl Party {
    /// Build the starting party listed in the data file.
    pub fn from_registry(reg: &Registry) -> Self {
        let mut party = Party::default();
        for id in &reg.data.starting_party {
            party.recruit(reg, id);
        }
        party
    }

    /// Add a character to the party by id. Returns whether it was added.
    /// This is the primary extension point for "more characters in battle".
    ///
    /// A new recruit joins at the **party's current level** rather than level 1,
    /// so a hero picked up mid-game (Elara after GREENWOOD, Gareth after
    /// TRAVELLER'S END) arrives on par with Roland instead of as dead weight —
    /// their base stats are grown up the same curve leveling grants. The starting
    /// party is unaffected: it is built one member at a time from an empty party,
    /// so the level target is 1.
    pub fn recruit(&mut self, reg: &Registry, id: &str) -> bool {
        let target_level = self.level();
        let Some(def) = reg.character(id) else {
            log::warn!("recruit failed: unknown character '{id}'");
            return false;
        };
        let Some(mut m) = PartyMember::from_def(reg, id) else {
            return false;
        };
        // Grow up to the party's level, learning each level's skills on the way —
        // so a hero recruited mid-game arrives knowing everything their level
        // grants, not just their starting kit.
        while m.level < target_level {
            m.level_up(def);
        }
        self.members.push(m);
        true
    }

    pub fn any_alive(&self) -> bool {
        self.members.iter().any(|m| m.is_alive())
    }

    /// A member's current item in `slot`, if any.
    fn slot_id(member: &PartyMember, slot: EquipSlot) -> &Option<String> {
        match slot {
            EquipSlot::Weapon => &member.weapon,
            EquipSlot::Armor => &member.armor,
        }
    }

    fn slot_id_mut(member: &mut PartyMember, slot: EquipSlot) -> &mut Option<String> {
        match slot {
            EquipSlot::Weapon => &mut member.weapon,
            EquipSlot::Armor => &mut member.armor,
        }
    }

    /// Move a member's equipped item in `slot` back into the [bag](Self::bag),
    /// leaving the slot empty. No-op if the slot is already empty.
    pub fn unequip(&mut self, member: usize, slot: EquipSlot) {
        if let Some(m) = self.members.get_mut(member) {
            if let Some(id) = Self::slot_id_mut(m, slot).take() {
                self.bag.push(id);
            }
        }
    }

    /// Equip the bag item at `bag_index` onto `member`, into the slot the item's
    /// definition dictates. Whatever the member had in that slot swaps back into
    /// the bag, so nothing is ever lost or duplicated. Returns whether it equipped
    /// (a valid, known item and member).
    pub fn equip_from_bag(&mut self, reg: &Registry, member: usize, bag_index: usize) -> bool {
        let Some(id) = self.bag.get(bag_index).cloned() else {
            return false;
        };
        let Some(slot) = reg.equipment(&id).map(|e| e.slot) else {
            return false;
        };
        if member >= self.members.len() {
            return false;
        }
        // Take the item out of the bag, swap it in, and return the displaced one.
        self.bag.remove(bag_index);
        let prev = Self::slot_id_mut(&mut self.members[member], slot).replace(id);
        if let Some(prev) = prev {
            self.bag.push(prev);
        }
        true
    }

    /// The bag indices holding equipment of the given `slot` type, for building a
    /// slot-filtered chooser in the inventory screen.
    pub fn bag_indices_for(&self, reg: &Registry, slot: EquipSlot) -> Vec<usize> {
        self.bag
            .iter()
            .enumerate()
            .filter(|(_, id)| reg.equipment(id).map(|e| e.slot) == Some(slot))
            .map(|(i, _)| i)
            .collect()
    }

    /// A member's equipped item id in `slot`, for display.
    pub fn equipped(&self, member: usize, slot: EquipSlot) -> Option<&str> {
        self.members
            .get(member)
            .and_then(|m| Self::slot_id(m, slot).as_deref())
    }

    /// The party's overall level: the highest level any member has reached.
    /// Used to scale roaming enemies (see [`crate::data::Stats::scaled_to`]) so
    /// they keep pace with the party. The max (rather than the average) keeps the
    /// yardstick stable when a fresh, low-level character is recruited mid-game.
    pub fn level(&self) -> i32 {
        self.members.iter().map(|m| m.level).max().unwrap_or(1)
    }

    /// Bring downed members back on their feet with a sliver of health. Called
    /// at the start of each battle so a KO'd hero rejoins the next fight rather
    /// than being lost for good.
    pub fn revive_downed(&mut self, hp: i32) {
        for m in &mut self.members {
            if !m.is_alive() {
                m.hp = hp.min(m.stats.max_hp);
            }
        }
    }

    /// Restore everyone to full (e.g. after resting / on a new run).
    pub fn full_heal(&mut self) {
        for m in &mut self.members {
            m.hp = m.stats.max_hp;
            m.mp = m.stats.max_mp;
        }
    }

    /// Award XP to living members, applying each level-up's stat growth and any
    /// skills the new level unlocks. Returns one [`LevelUp`] per level gained (in
    /// order) so the caller can report both the level and the learned moves.
    pub fn grant_xp(&mut self, reg: &Registry, amount: i32) -> Vec<LevelUp> {
        let mut events = Vec::new();
        for i in 0..self.members.len() {
            if !self.members[i].is_alive() {
                continue;
            }
            // The character def (its learnset) comes from the registry, disjoint
            // from the mutable party, so both borrows coexist. An unknown def still
            // banks the XP but can't grow or teach.
            let Some(def) = reg.character(&self.members[i].def_id) else {
                self.members[i].xp += amount;
                continue;
            };
            let m = &mut self.members[i];
            m.xp += amount;
            loop {
                let needed = m.level * 20;
                if m.xp < needed {
                    break;
                }
                m.xp -= needed;
                let learned = m.level_up(def);
                events.push(LevelUp {
                    member: i,
                    level: m.level,
                    learned,
                });
            }
        }
        events
    }

    /// Add `count` of consumable item `id` to the stash, stacking onto an existing
    /// entry or starting a new one. A zero count is a no-op.
    pub fn add_item(&mut self, id: &str, count: u32) {
        if count == 0 {
            return;
        }
        match self.items.iter_mut().find(|s| s.id == id) {
            Some(stack) => stack.count += count,
            None => self.items.push(ItemStack {
                id: id.to_string(),
                count,
            }),
        }
    }

    /// Use the item at `item_index` on member `member` **from the field** (the
    /// inventory screen). Applies its restorative effect and, only if that did
    /// something, consumes one from the stack — emptied stacks are removed.
    /// Battle usage is handled separately in [`crate::battle`] so the effect can
    /// also damage or buff.
    pub fn use_item_in_field(
        &mut self,
        reg: &Registry,
        item_index: usize,
        member: usize,
    ) -> FieldUse {
        let Some(id) = self.items.get(item_index).map(|s| s.id.clone()) else {
            return FieldUse::NoEffect;
        };
        let Some(effect) = reg.item(&id).map(|it| it.effect.clone()) else {
            return FieldUse::NoEffect;
        };
        let Some(m) = self.members.get_mut(member) else {
            return FieldUse::NoEffect;
        };
        let (hp, mp) = m.apply_field_item(&effect);
        if hp == 0 && mp == 0 {
            return FieldUse::NoEffect;
        }
        self.consume_item(item_index);
        FieldUse::Restored { hp, mp }
    }

    /// The **field-usable healing moves** `member` knows: skill ids whose effect
    /// restores HP outside of battle (a `Heal`-kind skill). Damage and buff skills
    /// are left out — they need a fight to matter.
    pub fn field_heal_skills(&self, reg: &Registry, member: usize) -> Vec<String> {
        let Some(m) = self.members.get(member) else {
            return Vec::new();
        };
        m.skills
            .iter()
            .filter(|id| reg.skill(id).map(|s| s.kind) == Some(SkillKind::Heal))
            .cloned()
            .collect()
    }

    /// Cast `caster`'s healing move `skill_id` on `target` **from the field**:
    /// spend the caster's MP and restore the target's HP (scaled off the caster's
    /// effective magic, like in battle). Refuses — spending nothing — if the skill
    /// isn't a heal, the caster can't afford it, or the target is already full.
    pub fn use_heal_skill_in_field(
        &mut self,
        reg: &Registry,
        caster: usize,
        skill_id: &str,
        target: usize,
    ) -> FieldUse {
        let Some(def) = reg.skill(skill_id) else {
            return FieldUse::NoEffect;
        };
        if def.kind != SkillKind::Heal {
            return FieldUse::NoEffect;
        }
        let (mp_cost, power) = (def.mp_cost, def.power);
        // Caster's affordability and effective magic (base + equipment).
        let mag = match self.members.get(caster) {
            Some(c) if c.is_alive() => {
                if c.mp < mp_cost {
                    return FieldUse::NotEnoughMp;
                }
                reg.equipped(&c.stats, c.weapon.as_deref(), c.armor.as_deref())
                    .stats
                    .magic
            }
            _ => return FieldUse::NoEffect,
        };
        let heal = (mag * power / 100).max(1);
        // Apply to the target; bail (spending no MP) if it would be wasted.
        let gained = match self.members.get_mut(target) {
            Some(t) if t.is_alive() && t.hp < t.stats.max_hp => {
                let before = t.hp;
                t.hp = (t.hp + heal).min(t.stats.max_hp);
                t.hp - before
            }
            _ => return FieldUse::NoEffect,
        };
        self.members[caster].mp -= mp_cost;
        FieldUse::Restored { hp: gained, mp: 0 }
    }

    /// Remove one of the item stack at `item_index`, dropping the stack when it
    /// hits zero. Out-of-range indices are ignored.
    pub fn consume_item(&mut self, item_index: usize) {
        if let Some(stack) = self.items.get_mut(item_index) {
            stack.count = stack.count.saturating_sub(1);
            if stack.count == 0 {
                self.items.remove(item_index);
            }
        }
    }
}
