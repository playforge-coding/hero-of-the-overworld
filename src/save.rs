//! Persistent save files in a small, hand-rolled binary format.
//!
//! The whole game state that survives a session — the party (levels, XP, live
//! HP/MP, equipment, gold), which levels are cleared, which cutscenes have
//! played, and the per-level "which demons have I beaten" progress — is packed
//! into [`SaveData`] and serialized by [`to_bytes`] / [`from_bytes`].
//!
//! The format is deliberately custom (little-endian, length-prefixed) rather
//! than a serde framework, so we own every byte and can evolve it explicitly via
//! the [`VERSION`] tag. It is content-agnostic: [`Game`](crate::game::Game) knows
//! how to turn its live state into a `SaveData` and back.
//!
//! Storage is platform-split behind [`load`] / [`store`]:
//!   - **native** — a file under the OS data dir (via the `dirs` crate).
//!   - **web** — IndexedDB, reached through a tiny miniquad JS plugin (see
//!     `hoto_storage.js` / `index.html`).

/// Four magic bytes at the head of every save so we can reject foreign data.
const MAGIC: [u8; 4] = *b"HOTO";
/// Bump when the layout below changes incompatibly; [`from_bytes`] refuses other
/// versions rather than misreading them.
const VERSION: u16 = 2;

/// How many independent save slots the player can keep. Each is a wholly separate
/// playthrough; the game autosaves into whichever slot is active (see
/// [`Game`](crate::game::Game)). Slots are addressed by index `0..SLOTS`.
pub const SLOTS: usize = 3;

/// One party member's mutable, save-worthy state. The immutable bits (name,
/// sprite, known skills) are rebuilt from the registry via `def_id` on load, so
/// only what actually changes during play is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedMember {
    pub def_id: String,
    pub level: i32,
    pub xp: i32,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub speed: i32,
    pub weapon: Option<String>,
    pub armor: Option<String>,
}

/// In-progress state of one level: for each screen, which of its spawned enemies
/// have been defeated. Keyed by level `id` so it survives level reordering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedLevel {
    pub id: String,
    /// `screens[s][e]` = enemy `e` on screen `s` has been beaten.
    pub screens: Vec<Vec<bool>>,
}

/// Where the player was standing when they last saved, so a resumed session
/// drops them back at the exact spot rather than on the world map. `None` when
/// the save was taken from the map screen (not inside a level).
#[derive(Clone, Debug, PartialEq)]
pub struct SavedLocation {
    /// Level `id` the player was in (survives level reordering).
    pub level_id: String,
    /// Which screen within the level.
    pub screen: usize,
    /// Player feet-center position in world pixels.
    pub x: f32,
    pub y: f32,
}

/// The complete persisted game state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveData {
    pub gold: i32,
    pub members: Vec<SavedMember>,
    /// Parallel to `reg.data.levels`: whether each level is fully cleared.
    pub cleared: Vec<bool>,
    pub played_cutscenes: Vec<String>,
    pub levels: Vec<SavedLevel>,
    /// The player's in-level position, if they saved while walking a level.
    pub location: Option<SavedLocation>,
    /// Owned, unequipped equipment ids (the party's [bag](crate::party::Party::bag)).
    /// Appended at the end of the format and read leniently, so pre-bag saves
    /// (which simply lack this trailing section) still load with an empty bag.
    pub bag: Vec<String>,
    /// Input mapping: which player number the keyboard is on, and one per gamepad
    /// (by connection order). Trailing and lenient like `bag`; absent → defaults
    /// (keyboard on player 0, each pad its own player).
    pub input_keyboard: u32,
    pub input_gamepads: Vec<u32>,
    /// Owned consumable items as `(id, count)` pairs (the party's
    /// [items](crate::party::Party::items)). Read leniently so pre-item saves
    /// (which lack it) load with no items.
    pub items: Vec<(String, u32)>,
    /// Per-level opened-chest grids, reusing [`SavedLevel`] where `screens[s][c]`
    /// = chest `c` on screen `s` has been opened. A trailing section, read
    /// leniently so pre-chest saves (which lack it) load with no chests looted.
    pub chest_levels: Vec<SavedLevel>,
    /// Per-level slain-mimic grids, same shape as [`chest_levels`](Self::chest_levels):
    /// `screens[s][m]` = mimic `m` on screen `s` has been beaten. A trailing
    /// section, read the same lenient way.
    pub mimic_levels: Vec<SavedLevel>,
    /// The story **chapter** the party is in (1-based). The final trailing field;
    /// absent (a pre-chapter save ends before it) → chapter 1. A `0` here (only a
    /// default-constructed [`SaveData`] writes that) is normalised to 1 by the game.
    pub chapter: u32,
}

