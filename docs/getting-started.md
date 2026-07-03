---
comments: true
---

# Getting Started

There are three ways to play Hero of the Overworld:

- **[Play in your browser](https://playforge-coding.github.io/hero-of-the-overworld/play/)** —
  nothing to install; the game runs on GitHub Pages.
- **Download a prebuilt binary** — grab a self-contained executable for Windows,
  macOS, or Linux from the [Releases](https://github.com/playforge-coding/hero-of-the-overworld/releases)
  page, then run it. Each archive holds a single binary (`hero`) with the
  textures and music baked in.
- **Build from source** — as a native binary or a WebAssembly bundle you serve
  yourself, using the steps below.

## Prerequisites

*(Only needed to build from source.)*

- A recent **Rust toolchain** (install via [rustup](https://rustup.rs)).
- A GPU and drivers that support one of wgpu's backends (Vulkan, Metal, DX12, or
  WebGL in the browser). Most machines from the last decade qualify.

Clone the repository first:

```sh
git clone https://github.com/playforge-coding/hero-of-the-overworld.git
cd hero-of-the-overworld
```

!!! note "Git LFS"
    The texture PNGs and the music track are stored with
    [Git LFS](https://git-lfs.com). If you don't have it installed, they come
    down as small pointer files and the game won't render or play sound
    correctly. Install `git-lfs`, then run `git lfs pull`.

## Native

```sh
cargo run            # debug
cargo run --release  # smooth
```

The first build downloads and compiles dependencies, so it takes a while;
subsequent runs are fast. A window opens on the [title screen](gameplay.md#the-title-screen).

## Web

The web build targets WebAssembly and uses [Trunk](https://trunkrs.dev) as the
bundler. Install the wasm target and Trunk once:

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Then run a dev server or produce a static bundle:

```sh
trunk serve            # dev server at http://127.0.0.1:8080
# or
trunk build --release  # static bundle written to dist/
```

wgpu is compiled with the `webgl` feature so the game runs in browsers without
native WebGPU support. The static `dist/` bundle is plain files — host it on any
static web server.

## Next steps

- **[Controls](controls.md)** — how to move, confirm, and command your party.
- **[Gameplay](gameplay.md)** — the moment-to-moment loop from title to victory.
