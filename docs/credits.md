---
comments: true
---

# Credits

Hero of the Overworld is built on a stack of excellent open-source Rust crates.
Thank you to their authors and maintainers.

| Crate | Role |
| ----- | ---- |
| [wgpu](https://crates.io/crates/wgpu) | GPU rendering (native WebGPU/Vulkan/Metal/DX and browser WebGL) |
| [winit](https://crates.io/crates/winit) | Windowing and input event loop |
| [glam](https://crates.io/crates/glam) | Vector / matrix math |
| [bytemuck](https://crates.io/crates/bytemuck) | Zero-copy vertex/instance buffers |
| [image](https://crates.io/crates/image) | Decoding the texture PNGs |
| [ron](https://crates.io/crates/ron) + [serde](https://crates.io/crates/serde) | Parsing the `game.ron` content database |
| [rodio](https://crates.io/crates/rodio) | Background music playback (native) |
| [log](https://crates.io/crates/log) + [env_logger](https://crates.io/crates/env_logger) | Logging |
| [pollster](https://crates.io/crates/pollster) | Blocking on wgpu's async device setup (native) |
| [wasm-bindgen](https://crates.io/crates/wasm-bindgen) + [web-sys](https://crates.io/crates/web-sys) | Browser bindings for the web build (canvas, `<audio>`) |
| [Trunk](https://trunkrs.dev) | Bundling the WebAssembly web build |
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