// ---- Encoding ---------------------------------------------------------------

fn put_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_bool(out: &mut Vec<u8>, v: bool) {
    out.push(v as u8);
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    put_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

fn put_opt_str(out: &mut Vec<u8>, s: &Option<String>) {
    match s {
        Some(s) => {
            put_bool(out, true);
            put_str(out, s);
        }
        None => put_bool(out, false),
    }
}

/// Serialize `data` to the binary save format.
pub fn to_bytes(data: &SaveData) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    put_u16(&mut out, VERSION);

    put_i32(&mut out, data.gold);

    put_u32(&mut out, data.members.len() as u32);
    for m in &data.members {
        put_str(&mut out, &m.def_id);
        for v in [
            m.level, m.xp, m.hp, m.mp, m.max_hp, m.max_mp, m.attack, m.defense, m.magic, m.speed,
        ] {
            put_i32(&mut out, v);
        }
        put_opt_str(&mut out, &m.weapon);
        put_opt_str(&mut out, &m.armor);
    }

    put_u32(&mut out, data.cleared.len() as u32);
    for &c in &data.cleared {
        put_bool(&mut out, c);
    }

    put_u32(&mut out, data.played_cutscenes.len() as u32);
    for id in &data.played_cutscenes {
        put_str(&mut out, id);
    }

    put_u32(&mut out, data.levels.len() as u32);
    for lv in &data.levels {
        put_str(&mut out, &lv.id);
        put_u32(&mut out, lv.screens.len() as u32);
        for screen in &lv.screens {
            put_u32(&mut out, screen.len() as u32);
            for &d in screen {
                put_bool(&mut out, d);
            }
        }
    }

    match &data.location {
        Some(loc) => {
            put_bool(&mut out, true);
            put_str(&mut out, &loc.level_id);
            put_u32(&mut out, loc.screen as u32);
            put_f32(&mut out, loc.x);
            put_f32(&mut out, loc.y);
        }
        None => put_bool(&mut out, false),
    }

    // Trailing, back-compatibly optional: the party's owned-item bag.
    put_u32(&mut out, data.bag.len() as u32);
    for id in &data.bag {
        put_str(&mut out, id);
    }

    // Trailing, back-compatibly optional: the input-source → player mapping.
    put_u32(&mut out, data.input_keyboard);
    put_u32(&mut out, data.input_gamepads.len() as u32);
    for &p in &data.input_gamepads {
        put_u32(&mut out, p);
    }

    // Trailing, back-compatibly optional: the party's consumable item stash.
    put_u32(&mut out, data.items.len() as u32);
    for (id, count) in &data.items {
        put_str(&mut out, id);
        put_u32(&mut out, *count);
    }

    // Trailing, back-compatibly optional: opened-chest and slain-mimic grids,
    // each an id-keyed list of per-screen bool grids (same shape as `levels`).
    put_levels(&mut out, &data.chest_levels);
    put_levels(&mut out, &data.mimic_levels);

    // Trailing, back-compatibly optional: the story chapter (final field).
    put_u32(&mut out, data.chapter);

    out
}

/// Write an id-keyed list of per-screen bool grids (the shared shape of the
/// enemy / chest / mimic progress sections).
fn put_levels(out: &mut Vec<u8>, levels: &[SavedLevel]) {
    put_u32(out, levels.len() as u32);
    for lv in levels {
        put_str(out, &lv.id);
        put_u32(out, lv.screens.len() as u32);
        for screen in &lv.screens {
            put_u32(out, screen.len() as u32);
            for &d in screen {
                put_bool(out, d);
            }
        }
    }
}

// ---- Decoding ---------------------------------------------------------------

