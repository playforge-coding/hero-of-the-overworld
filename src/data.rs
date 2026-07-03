//! Game data: the on-disk RON file format plus the registries built from it.
//!
//! All content (party characters, enemies, skills, encounters) lives in
//! `assets/data/game.ron` and is parsed into these serde structs. Adding a new
//! party member or enemy is a pure data edit — no code change required — which
//! is what makes the party/enemy systems "extensible".
//!
//! Textures are referenced by a string *key*; the actual PNG bytes are embedded
//! at compile time (see [`embedded_texture`]) so the exact same load path works
//! natively and on the web with no async asset fetching.

use std::collections::HashMap;

use serde::Deserialize;

/// Core combat stats shared by heroes and enemies.
#[derive(Clone, Debug, Deserialize)]
pub struct Stats {
    pub max_hp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub speed: i32,
}

/// A sprite-sheet slice: one animation row played as frames.
#[derive(Clone, Debug, Deserialize)]
pub struct AnimClip {
    pub row: u32,
    pub first_col: u32,
    pub frames: u32,
    pub fps: f32,
}

/// How a battler is drawn: which sheet, its frame grid, and named clips.
#[derive(Clone, Debug, Deserialize)]
pub struct BattlerSprite {
    /// Texture key resolved by [`embedded_texture`].
    pub texture: String,
    pub frame_w: u32,
    pub frame_h: u32,
    /// Rendered size in virtual pixels.
    pub draw_w: f32,
    pub draw_h: f32,
    /// If true, the artwork faces left by default and is flipped to face right.
    #[serde(default)]
    pub faces_left: bool,
    pub idle: AnimClip,
    pub attack: AnimClip,
    #[serde(default)]
    pub hurt: Option<AnimClip>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum SkillKind {
    Physical,
    Magical,
    Heal,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum TargetKind {
    OneEnemy,
    AllEnemies,
    OneAlly,
    AllAllies,
    SelfOnly,
}

/// An action a battler can perform. "Attack" is just the built-in basic skill.
#[derive(Clone, Debug, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub mp_cost: i32,
    /// Percentage multiplier applied to the relevant offensive stat.
    pub power: i32,
    pub kind: SkillKind,
    pub target: TargetKind,
}

/// A playable party character definition.
#[derive(Clone, Debug, Deserialize)]
pub struct CharacterDef {
    pub id: String,
    pub name: String,
    pub stats: Stats,
    pub sprite: BattlerSprite,
    /// Skill ids this character knows (in addition to the basic Attack).
    #[serde(default)]
    pub skills: Vec<String>,
}

/// Simple enemy behaviour selector, resolved by the battle AI.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Default)]
pub enum EnemyAi {
    /// Always uses its basic attack on a random hero.
    #[default]
    Basic,
    /// Randomly picks among its skills / basic attack.
    Random,
}

/// An enemy definition in the registry.
#[derive(Clone, Debug, Deserialize)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub stats: Stats,
    pub sprite: BattlerSprite,
    #[serde(default)]
    pub skills: Vec<String>,
    /// Behaviour selector; defaults to [`EnemyAi::Basic`] when omitted.
    #[serde(default)]
    pub ai: EnemyAi,
    pub xp: i32,
    pub gold: i32,
}

/// A named group of enemies used to seed a battle.
#[derive(Clone, Debug, Deserialize)]
pub struct EncounterDef {
    pub id: String,
    /// Enemy ids; repeats allowed (e.g. two demons).
    pub enemies: Vec<String>,
}

/// Root of the RON data file.
#[derive(Clone, Debug, Deserialize)]
pub struct GameData {
    pub characters: Vec<CharacterDef>,
    pub enemies: Vec<EnemyDef>,
    pub skills: Vec<SkillDef>,
    pub encounters: Vec<EncounterDef>,
    /// Character ids that make up the party at the start of the game.
    pub starting_party: Vec<String>,
}

/// Indexed, validated view of [`GameData`] for fast lookups during play.
pub struct Registry {
    pub data: GameData,
    characters: HashMap<String, usize>,
    enemies: HashMap<String, usize>,
    skills: HashMap<String, usize>,
    encounters: HashMap<String, usize>,
}

impl Registry {
    pub fn load() -> Self {
        let src = include_str!("../assets/data/game.ron");
        let data: GameData = ron::from_str(src).expect("parse assets/data/game.ron");
        Self::from_data(data)
    }

    pub fn from_data(data: GameData) -> Self {
        let index = |f: &dyn Fn() -> Vec<String>| -> HashMap<String, usize> {
            f().into_iter().enumerate().map(|(i, k)| (k, i)).collect()
        };
        let characters = index(&|| data.characters.iter().map(|c| c.id.clone()).collect());
        let enemies = index(&|| data.enemies.iter().map(|c| c.id.clone()).collect());
        let skills = index(&|| data.skills.iter().map(|c| c.id.clone()).collect());
        let encounters = index(&|| data.encounters.iter().map(|c| c.id.clone()).collect());
        Self {
            data,
            characters,
            enemies,
            skills,
            encounters,
        }
    }

    pub fn character(&self, id: &str) -> Option<&CharacterDef> {
        self.characters.get(id).map(|&i| &self.data.characters[i])
    }

    pub fn enemy(&self, id: &str) -> Option<&EnemyDef> {
        self.enemies.get(id).map(|&i| &self.data.enemies[i])
    }

    pub fn skill(&self, id: &str) -> Option<&SkillDef> {
        self.skills.get(id).map(|&i| &self.data.skills[i])
    }

    pub fn encounter(&self, id: &str) -> Option<&EncounterDef> {
        self.encounters.get(id).map(|&i| &self.data.encounters[i])
    }
}

impl EnemyDef {
    pub fn ai(&self) -> EnemyAi {
        self.ai
    }
}

/// Resolve a texture key (as used in the RON) to embedded PNG bytes.
///
/// New art is registered here once; RON then references it by key. Keeping this
/// a compile-time `match` (rather than a filesystem read) is what lets the web
/// build ship every asset inside the wasm bundle.
pub fn embedded_texture(key: &str) -> Option<&'static [u8]> {
    Some(match key {
        "swordsman" => include_bytes!("../assets/textures/entities/playables/swordsman.png"),
        "demon" => include_bytes!("../assets/textures/entities/monsters/demon.png"),
        _ => return None,
    })
}

/// The embedded font atlas PNG.
pub const FONT_PNG: &[u8] = include_bytes!("../assets/textures/ui/font.png");
