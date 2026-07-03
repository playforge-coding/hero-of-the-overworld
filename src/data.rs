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
    /// Optional RGB recolour multiplied over the sprite (e.g. to reskin a shared
    /// sheet for a different character). Defaults to no tint (white).
    #[serde(default)]
    pub tint: Option<(u8, u8, u8)>,
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
    /// One-line flavour/effect text shown in the skill menu.
    pub description: String,
    #[serde(default)]
    pub mp_cost: i32,
    /// Percentage multiplier applied to the relevant offensive stat.
    pub power: i32,
    pub kind: SkillKind,
    pub target: TargetKind,
}

/// Which slot a piece of equipment occupies. A battler holds at most one of each.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum EquipSlot {
    Weapon,
    Armor,
}

/// Flat stat bonuses granted by a piece of equipment, added on top of a
/// battler's base stats. Deliberately excludes max HP/MP — equipment tunes the
/// *combat* stats, while HP/MP growth stays the domain of levelling up.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct StatMods {
    #[serde(default)]
    pub attack: i32,
    #[serde(default)]
    pub defense: i32,
    #[serde(default)]
    pub magic: i32,
    #[serde(default)]
    pub speed: i32,
}

/// A weapon or piece of armor. Weapons typically raise attack/magic and add
/// **crit** and **accuracy**; armor raises defense and adds **evasion** (the
/// chance to dodge a blow entirely). Every item carries a description.
#[derive(Clone, Debug, Deserialize)]
pub struct EquipmentDef {
    pub id: String,
    pub name: String,
    /// Flavour / effect text shown wherever the item is inspected.
    pub description: String,
    pub slot: EquipSlot,
    /// Texture key for the item's 16×16 icon (resolved by [`embedded_texture`]).
    pub icon: String,
    #[serde(default)]
    pub mods: StatMods,
    /// Added crit chance, in percent (mostly weapons).
    #[serde(default)]
    pub crit: i32,
    /// Added accuracy, in percent (mostly weapons).
    #[serde(default)]
    pub accuracy: i32,
    /// Added evasion, in percent (mostly armor).
    #[serde(default)]
    pub evasion: i32,
}

/// A battler's stats after equipment is applied, plus the derived combat
/// attributes ([crit](Equipped::crit) / [accuracy](Equipped::accuracy) /
/// [evasion](Equipped::evasion)) that drive hit and critical rolls.
#[derive(Clone, Debug)]
pub struct Equipped {
    pub stats: Stats,
    pub crit: i32,
    pub accuracy: i32,
    pub evasion: i32,
}

/// How a character is drawn while walking around the overworld.
///
/// The sheet convention (see `game.ron`) is one walk row per facing direction,
/// each a run of `frames` columns played at `fps`. Separate from
/// [`BattlerSprite`] because the overworld needs four directions where battle
/// only needs idle/attack.
#[derive(Clone, Debug, Deserialize)]
pub struct OverworldWalk {
    /// Texture key resolved by [`embedded_texture`].
    pub texture: String,
    pub frame_w: u32,
    pub frame_h: u32,
    /// Rendered size in virtual pixels.
    pub draw_w: f32,
    pub draw_h: f32,
    pub row_down: u32,
    pub row_up: u32,
    pub row_left: u32,
    pub row_right: u32,
    pub frames: u32,
    pub fps: f32,
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
    /// Starting equipment, as ids into `equipment`. Both optional.
    #[serde(default)]
    pub weapon: Option<String>,
    #[serde(default)]
    pub armor: Option<String>,
    /// Optional overworld walk sprite. If absent the character can't lead the
    /// party on the map (only the front-runner needs one).
    #[serde(default)]
    pub overworld: Option<OverworldWalk>,
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
    /// Optional equipment (ids into `equipment`), same as characters.
    #[serde(default)]
    pub weapon: Option<String>,
    #[serde(default)]
    pub armor: Option<String>,
    /// Behaviour selector; defaults to [`EnemyAi::Basic`] when omitted.
    #[serde(default)]
    pub ai: EnemyAi,
    pub xp: i32,
    pub gold: i32,
    /// Optional overworld walk sprite. When present the roaming enemy animates a
    /// directional walk on the map; otherwise it falls back to its battle idle.
    #[serde(default)]
    pub overworld: Option<OverworldWalk>,
}

