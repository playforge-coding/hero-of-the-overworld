//! The persistent party.
//!
//! The party is a plain `Vec` of [`PartyMember`], so supporting more than one
//! hero — recruiting new characters mid-game, swapping the active line-up — is
//! just pushing/removing entries. Battle code iterates whatever is here, so a
//! second or third character assisting in battle needs no special-casing.

use crate::data::{BattlerSprite, Registry, Stats};

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
            level: 1,
            xp: 0,
        })
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

/// The whole active party. Extensible: add characters as they are recruited.
#[derive(Clone, Debug, Default)]
pub struct Party {
    pub members: Vec<PartyMember>,
    pub gold: i32,
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
    pub fn recruit(&mut self, reg: &Registry, id: &str) -> bool {
        match PartyMember::from_def(reg, id) {
            Some(m) => {
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
                m.level += 1;
                // Modest, extensible growth curve.
                m.stats.max_hp += 12;
                m.stats.max_mp += 3;
                m.stats.attack += 2;
                m.stats.defense += 1;
                m.stats.magic += 1;
                m.stats.speed += 1;
                m.hp = m.stats.max_hp;
                m.mp = m.stats.max_mp;
                leveled.push(i);
            }
        }
        leveled
    }
}
