---
comments: true
---

# Credits

Hero of the Overworld is built on a stack of excellent open-source Rust crates.
Thank you to their authors and maintainers.

| Crate | Role |
| ----- | ---- |
| [macroquad](https://crates.io/crates/macroquad) | Window, GL rendering, input, text, and audio — native (OpenGL) and web (WebGL) |
| [glam](https://crates.io/crates/glam) | Vector / matrix math |
| [ron](https://crates.io/crates/ron) + [serde](https://crates.io/crates/serde) | Parsing the per-entity RON content files |
| [include_dir](https://crates.io/crates/include_dir) | Baking the whole `assets/data` content tree into the binary at compile time |
| [log](https://crates.io/crates/log) + [env_logger](https://crates.io/crates/env_logger) | Logging |
| [image](https://crates.io/crates/image) | Loading screenshots in the end-to-end test suite |
| [rustautogui](https://crates.io/crates/rustautogui) | Driving the real window in the end-to-end test suite |

## Licensing

- **Game code** is licensed under **AGPL-3.0-only** (see the `LICENSE` file in the
  repository).
- **This documentation** is licensed **CC BY-NC-SA 4.0**.

## Art

The sprites are [MiniWorld Sprites](https://opengameart.org/content/miniworld-sprites)
by [Shade](https://opengameart.org/users/shade-1) on OpenGameArt.org. They are
released under **CC0** (public domain), but Shade has said he'd appreciate credit —
so, thank you, Shade!

Character art is stored as grids of 16×16 frames; the game slices out the walk
rows (for moving on the overworld) and idle/attack rows (for battle) per entity,
and the overworld tiles — grass, water, trees, and rocks — share the same 16×16
footprint.

### Dialogue portraits

The bespoke character busts shown in dialogue boxes (`assets/textures/ui/dialogue/`)
are composited from the [Liberated Pixel Cup (LPC)](https://opengameart.org/content/liberated-pixel-cup-lpc-base-assets-sprites-map-tiles)
base assets and its many community add-ons on OpenGameArt.org, credited to:

Benjamin K. Smith (BenCreating), bluecarrot16, Carlo Enrico Victoria (Nemisys),
dalonedrau, DarkwallLKE, Durrani, Eliza Wyatt (ElizaWy), Evert, JaidynReiman,
Johannes Sjölund (wulax), Lanea Zimmerman (Sharm), Matthew Krohn (makrohn),
Michael Whitlock (bigbeargames), MuffinElZangano, Napsio (Vitruvian Studio),
Nila122, Pierre Vigier (pvigier), Sander Frenken (castelonia),
Stephen Challener (Redshrike), TheraHedwig, and Tuomo Untinen (reemax).

Released under a mix of **OGA-BY 3.0**, **CC-BY-SA 3.0**, and **GPL 3.0** (per
piece — see the `.txt` note beside each portrait for its exact sources and
licenses). Unlike the MiniWorld sprites above, this art is **not** CC0: reusing
it elsewhere requires carrying forward this same attribution.

## Music

The looping background themes are from OpenGameArt.org. Thank you to their
composers.

| Track | Composer | License |
| ----- | -------- | ------- |
| **Title** theme (`music/title.ogg`) | [bart](https://opengameart.org/users/bart) on OpenGameArt.org | CC BY 3.0 |
| **Boss** theme (`music/boss.ogg`) | [ATMANAN](https://opengameart.org/users/atmanan) on OpenGameArt.org | CC BY 4.0 |
