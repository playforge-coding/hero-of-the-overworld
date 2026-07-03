---
comments: true
---

# Hero of the Overworld

**Hero of the Overworld** is a small, **extensible** turn-based JRPG written in
Rust. You lead a party of heroes across a **world map**, explore tile-mapped
**levels** full of roaming demons, and fight **turn-based battles** with a
classic ATTACK / SKILL / DEFEND menu. Clear a level and a scripted **cutscene**
may usher a new ally into your party.

It renders everything as textured quads with **wgpu**, and runs both natively
and in the browser (via WebGL) using **[Trunk](https://trunkrs.dev)**.

<div class="grid cards" markdown>

- :material-rocket-launch: **[Getting Started](getting-started.md)**

- :material-controller: **[Controls](controls.md)**

- :material-sword-cross: **[Gameplay](gameplay.md)**

- :material-earth: **[The Overworld](world.md)**

- :material-shield-sword: **[Battles](battles.md)**

- :material-file-code: **[Extending the Game](modding.md)**

</div>

## At a glance

|                 |                                                                     |
| --------------- | ------------------------------------------------------------------- |
| **Genre**       | Single-player, turn-based JRPG                                       |
| **Renderer**    | wgpu (native WebGPU / Vulkan / Metal / DX; WebGL in the browser)     |
| **Canvas**      | Fixed 320×180 virtual resolution, letterboxed into the window        |
| **You control** | The party leader on the map; each hero's action in battle           |
| **Overworld**   | A world map of levels, each a set of connected tile screens          |
| **Enemies**     | Demons that roam levels and chase you into turn-based battles        |
| **Content**     | Heroes, enemies, skills, levels, and cutscenes live in one RON file  |
| **Platforms**   | Native (Windows/macOS/Linux) and web (WebAssembly)                   |

!!! tip "New here?"
    Head to **[Getting Started](getting-started.md)** to get the game running,
    then skim **[Controls](controls.md)** and **[Gameplay](gameplay.md)** before
    you march into the Greenwood.
