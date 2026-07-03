---
comments: true
---

# Extending the Game

Almost everything in Hero of the Overworld is **data**, not code. Heroes,
enemies, skills, encounters, whole levels, and cutscenes all live in a single
plain-text [RON](https://github.com/ron-rs/ron) file:

```
assets/data/game.ron
```

Editing that file is enough to add party members, enemies, skills, levels, and
story beats — the battle and overworld systems just iterate whatever they find.
The one thing that needs a (tiny) code change is genuinely **new art**.

## Textures are embedded by key

Art is referenced from the RON by a string **key**, and the actual PNG bytes are
baked into the binary at compile time so the native and web builds load through
the exact same path. The mapping lives in
[`src/data.rs`](https://github.com/playforge-coding/hero-of-the-overworld/blob/main/src/data.rs)
in `embedded_texture`:

```rust
"swordsman" => include_bytes!("../assets/textures/entities/playables/swordsman.png"),
"demon"     => include_bytes!("../assets/textures/entities/monsters/demon.png"),
"grass"     => include_bytes!("../assets/textures/tiles/grass.png"),
// ...
```

To use a brand-new PNG, drop it under `assets/` and add one line here. To reskin
an **existing** sheet for a new character, you don't even need that — apply a
`tint` in the RON instead (that's how the bundled mage reuses the swordsman
sheet).

## Sprite sheets

Battler sheets are grids of 16×16 frames. An animation clip picks a **row** and a
run of **frames** columns starting at **first_col**, played at **fps**:

- `swordsman.png` — 5×12. Rows 0–3 walk down/up/right/left; rows 4–7 attack.
- `demon.png` — 6×8, same convention.

A `BattlerSprite` needs an `idle` and `attack` clip (used in battle). An optional
`OverworldWalk` gives four directional walk rows for moving on the map; without
one, a character falls back to its battle idle.

## Add a party member

```ron
CharacterDef(
    id: "mage",
    name: "ELARA",
    stats: Stats(max_hp: 78, max_mp: 60, attack: 10, defense: 9, magic: 26, speed: 13),
    sprite: BattlerSprite(
        texture: "swordsman",            // any key from embedded_texture
        frame_w: 16, frame_h: 16,
        draw_w: 48.0, draw_h: 48.0,
        tint: Some((120, 150, 255)),     // recolour a shared sheet
        idle:   AnimClip(row: 2, first_col: 0, frames: 4, fps: 5.0),
        attack: AnimClip(row: 6, first_col: 0, frames: 4, fps: 14.0),
    ),
    skills: ["firebolt", "frost", "mend"],
),
```

Then either list its `id` in `starting_party` so it begins in the party, or add
it via a cutscene `Recruit` step (below). The battle scene iterates the whole
party, so a second or third hero fights with **no engine changes**.

## Add a skill

```ron
SkillDef(
    id: "firebolt", name: "FIREBOLT",
    mp_cost: 6, power: 180, kind: Magical, target: OneEnemy,
),
```

- `kind` is `Physical`, `Magical`, or `Heal`.
- `target` is `OneEnemy`, `AllEnemies`, `OneAlly`, `AllAllies`, or `SelfOnly`.
- `power` is a percentage multiplier on the relevant stat (see
  [How damage works](battles.md#how-damage-works)).

## Add an enemy and an encounter

```ron
EnemyDef(
    id: "demon", name: "DEMON",
    stats: Stats(max_hp: 58, max_mp: 12, attack: 16, defense: 8, magic: 9, speed: 9),
    sprite: BattlerSprite( /* ... */ ),
    skills: ["claw"],
    ai: Random,            // Basic (always attacks) or Random (mixes in skills)
    xp: 12, gold: 8,       // rewards on defeat
),
```

An **encounter** is just a named group of enemy ids (repeats allowed):

```ron
EncounterDef(id: "demon_trio", enemies: ["demon", "demon", "demon"]),
```

## Add a level

A `LevelDef` is a marker on the world map plus a set of connected **screens**.
Each screen is an ASCII tile map with enemy **spawns** and links to its
neighbours. The tile legend:

| Char | Tile |
| ---- | ---- |
| `.` / space | grass (walkable) |
| `T` | tree (solid) |
| `R` | rock (solid) |
| `~` | water (solid) |
| `#` | barricade (solid) |

```ron
LevelDef(
    id: "greenwood", name: "GREENWOOD",
    node: (1, 2),                    // marker position on the world map
    start_screen: 0, start: (2, 5),  // which screen, and the tile you spawn on
    intro_cutscene: Some("greenwood_intro"),
    clear_cutscene: Some("mage_joins"),
    screens: [
        ScreenDef(
            map: [
                "TTTTTTTTTTTTTTTTTTTT",
                "T..................T",
                "T..................",   // gap in the right wall = an east exit
                "T..................T",
                "TTTTTTTTTTTTTTTTTTTT",
            ],
            east: Some(1),           // walking through that gap flips to screen 1
            spawns: [
                SpawnDef(col: 11, row: 2, encounter: "demon_solo"),
            ],
        ),
        // screen 1 ...
    ],
),
```

- `node` places the marker; the map cursor moves between markers by direction.
- `north`/`south`/`east`/`west` are **indices into this level's `screens`**. Leave
  an opening in the middle of the matching wall so the player can walk through it.
- A roaming enemy takes its on-map look from its encounter's **first** enemy.

## Add a cutscene

```ron
CutsceneDef(
    id: "mage_joins",
    steps: [
        Say(speaker: Some("ELARA"), portrait: Some("mage"),
            text: "I AM ELARA, A WANDERING MAGE. YOUR CAUSE IS MINE NOW."),
        Recruit(character: "mage"),
        Say(speaker: Some("ROLAND"), portrait: Some("swordsman"),
            text: "GLAD TO HAVE YOUR FIRE AT MY BACK, ELARA."),
    ],
),
```

- `Say` shows a dialogue line; `portrait` is any character/enemy id whose sprite
  is drawn beside the text.
- `Recruit` adds a character to the party (a no-op if they're already in it).
- Reference the cutscene from a level's `intro_cutscene` or `clear_cutscene`.
  Each fires only once per run.

## Check your work

The fast test suite parses the data file and cross-checks every reference — that
each skill, enemy, encounter, texture, and cutscene id resolves, and that every
level's screens are linked and traversable. Run it after editing:

```sh
cargo test --test data
```
