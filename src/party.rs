//! The persistent party.
//!
//! The party is a plain `Vec` of [`PartyMember`], so supporting more than one
//! hero — recruiting new characters mid-game, swapping the active line-up — is
//! just pushing/removing entries. Battle code iterates whatever is here, so a
//! second or third character assisting in battle needs no special-casing.

use crate::data::{BattlerSprite, EquipSlot, Registry, Stats};

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
    /// Build a fresh member from a character definition at full health.
    pub fn from_def(reg: &Registry, id: &str) -> Option<Self> {
        let def = reg.character(id)?;
        Some(Self {
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
        })
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Apply one level's worth of growth: bump the level, add the modest,
    /// extensible stat gains, and refill HP/MP. Speed is intentionally left
    /// unchanged — enemies don't gain speed with level either, so leveling it
    /// would skew turn order. This is the single source of the growth curve,
    /// shared by [`Party::grant_xp`] (earning a level in battle) and
    /// [`Party::recruit`] (bringing a new hero up to the party's level).
    fn level_up(&mut self) {
        self.level += 1;
        self.stats.max_hp += 12;
        self.stats.max_mp += 3;
        self.stats.attack += 2;
        self.stats.defense += 1;
        self.stats.magic += 1;
        self.hp = self.stats.max_hp;
        self.mp = self.stats.max_mp;
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
        match PartyMember::from_def(reg, id) {
            Some(mut m) => {
                while m.level < target_level {
                    m.level_up();
                }
                self.members.push(m);
                true
            }
            None => {
                log::warn!("recruit failed: unknown character '{id}'");
                false
            }
        }
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

    /// Award XP to living members and grant a simple level-up. Returns members
    /// that gained a level (by index) for UI feedback.
    pub fn grant_xp(&mut self, amount: i32) -> Vec<usize> {
        let mut leveled = Vec::new();
        for (i, m) in self.members.iter_mut().enumerate() {
            if !m.is_alive() {
                continue;
            }
            m.xp += amount;
            let needed = m.level * 20;
            while m.xp >= needed {
                m.xp -= needed;
                m.level_up();
                leveled.push(i);
            }
        }
        leveled
    }
}
