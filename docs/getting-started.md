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
- A GPU with OpenGL (native) or WebGL (browser). Most machines from the last
  decade qualify. On Linux you also need the usual X11 / OpenGL / ALSA dev
  packages macroquad links against, e.g. `libx11-dev libxi-dev libgl1-mesa-dev
  libasound2-dev`.

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

macroquad compiles the game to a single `.wasm` that its JavaScript glue loads
into a canvas — no wasm-bindgen or bundler. Add the wasm target once:

```sh
rustup target add wasm32-unknown-unknown
```

Build the wasm binary (the `hero` bin becomes `hero.wasm`):

```sh
cargo build --release --target wasm32-unknown-unknown --bin hero
```

Assemble a folder to serve — the repo's [`index.html`](https://github.com/playforge-coding/hero-of-the-overworld/blob/master/index.html)
already loads `./hero.wasm` and `./mq_js_bundle.js`:

```sh
mkdir -p web && cp index.html web/
cp target/wasm32-unknown-unknown/release/hero.wasm web/
# macroquad's JS loader (fetch once):
curl -L https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js -o web/mq_js_bundle.js
```

Then serve `web/` with any static server (browsers won't run wasm off `file://`):

```sh
python3 -m http.server -d web 8080   # then open http://127.0.0.1:8080
```

The published site does exactly this in CI — see the [Docs workflow](https://github.com/playforge-coding/hero-of-the-overworld/blob/master/.github/workflows/docs.yml).

## Next steps

- **[Controls](controls.md)** — how to move, confirm, and command your party.
- **[Gameplay](gameplay.md)** — the moment-to-moment loop from title to victory.
