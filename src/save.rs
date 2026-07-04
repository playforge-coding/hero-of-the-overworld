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

    out
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

    Some(SaveData {
        gold,
        members,
        cleared,
        played_cutscenes,
        levels,
        location,
    })
}

// ---- Platform storage -------------------------------------------------------

/// Load and decode the saved game, or `None` if there is no (valid) save.
pub fn load() -> Option<SaveData> {
    let bytes = storage::read()?;
    from_bytes(&bytes)
}

/// Encode and persist `data`. Best-effort: failures are logged, never fatal —
/// losing a save must not crash the game.
pub fn store(data: &SaveData) {
    storage::write(&to_bytes(data));
}

/// Erase any existing save (used when starting a brand-new game).
pub fn clear() {
    storage::remove();
}

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::path::PathBuf;

    /// `<data_dir>/hero-of-the-overworld/save.bin`, e.g. `~/.local/share/...`
    /// on Linux or `%APPDATA%\...` on Windows.
    fn path() -> Option<PathBuf> {
        Some(
            dirs::data_dir()?
                .join("hero-of-the-overworld")
                .join("save.bin"),
        )
    }

    pub fn read() -> Option<Vec<u8>> {
        std::fs::read(path()?).ok()
    }

    pub fn write(bytes: &[u8]) {
        let Some(p) = path() else {
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

    pub fn remove() {
        if let Some(p) = path() {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod storage {
    use sapp_jsutils::JsObject;

    // Implemented in JS by the `hoto_storage` miniquad plugin (see
    // `hoto_storage.js`), backed by IndexedDB. `load` returns whatever the plugin
    // preloaded into its cache at startup (nil if there is no save yet).
    extern "C" {
        fn hoto_storage_save(data: JsObject);
        fn hoto_storage_load() -> JsObject;
        fn hoto_storage_clear();
    }

    pub fn read() -> Option<Vec<u8>> {
        let obj = unsafe { hoto_storage_load() };
        if obj.is_nil() || obj.is_undefined() {
            return None;
        }
        let mut buf = Vec::new();
        obj.to_byte_buffer(&mut buf);
        (!buf.is_empty()).then_some(buf)
    }

    pub fn write(bytes: &[u8]) {
        let obj = JsObject::buffer(bytes);
        unsafe { hoto_storage_save(obj) };
    }

    pub fn remove() {
        unsafe { hoto_storage_clear() };
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
