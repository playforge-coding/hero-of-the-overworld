# Hero of the Overworld

A small but **extensible** turn-based JRPG written in Rust. Travel a world map,
explore tile-mapped levels, dodge (or fight) roaming demons, and win data-driven
turn-based battles — then watch a scripted cutscene recruit a new party member.

- **Rendering:** [`wgpu`](https://wgpu.rs) (WebGPU / Vulkan / Metal / DX12 / GL)
- **Windowing & input:** [`winit`](https://github.com/rust-windowing/winit) — native *and* web
- **Web build:** [`trunk`](https://trunkrs.dev) → WebAssembly
- **Audio:** looping background music via [`rodio`](https://crates.io/crates/rodio)
  (native) and an `<audio>` element (web)
- **Content:** a plain-text [RON](https://github.com/ron-rs/ron) data file — add heroes,
  enemies, skills, levels, and cutscenes without touching engine code
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
trunk serve          # dev server at http://127.0.0.1:8080
trunk build --release
```

Controls: **arrows / WASD** move, **Enter / Z / Space** confirm, **Esc / X / Backspace**
cancel or back out, **Shift / C** open the menu (leave a level).

Press Enter on the title to reach the **world map**, pick a level, and enter it. You walk
the tiled overworld freely; roaming demons chase you inside an aggro radius and start a
**battle** on contact — but you move faster than they do, so encounters can be dodged. In
battle each living party member chooses **ATTACK**, a **SKILL**, or **DEFEND**, then
everyone acts in speed order. Clear every demon in a level to mark it done on the map.

---

## Architecture

Everything is drawn as tinted textured quads in a fixed **320×180 virtual canvas** that is
letterboxed into the real window, so game code never deals with real pixels or DPI.

| Module | Responsibility |
| --- | --- |
| [`src/renderer.rs`](src/renderer.rs) | wgpu setup + an immediate-mode sprite/rect/text batcher |
| [`shaders/sprite.wgsl`](shaders/sprite.wgsl) | one instanced textured-quad pipeline for *everything* |
| [`src/data.rs`](src/data.rs) | the RON file format + indexed registries (the content DB) |
| [`src/party.rs`](src/party.rs) | the persistent, extensible party (HP/MP/XP carried between battles) |
| [`src/overworld.rs`](src/overworld.rs) | tile-mapped levels: screens, walking, camera, roaming enemies |
| [`src/battle.rs`](src/battle.rs) | the turn-based battle scene (commands → AI → resolve) |
| [`src/cutscene.rs`](src/cutscene.rs) | data-driven scripted dialogue + party recruitment |
| [`src/audio.rs`](src/audio.rs) | background music (rodio on native, `<audio>` on web; no-op if unavailable) |
| [`src/game.rs`](src/game.rs) | scene state machine (title → map → level → cutscene → battle → report) |
| [`src/input.rs`](src/input.rs) | logical buttons + frame-independent edge detection |
| [`src/app.rs`](src/app.rs) | winit `ApplicationHandler`; native + web entry |

Assets (sprite sheets, tile art, font atlas, music) are **embedded at compile time**, so the
native and web builds load content through the exact same path with no async asset fetching.

### Text

There is no font-rendering dependency. A monospace bitmap-font atlas is baked to
`assets/textures/ui/font.png` and drawn as glyph quads through the same sprite pipeline.

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

The data file also defines **skills** (physical / magical / heal, single or all targets),
**enemies** (stats, skills, AI, XP/gold rewards), **encounters** (named groups of enemies),
**levels** (a map marker plus a set of connected ASCII-tile screens with enemy spawns), and
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
binary in a borderless-fullscreen "test mode" (`HOTO_TEST_WINDOW=1`, which maps the canvas at
a clean integer scale and always owns focus), send keyboard input, take screenshots, and
assert on the result — including **template-matching the hero and demon sprites on screen**.
Because they take over the screen and mouse, they are marked `#[ignore]` and run explicitly:

```bash
cargo test --test e2e -- --ignored --test-threads=1
```

They need a display (X11 here) and the `libX11`/`libXtst` runtime libs. What they check:

| Test | Asserts |
| --- | --- |
| `boots_and_survives` | window + wgpu + data + textures come up without panicking |
| `title_screen_renders_content` | the title actually renders (not a blank frame) |
| `enter_map_then_level_via_cutscene` | title → map → intro cutscene → tiled level, each transition visibly changing the screen |
| `walking_into_demon_starts_battle` | walking into a roaming demon starts a battle; hero and demon sprites are found on screen via template match |

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
  textures/ui/font.png              # baked bitmap font
  music/battle.ogg                  # looping battle theme (embedded)
shaders/sprite.wgsl
src/                                # engine + game (see table above)
tests/
  data.rs                           # fast, display-free
  e2e.rs                            # rustautogui GUI suite
  common/                           # shared e2e helpers
  fixtures/                         # sprite templates for matching
docs/ + mkdocs.yml                  # player-facing docs site (Zensical / Material)
index.html / Trunk.toml             # web build
```

## License

**AGPL-3.0-only** — see [LICENSE](LICENSE).