/// A named group of enemies used to seed a battle.
#[derive(Clone, Debug, Deserialize)]
pub struct EncounterDef {
    pub id: String,
    /// Enemy ids; repeats allowed (e.g. two demons).
    pub enemies: Vec<String>,
}

/// One roaming enemy placed on the overworld map. When it touches the player it
/// starts the referenced [`EncounterDef`]. Its on-map appearance is taken from
/// the encounter's first enemy sprite.
#[derive(Clone, Debug, Deserialize)]
pub struct SpawnDef {
    /// Tile column / row of the spawn point.
    pub col: u32,
    pub row: u32,
    /// Encounter id started on contact.
    pub encounter: String,
}

/// A single screen (room) within a level: one ASCII tile map, its enemy spawns,
/// and links to neighbouring screens. Walking into the mid-edge opening on a
/// side with a neighbour flips to that screen.
///
/// Each row of `map` is a string of tile chars (see [`crate::overworld::Tile`]
/// for the legend). Rows may be ragged; shorter rows are grass-padded to the
/// widest row. Neighbour fields are indices into the owning level's `screens`.
#[derive(Clone, Debug, Deserialize)]
pub struct ScreenDef {
    pub map: Vec<String>,
    #[serde(default)]
    pub spawns: Vec<SpawnDef>,
    #[serde(default)]
    pub north: Option<usize>,
    #[serde(default)]
    pub south: Option<usize>,
    #[serde(default)]
    pub east: Option<usize>,
    #[serde(default)]
    pub west: Option<usize>,
}

/// A level: a marker on the world map screen plus a set of connected screens.
#[derive(Clone, Debug, Deserialize)]
pub struct LevelDef {
    pub id: String,
    pub name: String,
    /// Grid position of this level's marker on the map-select screen.
    pub node: (u32, u32),
    /// Index (into `screens`) of the screen the player enters on.
    #[serde(default)]
    pub start_screen: usize,
    /// Player start tile (col, row) within `start_screen`.
    pub start: (u32, u32),
    pub screens: Vec<ScreenDef>,
    /// Cutscene id played the first time this level is entered.
    #[serde(default)]
    pub intro_cutscene: Option<String>,
    /// Cutscene id played the first time every demon in the level is cleared.
    #[serde(default)]
    pub clear_cutscene: Option<String>,
}

/// One step of a [`CutsceneDef`]. This enum is the extension point: new kinds of
/// scripted moments (set a flag, move an actor, fade, …) are new variants.
#[derive(Clone, Debug, Deserialize)]
pub enum CutsceneStep {
    /// A line of dialogue. `portrait` is a character/enemy id whose sprite is
    /// shown beside the text.
    Say {
        #[serde(default)]
        speaker: Option<String>,
        text: String,
        #[serde(default)]
        portrait: Option<String>,
    },
    /// Add a character to the party (no-op if already recruited). This is how
    /// new party members join the story.
    Recruit { character: String },
}

/// A named, ordered script of [`CutsceneStep`]s.
#[derive(Clone, Debug, Deserialize)]
pub struct CutsceneDef {
    pub id: String,
    pub steps: Vec<CutsceneStep>,
}