/// A bounds-checked cursor over the save bytes. Every read returns `None` on
/// truncation, so a corrupt or partial file degrades to "no save" instead of a
/// panic.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let slice = self.buf.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    /// Bytes not yet consumed. Used to tell a *missing* trailing field (nothing
    /// left → use its default) from one that is *present but truncated* (some
    /// bytes left, but too few → a corrupt save that must be rejected).
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn bool(&mut self) -> Option<bool> {
        Some(self.take(1)?[0] != 0)
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).ok()
    }

    fn opt_string(&mut self) -> Option<Option<String>> {
        if self.bool()? {
            Some(Some(self.string()?))
        } else {
            Some(None)
        }
    }
}

/// Read one trailing id-keyed list of per-screen bool grids (chest/mimic
/// progress), the lenient counterpart of [`put_levels`]. A missing leading count
/// (an older save that ends before this section) yields an empty list rather than
/// a decode failure; a count that is present but truncated fails like any other
/// malformed field. `Some(_)` on success (possibly empty), `None` on corruption.
fn read_levels(r: &mut Reader) -> Option<Vec<SavedLevel>> {
    let count = match r.u32() {
        Some(n) => n as usize,
        None => return Some(Vec::new()),
    };
    let mut levels = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let id = r.string()?;
        let screen_count = r.u32()? as usize;
        let mut screens = Vec::with_capacity(screen_count.min(1024));
        for _ in 0..screen_count {
            let n = r.u32()? as usize;
            let mut screen = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                screen.push(r.bool()?);
            }
            screens.push(screen);
        }
        levels.push(SavedLevel { id, screens });
    }
    Some(levels)
}

/// Parse save bytes produced by [`to_bytes`]. Returns `None` for anything that
/// isn't a well-formed save of the current [`VERSION`] (foreign magic, wrong
/// version, truncation, bad UTF-8), so callers can just start a new game.
pub fn from_bytes(bytes: &[u8]) -> Option<SaveData> {
    let mut r = Reader::new(bytes);
    if r.take(4)? != MAGIC {
        return None;
    }
    if r.u16()? != VERSION {
        return None;
    }

    let gold = r.i32()?;

    let member_count = r.u32()? as usize;
    let mut members = Vec::with_capacity(member_count.min(64));
    for _ in 0..member_count {
        let def_id = r.string()?;
        let level = r.i32()?;
        let xp = r.i32()?;
        let hp = r.i32()?;
        let mp = r.i32()?;
        let max_hp = r.i32()?;
        let max_mp = r.i32()?;
        let attack = r.i32()?;
        let defense = r.i32()?;
        let magic = r.i32()?;
        let speed = r.i32()?;
        let weapon = r.opt_string()?;
        let armor = r.opt_string()?;
        members.push(SavedMember {
            def_id,
            level,
            xp,
            hp,
            mp,
            max_hp,
            max_mp,
            attack,
            defense,
            magic,
            speed,
            weapon,
            armor,
        });
    }

    let cleared_count = r.u32()? as usize;
    let mut cleared = Vec::with_capacity(cleared_count.min(1024));
    for _ in 0..cleared_count {
        cleared.push(r.bool()?);
    }

    let cutscene_count = r.u32()? as usize;
    let mut played_cutscenes = Vec::with_capacity(cutscene_count.min(1024));
    for _ in 0..cutscene_count {
        played_cutscenes.push(r.string()?);
    }

    let level_count = r.u32()? as usize;
    let mut levels = Vec::with_capacity(level_count.min(1024));
    for _ in 0..level_count {
        let id = r.string()?;
        let screen_count = r.u32()? as usize;
        let mut screens = Vec::with_capacity(screen_count.min(1024));
        for _ in 0..screen_count {
            let enemy_count = r.u32()? as usize;
            let mut screen = Vec::with_capacity(enemy_count.min(4096));
            for _ in 0..enemy_count {
                screen.push(r.bool()?);
            }
            screens.push(screen);
        }
        levels.push(SavedLevel { id, screens });
    }

    let location = if r.bool()? {
        let level_id = r.string()?;
        let screen = r.u32()? as usize;
        let x = r.f32()?;
        let y = r.f32()?;
        Some(SavedLocation {
            level_id,
            screen,
            x,
            y,
        })
    } else {
        None
    };

    // The bag is a trailing addition: a save written before it simply ends here,
    // so a missing count reads as an empty bag rather than a decode failure.
    let bag = match r.u32() {
        Some(n) => {
            let mut bag = Vec::with_capacity((n as usize).min(1024));
            for _ in 0..n {
                bag.push(r.string()?);
            }
            bag
        }
        None => Vec::new(),
    };

    // The input mapping is a further trailing addition, read the same lenient way:
    // absent (older save) → defaults.
    let (input_keyboard, input_gamepads) = match r.u32() {
        Some(kb) => {
            let n = r.u32()? as usize;
            let mut gamepads = Vec::with_capacity(n.min(64));
            for _ in 0..n {
                gamepads.push(r.u32()?);
            }
            (kb, gamepads)
        }
        None => (0, Vec::new()),
    };

    // The item stash is the final trailing addition, read the same lenient way:
    // absent (a pre-item save) → no items.
    let items = match r.u32() {
        Some(n) => {
            let mut items = Vec::with_capacity((n as usize).min(1024));
            for _ in 0..n {
                let id = r.string()?;
                let count = r.u32()?;
                items.push((id, count));
            }
            items
        }
        None => Vec::new(),
    };

    // The chest and mimic progress grids are the last two trailing additions,
    // read the same lenient way: absent (a pre-chest save) → nothing looted/slain.
    let chest_levels = read_levels(&mut r)?;
    let mimic_levels = read_levels(&mut r)?;

    // The chapter is the final trailing field. Absent (a pre-chapter save ends
    // before it) → chapter 1; present → read strictly, so a save truncated *within*
    // this field is rejected rather than silently defaulting.
    let chapter = if r.remaining() == 0 { 1 } else { r.u32()? };

    Some(SaveData {
        gold,
        members,
        cleared,
        played_cutscenes,
        levels,
        location,
        bag,
        input_keyboard,
        input_gamepads,
        items,
        chest_levels,
        mimic_levels,
        chapter,
    })
}

