# Hero of the Overworld

A small but **extensible** turn-based JRPG written in Rust.

- **Rendering:** [`wgpu`](https://wgpu.rs) (WebGPU / Vulkan / Metal / DX12 / GL)
- **Windowing & input:** [`winit`](https://github.com/rust-windowing/winit) — native *and* web
- **Web build:** [`trunk`](https://trunkrs.dev) → WebAssembly
- **Content:** a plain-text [RON](https://github.com/ron-rs/ron) data file — add heroes and
  enemies without touching engine code
- **Tests:** fast data/logic tests plus a real end-to-end suite that drives the actual game
  window with [`rustautogui`](https://crates.io/crates/rustautogui) (keyboard + screenshots +
  template matching)

<p align="center"><em>Roland vs. a Demon — turn-based battle with a classic ATTACK / SKILL / DEFEND menu.</em></p>

---

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

Controls: **arrows / WASD** move the cursor, **Enter / Z / Space** confirm, **Esc / X** cancel.

On the title screen pick an encounter and press Enter. In battle each living party member
chooses ATTACK, a SKILL, or DEFEND; then everyone acts in speed order.

---

## Architecture

Everything is drawn as tinted textured quads in a fixed **320×180 virtual canvas** that is
letterboxed into the real window, so game code never deals with real pixels or DPI.

| Module | Responsibility |
| --- | --- |
| [`src/renderer.rs`](src/renderer.rs) | wgpu setup + an immediate-mode sprite/rect/text batcher |
| [`shaders/sprite.wgsl`](shaders/sprite.wgsl) | one instanced textured-quad pipeline for *everything* |
| [`src/data.rs`](src/data.rs) | the RON file format + indexed registries (the content DB) |
| [`src/party.rs`](src/party.rs) | the persistent, extensible party |
| [`src/battle.rs`](src/battle.rs) | the turn-based battle scene (commands → AI → resolve) |
| [`src/game.rs`](src/game.rs) | scene state machine (title ↔ battle ↔ result) |
| [`src/input.rs`](src/input.rs) | logical buttons + frame-independent edge detection |
| [`src/app.rs`](src/app.rs) | winit `ApplicationHandler`; native + web entry |

Assets (sprite sheets, font atlas) are **embedded at compile time**, so the native and web
builds load content through the exact same path with no async asset fetching.

### Text

There is no font-rendering dependency. A monospace bitmap-font atlas is baked to
`assets/textures/ui/font.png` and drawn as glyph quads through the same sprite pipeline.

---

## Extending the game (data-only)

All content lives in [`assets/data/game.ron`](assets/data/game.ron). To add a **new party
member** or **enemy** you usually only edit that file:

```ron
CharacterDef(
    id: "mage",
    name: "ELARA",
    stats: Stats(max_hp: 80, max_mp: 60, attack: 10, defense: 8, magic: 24, speed: 14),
    sprite: BattlerSprite(
        texture: "mage",                 // <- register PNG bytes in src/data.rs::embedded_texture
        frame_w: 16, frame_h: 16,
        draw_w: 48.0, draw_h: 48.0,
        idle:   AnimClip(row: 2, first_col: 0, frames: 4, fps: 5.0),
        attack: AnimClip(row: 6, first_col: 0, frames: 4, fps: 14.0),
    ),
    skills: ["mend", "power_strike"],
),
```

Then add it to `starting_party` (or recruit it at runtime with `Party::recruit`). The battle
system iterates whatever heroes are present, so a **second or third character assisting in
battle needs no engine changes** — the party is just a `Vec`.

Adding new art is the only code touch: register the PNG once in
[`embedded_texture`](src/data.rs) so it ships inside the wasm bundle too.

The data file also defines **skills** (physical / magical / heal, single or all targets),
**enemies** (stats, skills, AI, XP/gold rewards) and **encounters** (named groups of enemies).

### The sprite sheets

Both provided sheets are 16×16 grids:

- `swordsman.png` — 5×12. Rows 0–3 walk (down/up/right/left), rows 4–7 attack.
  The battler uses the *walk-right* row as its idle and the *attack-right* row when striking.
- `demon.png` — 6×8.

---

## Tests

Two layers:

**1. Fast data/logic tests** — run on every `cargo test`, no display needed. They validate the
RON parses, every skill/enemy/encounter/texture cross-reference resolves, and the party
mechanics (build, recruit, level-up) behave.

```bash
cargo test --test data
```

**2. End-to-end GUI tests (`rustautogui`)** — drive the *real* game window. They launch the
binary in a borderless-fullscreen "test mode" (`HOTO_TEST_WINDOW=1`, a clean 6× scale of the
canvas that always owns focus), send keyboard input, take screenshots, and assert on the
result — including **template-matching the hero and demon sprites on screen**. Because they
take over the screen and mouse, they are marked `#[ignore]` and run explicitly:

```bash
cargo test --test e2e -- --ignored --test-threads=1
```

They need a display (X11 here) and the `libX11`/`libXtst` runtime libs. What they check:

| Test | Asserts |
| --- | --- |
| `boots_and_survives` | window + wgpu + data + textures come up without panicking |
| `title_screen_renders_content` | the title actually renders (not a blank frame) |
| `enter_starts_battle` | pressing Enter transitions title → battle |
| `battle_shows_hero_and_demon_sprites` | both sprites are found on screen via template match |
| `attack_command_resolves` | issuing ATTACK animates and changes the scene |

> A subtle bug the e2e suite caught: automation tools deliver a key press and release within
> a single frame, so classic `down && !previous` edge detection (sampled once per update)
> misses them. [`input.rs`](src/input.rs) latches presses as events arrive, which fixes both
> the tests and any fast real input.

---

## Project layout

```
assets/
  data/game.ron                     # the content database
  textures/entities/...             # sprite sheets
  textures/ui/font.png              # baked bitmap font
shaders/sprite.wgsl
src/                                # engine + game (see table above)
tests/
  data.rs                           # fast, display-free
  e2e.rs                            # rustautogui GUI suite
  fixtures/                         # sprite templates for matching
index.html / Trunk.toml             # web build
```

## License

MIT — see [LICENSE](LICENSE).
