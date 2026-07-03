# Hero of the Overworld

A small but **extensible** turn-based JRPG written in Rust. Travel a world map,
explore tile-mapped levels, dodge (or fight) roaming monsters, and win data-driven
turn-based battles — then watch a scripted cutscene recruit a new party member.

- **Engine:** [`macroquad`](https://macroquad.rs) — one small crate for the window, GL
  rendering, input, text, and audio, on both native (OpenGL) and web (WebGL)
- **Web build:** `cargo build --target wasm32-unknown-unknown` → a single `hero.wasm`
  loaded by macroquad's JS bundle (no wasm-bindgen/Trunk)
- **Audio:** looping background music via [`macroquad::audio`](https://docs.rs/macroquad/latest/macroquad/audio/)
  (native + web)
- **Content:** a plain-text [RON](https://github.com/ron-rs/ron) data file — add heroes,
  enemies, skills, levels, and cutscenes without touching engine code
- **Saves:** progress persists automatically in a small custom binary format — a
  file (via [`dirs`](https://crates.io/crates/dirs)) natively, IndexedDB on the web
- **Tests:** fast data/logic tests plus a real end-to-end suite that drives the actual game
  window with [`rustautogui`](https://crates.io/crates/rustautogui) (keyboard + screenshots +
  template matching)

---

## Play

- **In your browser:** https://playforge-coding.github.io/hero-of-the-overworld/play/
  (published to GitHub Pages by [`.github/workflows/docs.yml`](.github/workflows/docs.yml)).
- **Prebuilt binaries:** self-contained Windows/macOS/Linux builds are attached to
  each [tagged release](https://github.com/playforge-coding/hero-of-the-overworld/releases)
  by [`.github/workflows/release.yml`](.github/workflows/release.yml).
- **From source:** see below.

## Running

### Native

```bash
cargo run            # debug
cargo run --release  # smooth
```

### Web

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --bin hero
# then serve index.html + hero.wasm + mq_js_bundle.js, plus the save-file glue
# sapp_jsutils.js + hoto_storage.js (see docs/getting-started)
```

Controls: **arrows / WASD** move, **Enter / Z / Space** confirm, **Esc / X / Backspace**
cancel or back out, **Shift / C** open the menu (leave a level). A **gamepad**
works too (native builds) — D-pad/stick to move, A confirm, B cancel; plug in
several and each controls a party member in battle. See [`docs/controls.md`](docs/controls.md).

Press Enter on the title to reach the **world map**, pick a level, and enter it. You walk
the tiled overworld freely; roaming enemies chase you inside an aggro radius and start a
**battle** on contact — but you move faster than they do, so encounters can be dodged. In
battle each living party member chooses **ATTACK**, a **SKILL**, or **DEFEND**, then
everyone acts in speed order. Hits can **miss** or land a **critical** for extra damage,
and each hero's **weapon and armor** tilt those odds. Clear every demon in a level to
mark it done on the map and **unlock the next** — progression is linear. Your party,
clears, and per-level progress are **saved automatically** and resume on the next launch.

---

## Architecture

Everything is drawn into a fixed **320×180 virtual canvas** (a macroquad render target)
that is letterboxed into the real window, so game code never deals with real pixels or DPI.

| Module | Responsibility |
| --- | --- |
| [`src/renderer.rs`](src/renderer.rs) | thin macroquad wrapper: a virtual-resolution sprite/rect/text draw queue, letterboxed |
| [`src/data.rs`](src/data.rs) | the RON file format + indexed registries (the content DB) |
| [`src/party.rs`](src/party.rs) | the persistent, extensible party (HP/MP/XP carried between battles) |
| [`src/overworld.rs`](src/overworld.rs) | tile-mapped levels: screens, walking, camera, roaming enemies |
| [`src/battle.rs`](src/battle.rs) | the turn-based battle scene (commands → AI → resolve) |
| [`src/cutscene.rs`](src/cutscene.rs) | data-driven scripted dialogue + party recruitment |
| [`src/audio.rs`](src/audio.rs) | background music via `macroquad::audio` (native + web) |
| [`src/save.rs`](src/save.rs) | custom binary save format + storage (native file via `dirs`, web IndexedDB) |
| [`src/game.rs`](src/game.rs) | scene state machine (title → map → level → cutscene → battle → report) |
| [`src/input.rs`](src/input.rs) | logical buttons polled from the keyboard and gamepads (`gilrs`) each frame |
| [`src/app.rs`](src/app.rs) | the macroquad main loop + window config |

The `Renderer` keeps the same small API the game modules were written against (a queue of
`draw_rect` / `draw_sprite` / `draw_text` calls replayed each frame), so the rendering
backend swapped from a custom wgpu engine to macroquad without touching any game logic.

Assets (sprite sheets, tile art, font, music) are **embedded at compile time**, so the
native and web builds load content through the exact same path with no async asset fetching.

### Text

Text uses macroquad's built-in TrueType rasteriser with the embedded pixel font
`assets/textures/ui/font.ttf`. Glyphs are drawn *after* the low-res scene is scaled up, at
the final on-screen size, so the UI text stays crisp rather than being magnified from a
tiny atlas.

---

## Extending the game (data-only)

All content lives in [`assets/data/game.ron`](assets/data/game.ron). To add a **new party
member** or **enemy** you usually only edit that file. A character can use its own sprite
sheet (the bundled mage, ELARA, has her own purple `mage.png`) or reuse and recolour an
existing one with a `tint`:

```ron
CharacterDef(
    id: "mage",
    name: "ELARA",
    stats: Stats(max_hp: 78, max_mp: 60, attack: 10, defense: 9, magic: 26, speed: 13),
    sprite: BattlerSprite(
        texture: "mage",                 // any key registered in src/data.rs::embedded_texture
        frame_w: 16, frame_h: 16,
        draw_w: 48.0, draw_h: 48.0,
        // tint: Some((150, 100, 220)),  // optionally recolour a shared sheet instead
        idle:   AnimClip(row: 2, first_col: 0, frames: 4, fps: 5.0),
        attack: AnimClip(row: 7, first_col: 0, frames: 5, fps: 14.0),
    ),
    skills: ["firebolt", "frost", "mend"],
),
```

Add the id to `starting_party` to have it join at the start, or recruit it mid-game from a
cutscene `Recruit` step (that's how ELARA joins after GREENWOOD is cleared). The battle
system iterates whatever heroes are present, so a **second or third character assisting in
battle needs no engine changes** — the party is just a `Vec`.

Adding genuinely new art is the only code touch: register the PNG once in
[`embedded_texture`](src/data.rs) so it ships inside the wasm bundle too.

The data file also defines **skills** (physical / magical / heal, single or all targets,
each with a description), **equipment** (weapons and armor with stat bonuses, crit /
accuracy / evasion, and descriptions — heroes and enemies equip them by id), **enemies**
(stats, skills, AI, XP/gold rewards), **encounters** (named groups of enemies), **levels**
(a map marker plus a set of connected ASCII-tile screens with enemy spawns), and
**cutscenes** (scripted dialogue lines and recruits).

### The sprite sheets

The battler sheets are grids of 16×16 frames:

- `swordsman.png` — 5×12. Rows 0–3 walk (down/up/right/left), rows 4–7 attack.
  The overworld walk uses rows 0–3; the battler uses the *walk-right* row as its idle and
  the *attack-right* row when striking.
- `mage.png` — 6×8 (ELARA). Rows 0–3 walk (4 frames), rows 4–7 cast (5 frames). The cast
  rows are ordered down/up/*left/right* — swapping left and right versus the walk rows.
- `demon.png` — 6×8. Same convention as the swordsman; the roaming overworld demon uses the walk rows.

Tiles (`grass`, `water`, `tree`, `rock`, `barricade`) are their own 16×16 PNGs under
`assets/textures/tiles/`.

---

## Tests

Two layers:

**1. Fast data/logic tests** — run on every `cargo test`, no display needed. They validate the
RON parses, every skill/enemy/encounter/texture/cutscene cross-reference resolves, that every
level's screens are linked and traversable, and that the party mechanics (build, recruit,
level-up) behave.

```bash
cargo test --test data
```

**2. End-to-end GUI tests (`rustautogui`)** — drive the *real* game window. They launch the
binary in a fullscreen "test mode" (`HOTO_TEST_WINDOW=1`, so it owns the whole screen and
holds focus), send keyboard input, take screenshots, and assert on the result — including
**template-matching the hero and demon sprites on screen**. Because they take over the screen
and mouse, they are marked `#[ignore]` and run explicitly:

```bash
cargo test --test e2e -- --ignored --test-threads=1
```

They need a display (X11 here) and the `libX11`/`libXtst` runtime libs. What they check:

| Test | Asserts |
| --- | --- |
| `boots_and_survives` | window + macroquad + data + textures come up without panicking |
| `title_screen_renders_content` | the title actually renders (not a blank frame) |
| `enter_map_then_level_via_cutscene` | title → map → intro cutscene → tiled level, each transition visibly changing the screen |
| `walking_into_slime_starts_battle` | walking into a roaming enemy starts a battle; hero and slime sprites are found on screen via template match |

> A subtle bug the e2e suite caught: automation tools deliver a key press and release within
> a single frame, so classic `down && !previous` edge detection (sampled once per update)
> misses them. [`input.rs`](src/input.rs) latches presses as events arrive, which fixes both
> the tests and any fast real input.

---

## Project layout

```
assets/
  data/game.ron                     # the content database
  textures/entities/playables/...   # party sprite sheets
  textures/entities/monsters/...    # enemy sprite sheets
  textures/tiles/...                # overworld tile art
  textures/ui/font.ttf              # embedded UI font (TrueType)
  music/battle.ogg                  # looping battle theme (embedded)
src/                                # engine + game (see table above)
tests/
  data.rs                           # fast, display-free
  e2e.rs                            # rustautogui GUI suite
  common/                           # shared e2e helpers
  fixtures/                         # sprite templates for matching
docs/ + mkdocs.yml                  # player-facing docs site (Zensical / Material)
index.html                          # macroquad web page (loads hero.wasm)
sapp_jsutils.js + hoto_storage.js   # web save glue (byte marshalling + IndexedDB)
```

## License

**AGPL-3.0-only** — see [LICENSE](LICENSE).