// ---- Platform storage -------------------------------------------------------

/// Load and decode the game saved in `slot`, or `None` if that slot is empty or
/// holds an invalid save. Slots are `0..SLOTS`.
pub fn load(slot: usize) -> Option<SaveData> {
    let bytes = storage::read(slot)?;
    from_bytes(&bytes)
}

/// Encode and persist `data` into `slot`. Best-effort: failures are logged, never
/// fatal — losing a save must not crash the game.
pub fn store(slot: usize, data: &SaveData) {
    storage::write(slot, &to_bytes(data));
}

/// Erase the save in `slot` (used to free it for a new game).
pub fn clear(slot: usize) {
    storage::remove(slot);
}

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::path::PathBuf;

    /// `<data_dir>/hero-of-the-overworld/save-<slot>.bin`, e.g. `~/.local/share/...`
    /// on Linux or `%APPDATA%\...` on Windows. Each slot is its own file, so the
    /// slots are wholly independent playthroughs.
    fn path(slot: usize) -> Option<PathBuf> {
        Some(
            dirs::data_dir()?
                .join("hero-of-the-overworld")
                .join(format!("save-{slot}.bin")),
        )
    }

    pub fn read(slot: usize) -> Option<Vec<u8>> {
        std::fs::read(path(slot)?).ok()
    }

    pub fn write(slot: usize, bytes: &[u8]) {
        let Some(p) = path(slot) else {
            log::warn!("no data dir available; not saving");
            return;
        };
        if let Some(parent) = p.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("could not create save dir {}: {e}", parent.display());
                return;
            }
        }
        if let Err(e) = std::fs::write(&p, bytes) {
            log::warn!("could not write save {}: {e}", p.display());
        }
    }

    pub fn remove(slot: usize) {
        if let Some(p) = path(slot) {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod storage {
    use sapp_jsutils::JsObject;

    // Implemented in JS by the `hoto_storage` miniquad plugin (see
    // `hoto_storage.js`), backed by IndexedDB. Each function takes a slot index so
    // the several slots persist independently. `load` returns whatever the plugin
    // preloaded into its cache at startup (nil if that slot is empty).
    extern "C" {
        fn hoto_storage_save(slot: u32, data: JsObject);
        fn hoto_storage_load(slot: u32) -> JsObject;
        fn hoto_storage_clear(slot: u32);
    }

    pub fn read(slot: usize) -> Option<Vec<u8>> {
        let obj = unsafe { hoto_storage_load(slot as u32) };
        if obj.is_nil() || obj.is_undefined() {
            return None;
        }
        let mut buf = Vec::new();
        obj.to_byte_buffer(&mut buf);
        (!buf.is_empty()).then_some(buf)
    }

    pub fn write(slot: usize, bytes: &[u8]) {
        let obj = JsObject::buffer(bytes);
        unsafe { hoto_storage_save(slot as u32, obj) };
    }

    pub fn remove(slot: usize) {
        unsafe { hoto_storage_clear(slot as u32) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SaveData {
        SaveData {
            gold: 1234,
            members: vec![
                SavedMember {
                    def_id: "swordsman".into(),
                    level: 3,
                    xp: 17,
                    hp: 40,
                    mp: 5,
                    max_hp: 56,
                    max_mp: 8,
                    attack: 14,
                    defense: 9,
                    magic: 3,
                    speed: 11,
                    weapon: Some("starter_sword".into()),
                    armor: None,
                },
                SavedMember {
                    def_id: "mage".into(),
                    level: 2,
                    xp: 3,
                    hp: 0,
                    mp: 20,
                    max_hp: 28,
                    max_mp: 24,
                    attack: 5,
                    defense: 4,
                    magic: 16,
                    speed: 9,
                    weapon: None,
                    armor: Some("starter_gear".into()),
                },
            ],
            cleared: vec![true, false, false],
            played_cutscenes: vec!["intro_greenwood".into(), "recruit_mage".into()],
            levels: vec![
                SavedLevel {
                    id: "greenwood".into(),
                    screens: vec![vec![true, true], vec![false]],
                },
                SavedLevel {
                    id: "stone_pass".into(),
                    screens: vec![vec![]],
                },
            ],
            location: Some(SavedLocation {
                level_id: "greenwood".into(),
                screen: 1,
                x: 123.5,
                y: 48.0,
            }),
            bag: vec!["iron_sword".into(), "leather_armor".into()],
            input_keyboard: 0,
            input_gamepads: vec![1, 2],
            items: vec![("potion".into(), 3), ("bomb".into(), 1)],
            chest_levels: vec![SavedLevel {
                id: "greenwood".into(),
                screens: vec![vec![true], vec![], vec![false]],
            }],
            mimic_levels: vec![SavedLevel {
                id: "greenwood".into(),
                screens: vec![vec![], vec![true]],
            }],
            chapter: 2,
        }
    }

    #[test]
    fn round_trips() {
        let data = sample();
        let bytes = to_bytes(&data);
        let back = from_bytes(&bytes).expect("valid save decodes");
        assert_eq!(data, back);
    }

    #[test]
    fn empty_save_round_trips() {
        let data = SaveData::default();
        assert_eq!(from_bytes(&to_bytes(&data)), Some(data));
    }

    #[test]
    fn pre_bag_save_loads_with_defaults() {
        // A save written before the trailing bag + input + items + chest + mimic +
        // chapter sections existed simply ends after `location`. With everything
        // empty those sections are six zero counts — bag (0) + keyboard (0) +
        // gamepad count (0) + item count (0) + chest count (0) + mimic count (0) =
        // 24 bytes — followed by the 4-byte chapter, 28 bytes in all. Dropping them
        // yields exactly such an older save, which must still decode — to empty bag,
        // default (all-zero) input, no items, no looted chests / slain mimics, and
        // chapter 1 — rather than failing.
        let mut data = sample();
        data.bag = Vec::new();
        data.input_keyboard = 0;
        data.input_gamepads = Vec::new();
        data.items = Vec::new();
        data.chest_levels = Vec::new();
        data.mimic_levels = Vec::new();
        data.chapter = 1;
        let bytes = to_bytes(&data);
        let old = &bytes[..bytes.len() - 28];
        assert_eq!(from_bytes(old), Some(data));
    }

    #[test]
    fn rejects_foreign_or_truncated_data() {
        assert!(from_bytes(b"").is_none());
        assert!(from_bytes(b"NOPE\x01\x00").is_none());
        let good = to_bytes(&sample());
        // Any prefix shorter than the whole thing must not panic and must fail.
        assert!(from_bytes(&good[..good.len() - 1]).is_none());
    }

    #[test]
    fn rejects_wrong_version() {
        let mut bytes = to_bytes(&SaveData::default());
        bytes[4] = 0xFF; // corrupt the version tag
        assert!(from_bytes(&bytes).is_none());
    }
}