/// Root of the RON data file.
#[derive(Clone, Debug, Deserialize)]
pub struct GameData {
    pub characters: Vec<CharacterDef>,
    pub enemies: Vec<EnemyDef>,
    pub skills: Vec<SkillDef>,
    /// Weapons and armor referenced by characters/enemies.
    #[serde(default)]
    pub equipment: Vec<EquipmentDef>,
    pub encounters: Vec<EncounterDef>,
    /// Character ids that make up the party at the start of the game.
    pub starting_party: Vec<String>,
    /// The levels reachable from the map screen.
    pub levels: Vec<LevelDef>,
    /// Scripted cutscenes, referenced by id from levels (and future triggers).
    #[serde(default)]
    pub cutscenes: Vec<CutsceneDef>,
}

/// Indexed, validated view of [`GameData`] for fast lookups during play.
pub struct Registry {
    pub data: GameData,
    characters: HashMap<String, usize>,
    enemies: HashMap<String, usize>,
    skills: HashMap<String, usize>,
    equipment: HashMap<String, usize>,
    encounters: HashMap<String, usize>,
    cutscenes: HashMap<String, usize>,
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
        let equipment = index(&|| data.equipment.iter().map(|c| c.id.clone()).collect());
        let encounters = index(&|| data.encounters.iter().map(|c| c.id.clone()).collect());
        let cutscenes = index(&|| data.cutscenes.iter().map(|c| c.id.clone()).collect());
        Self {
            data,
            characters,
            enemies,
            skills,
            equipment,
            encounters,
            cutscenes,
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

    pub fn equipment(&self, id: &str) -> Option<&EquipmentDef> {
        self.equipment.get(id).map(|&i| &self.data.equipment[i])
    }

    /// Apply a weapon and armor to `base`, returning the effective stats plus the
    /// derived crit / accuracy / evasion. Unknown or absent ids are skipped, so a
    /// battler with no gear just gets its base stats and zeroed attributes.
    pub fn equipped(&self, base: &Stats, weapon: Option<&str>, armor: Option<&str>) -> Equipped {
        let mut eq = Equipped {
            stats: base.clone(),
            crit: 0,
            accuracy: 0,
            evasion: 0,
        };
        for id in [weapon, armor].into_iter().flatten() {
            if let Some(item) = self.equipment(id) {
                eq.stats.attack += item.mods.attack;
                eq.stats.defense += item.mods.defense;
                eq.stats.magic += item.mods.magic;
                eq.stats.speed += item.mods.speed;
                eq.crit += item.crit;
                eq.accuracy += item.accuracy;
                eq.evasion += item.evasion;
            }
        }
        eq
    }

    pub fn encounter(&self, id: &str) -> Option<&EncounterDef> {
        self.encounters.get(id).map(|&i| &self.data.encounters[i])
    }

    pub fn cutscene(&self, id: &str) -> Option<&CutsceneDef> {
        self.cutscenes.get(id).map(|&i| &self.data.cutscenes[i])
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
        "mage" => include_bytes!("../assets/textures/entities/playables/mage.png"),
        "demon" => include_bytes!("../assets/textures/entities/monsters/demon.png"),
        "starter_sword" => include_bytes!("../assets/textures/items/starter_sword.png"),
        "starter_gear" => include_bytes!("../assets/textures/items/starter_gear.png"),
        "grass" => include_bytes!("../assets/textures/tiles/grass.png"),
        "water" => include_bytes!("../assets/textures/tiles/water.png"),
        "tree" => include_bytes!("../assets/textures/tiles/tree.png"),
        "rock" => include_bytes!("../assets/textures/tiles/rock.png"),
        "barricade" => include_bytes!("../assets/textures/tiles/barricade.png"),
        _ => return None,
    })
}

/// The embedded font atlas PNG.
pub const FONT_PNG: &[u8] = include_bytes!("../assets/textures/ui/font.png");

/// Looping battle theme (Vorbis). Embedded so the exact same track ships in the
/// native binary and the wasm bundle. Played by [`crate::audio`].
pub const BATTLE_MUSIC_OGG: &[u8] = include_bytes!("../assets/music/battle.ogg");
