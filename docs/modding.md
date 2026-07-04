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
"swordsman"     => include_bytes!("../assets/textures/entities/playables/swordsman.png"),
"mage"          => include_bytes!("../assets/textures/entities/playables/mage.png"),
"demon"         => include_bytes!("../assets/textures/entities/monsters/demon.png"),
"starter_sword" => include_bytes!("../assets/textures/items/starter_sword.png"),
"starter_gear"  => include_bytes!("../assets/textures/items/starter_gear.png"),
"grass"         => include_bytes!("../assets/textures/tiles/grass.png"),
// ...
```

To use a brand-new PNG — like Elara's own `mage.png` — drop it under `assets/` and
add one line here. You can also **reskin an existing** sheet for a new character with
no new art at all: apply a `tint` in the RON to recolour one you already ship.

## Sprite sheets

Battler sheets are grids of 16×16 frames. An animation clip picks a **row** and a
run of **frames** columns starting at **first_col**, played at **fps**:

- `swordsman.png` — 5×12. Rows 0–3 walk down/up/right/left; rows 4–7 attack.
- `mage.png` — 6×8. Rows 0–3 walk (4 frames); rows 4–7 cast (5 frames), ordered
  down/up/**left/right** — the cast rows swap left and right versus the walk rows.
- `demon.png` — 6×8, same convention as the swordsman.
- `gargoyle.png` — 6×8, reuses the demon's layout with different art.
- `slime.png` — 6×4. Rows 0–3 walk; it has no attack rows, so its "attack" clip
  just replays a walk row a little faster.

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
        texture: "mage",                 // any key from embedded_texture
        frame_w: 16, frame_h: 16,
        draw_w: 48.0, draw_h: 48.0,
        // tint: Some((150, 100, 220)),  // optional: recolour a shared sheet
        idle:   AnimClip(row: 2, first_col: 0, frames: 4, fps: 5.0),
        attack: AnimClip(row: 7, first_col: 0, frames: 5, fps: 14.0),
    ),
    skills: ["firebolt", "frost", "mend"],
    // armor: Some("travelers_robe"),   // optional starting gear (see below)
),
```

Then either list its `id` in `starting_party` so it begins in the party, or add
it via a cutscene `Recruit` step (below). The battle scene iterates the whole
party, so a second or third hero fights with **no engine changes**. A character
can also start with a `weapon` and/or `armor` — see [Add equipment](#add-equipment).

## Add a skill

Every skill needs a one-line `description` (shown in the skill menu):

```ron
SkillDef(
    id: "firebolt", name: "FIREBOLT",
    description: "Hurl a searing bolt of flame at one foe.",
    mp_cost: 6, power: 180, kind: Magical, target: OneEnemy,
),
```

- `kind` is `Physical`, `Magical`, or `Heal`.
- `target` is `OneEnemy`, `AllEnemies`, `OneAlly`, `AllAllies`, or `SelfOnly`.
- `power` is a percentage multiplier on the relevant stat (see
  [How damage works](battles.md#how-damage-works)).

## Add equipment

Weapons and armor live in an `equipment` list. Each item has a **slot**, a 16×16
**icon** (register its texture key like any other art), stat **mods**, optional
**crit / accuracy / evasion** bonuses, and a **description**:

```ron
EquipmentDef(
    id: "iron_sword", name: "IRON SWORD",
    description: "Standard-issue soldier's blade.",
    slot: Weapon, icon: "starter_sword",     // register the icon in embedded_texture
    mods: StatMods(attack: 6), crit: 6, accuracy: 4,
),
EquipmentDef(
    id: "leather_armor", name: "LEATHER ARMOR",
    description: "Boiled leather; turns a glance.",
    slot: Armor, icon: "starter_gear",
    mods: StatMods(defense: 5), evasion: 4,
),
```

- `slot` is `Weapon` or `Armor` (a battler holds one of each).
- `mods` is a `StatMods(...)` of flat bonuses — any of `attack`, `defense`,
  `magic`, `speed` (all default to 0). Equipment deliberately **can't** change max
  HP/MP; that stays the domain of levelling up.
- `crit`, `accuracy`, and `evasion` are percent bonuses (see
  [Battles](battles.md#hit-miss-and-crit)).
- Keep the `description` short — it's drawn on one line in the in-battle gear
  panel.

Then equip it on a character or enemy by id:

```ron
CharacterDef(
    id: "swordsman", name: "ROLAND",
    // ...
    weapon: Some("iron_sword"),
    armor:  Some("leather_armor"),
),
```

Both fields are optional; a battler with neither just fights on its base stats.
For the shipped items and what they do, see **[Weapons & Armor](equipment.md)**.

## Add a shop

A **shop** is a store the player enters from the overworld to buy gear. It's two
data edits: a `ShopDef` in the `shops` list, and a **placement** on a screen so a
keeper stands there. Buying deducts the price and equips the item — see
**[Shops](shops.md)** for the player-facing flow.

```ron
ShopDef(
    id: "greenwood_outfitter", name: "OUTFITTER",
    greeting: Some("FRESH FROM THE FORGE - WHAT'LL IT BE?"),
    facing: Down,                         // the wall the keeper faces = the exit door
    stock: [
        ShopStock(item: "iron_sword",     price: 40),   // ids into `equipment`
        ShopStock(item: "leather_armor",  price: 40),
        ShopStock(item: "travelers_robe", price: 70),
    ],
),
```

- `facing` is `Down` (default), `Up`, `Left`, or `Right` — it picks which wall
  the interior's **doorway** sits on, so the player leaves the way the keeper
  looks.
- `greeting` is optional flavour shown in the buy menu.
- Each `ShopStock` names an `equipment` **id** and its **price** in gold. Stock is
  unlimited; the buyer chooses which party member to outfit.

Then place a keeper on a screen with a `shops` entry (alongside its `spawns`),
pointing at the shop id:

```ron
ScreenDef(
    map: [ /* ... */ ],
    spawns: [ /* ... */ ],
    shops: [
        ShopSpawn(col: 5, row: 6, shop: "greenwood_outfitter"),
    ],
),
```

- `col`/`row` is the standable tile the keeper stands on (walk up + **Confirm**
  to enter). Keep it on open ground, like a spawn.
- The keeper's on-map sprite is the shared `shopkeeper` texture.

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

Add `boss: true` to make it a **boss fight** — the battle then plays the boss
theme (`music/boss.ogg`) instead of the normal battle track. Everything else about
the encounter works the same:

```ron
EncounterDef(id: "dragon_boss", enemies: ["dragon"], boss: true),
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
  an opening anywhere in the matching wall so the player can walk through it; on
  arrival they step out of the opening on the far screen's edge nearest to where
  they left, so the gaps needn't line up at the mid-point.
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
each skill, enemy, encounter, texture, cutscene, and shop id resolves, that shop
wares and keeper placements are valid, and that every level's screens are linked
and traversable. Run it after editing:

```sh
cargo test --test data
```
