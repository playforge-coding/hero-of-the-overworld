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
    skills: ["firebolt"],                // known from the start
    learnset: [                          // unlocked as they level up (optional)
        LearnedSkill(level: 4, skill: "frost"),
        LearnedSkill(level: 7, skill: "mend"),   // a capstone: her last, strongest move
    ],
    // armor: Some("travelers_robe"),   // optional starting gear (see below)
    // timing: Some(TimingWindow(perfect: 0.09, good: 0.20)),  // optional
),
```

Then either list its `id` in `starting_party` so it begins in the party, or add
it via a cutscene `Recruit` step (below). The battle scene iterates the whole
party, so a second or third hero fights with **no engine changes**. A character
can also start with a `weapon` and/or `armor` — see [Add equipment](#add-equipment).

The optional **`learnset`** unlocks skills as the hero **levels up**: each entry
teaches its `skill` (an id from the [skills](#add-a-skill) list) once the hero
reaches that `level`. Skills in `skills` are known from level 1; put anything
gated behind a level in `learnset` (its `level` must be **> 1**). Learning is
announced on the victory report, and a hero [recruited](#add-a-cutscene) mid-game
arrives at the party's level already knowing everything it grants — so a late
recruit isn't stuck with only their starting kit.

The optional **`timing`** widens (or tightens) that hero's
[timed-hit window](battles.md#action-timing-strikes-and-blocks) — the *same* window
applies to both their **attacks** and their **blocks**. The two values are
half-widths, in animation seconds, of the **PERFECT** and **GOOD** windows around
the moment the blow connects — bigger is more forgiving. Omit it and the hero uses
the game's default (`perfect: 0.05`, `good: 0.13`); Gareth's generous `0.09 / 0.20`
is why his bonuses are so much easier to land.

## Add a skill

Every skill needs a one-line `description` (shown in the skill menu):

```ron
SkillDef(
    id: "firebolt", name: "FIREBOLT",
    description: "Hurl a searing bolt of flame at one foe.",
    mp_cost: 6, power: 180, kind: Magical, target: OneEnemy,
    unblockable: true,
    anim: Projectile(texture: "fireball"),   // optional attack animation
),
```

- `kind` is `Physical`, `Magical`, or `Heal`.
- `target` is `OneEnemy`, `AllEnemies`, `OneAlly`, `AllAllies`, or `SelfOnly`.
- `power` is a percentage multiplier on the relevant stat (see
  [How damage works](battles.md#how-damage-works)).
- `inflicts` (optional) is a list of status ids applied on a hit, e.g.
  `inflicts: ["burn"]`.
- `unblockable: true` (optional) makes the attack skip a hero's timed block, for
  piercing or magical blows — see
  [Action timing](battles.md#unblockable-attacks). Omitted, the attack can be
  blocked.
- `revives: true` (optional, `Heal` skills only) lets the heal target a **downed**
  ally and bring them back with the healed HP — not just top up the living. MEND
  sets this; ordinary heals leave it off.
- `anim` (optional) picks the skill's [attack animation](battles.md#attack-animations)
  — a purely cosmetic motion. It's one of:
    - `Lunge` — the default step-in-and-strike (used if omitted).
    - `Projectile(texture: "fireball")` — fires a sprite from the caster to each
      target, landing as it arrives. `texture` is a key in `embedded_texture` (the
      bundled `fireball` art lives in `assets/textures/entities/animation_helpers/`).
    - `Charge` — the attacker dashes through the target(s), wraps around the
      screen, and returns.

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

## Add an item

**Consumables** live in an `items` list. Unlike equipment they're **spent, not
worn**: used in battle from the **ITEM** command (restorative ones can also be
used from a rest), and acquired by [buying](#add-a-shop) or as
[enemy drops](#add-an-enemy-and-an-encounter). Each has a **target** and a
composable **effect** — any mix of `heal`, `damage`, `restore_mp`, and `inflicts`
(status ids). That last one is how an item **changes stats** or **defends**: point
it at a status whose `stat_mods` grants the buff.

```ron
ItemDef(
    id: "potion", name: "POTION",
    description: "A herbal draught. Restores 40 HP.",
    price: 30, target: OneAlly,
    effect: ItemEffect(heal: 40),
),
ItemDef(
    id: "bomb", name: "BOMB",
    description: "A lit satchel charge. 35 fire damage to a foe.",
    price: 40, target: OneEnemy,
    effect: ItemEffect(damage: 35, inflicts: ["burn"]),   // ids into `statuses`
),
ItemDef(
    id: "might_tonic", name: "MIGHT TONIC",
    description: "Steels the blood. Raises ATTACK for a few rounds.",
    price: 50, target: OneAlly,
    effect: ItemEffect(inflicts: ["might"]),              // a stat_mods status = a buff
),
```

- `target` is a `TargetKind` (`OneEnemy`, `AllEnemies`, `OneAlly`, `AllAllies`,
  `SelfOnly`) — offensive items target foes, restorative/buff items target allies.
- `effect` fields all default to 0 / empty, so list only what the item does. Item
  damage and healing are **flat** (no stat scaling) and reliable — consumables
  don't miss.
- `icon` is optional and defaults to the shared `item_bag` pouch; override it (and
  register the key) to give an item bespoke art.
- Only `heal` / `restore_mp` items can be used **outside** battle; damage and
  buffs need a fight to matter.

Sell it at a shop or drop it from an enemy (below), and see
**[Items](items.md)** for the shipped set.

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
        ShopStock(item: "iron_sword",     price: 40),   // an `equipment` id …
        ShopStock(item: "leather_armor",  price: 40),
        ShopStock(item: "travelers_robe", price: 70),
        ShopStock(item: "potion",         price: 30),   // … or an `items` id
    ],
),
```

- `facing` is `Down` (default), `Up`, `Left`, or `Right` — it picks which wall
  the interior's **doorway** sits on, so the player leaves the way the keeper
  looks.
- `greeting` is optional flavour shown in the buy menu.
- Each `ShopStock` names a ware **id** and its **price** in gold. The id is
  resolved as **equipment first, then an [item](#add-an-item)** — so the same
  counter can sell either. Stock is unlimited; buying gear lets the buyer pick who
  to outfit, while a consumable just goes into the shared stash.

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
    drops: [               // optional item drops, each rolled on victory
        ItemDrop(item: "potion", chance: 0.25),   // id into `items`, chance 0.0–1.0
    ],
),
```

`drops` is optional (most foes drop nothing). Each `ItemDrop` names an
[item](#add-an-item) id and an independent **chance** in `0.0..=1.0`; every
defeated enemy rolls its own table, and a hit is announced on the victory report.

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

### Make it a tool enemy

Give an enemy a `tool` field and it becomes an inert **[tool
enemy](battles.md#tool-enemies)** — a siege engine (like the
[ballista](entities/ballista.md)) that never takes its own turn. Instead, the
**aware** foes beside it (any living, non-tool enemy) can spend a turn to **work
it**, firing its `skill` at the party *from the tool itself* — so the tool's own
`stats` drive the shot. The moment no aware enemy is left to work it, the tool
**crumbles** on its own; until then it can be attacked and destroyed directly, so
tools are usually built tanky.

```ron
EnemyDef(
    id: "ballista", name: "BALLISTA",
    stats: Stats(max_hp: 160, max_mp: 0, attack: 24, defense: 18, magic: 0, speed: 3),
    sprite: BattlerSprite( /* ... */ ),   // its own sheet; no `overworld` — tools don't roam
    xp: 30, gold: 22,
    tool: Some(ToolDef(
        skill: "ballista_bolt",   // an id from the skills list — what it fires
        operate_chance: 0.8,      // 0.0–1.0: how often an aware foe works it on its turn
    )),
),
```

The whole inert / operated / crumbles-when-abandoned behaviour comes for free — a
new engine (a cannon, a catapult, …) is just another `EnemyDef` with a `tool` and
its own fired skill, **no engine change**. Always place a tool in encounters
**alongside real foes** (and never as the encounter's first enemy, since it has no
overworld sprite) — a tool with no crew crumbles the instant the fight starts.

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
each skill (including a character's `learnset` and any `Projectile` animation
texture), enemy, encounter, texture, cutscene, shop, item, and drop id resolves,
that shop wares and keeper placements are valid, that item effects and enemy drops
point at real statuses/items at sane odds, and that every level's screens are
linked and traversable. It also exercises level-up skill unlocks. Run it after
editing:

```sh
cargo test --test data
```

## Developer skipping & resetting (debug builds only)

Testing content deep in the game — a late level, the underworld, a level-clear
cutscene — is tedious if you must clear everything before it first, and replaying a
level you have already beaten is worse (its enemies stay defeated, so you walk an
empty map). In a **debug build** (`cargo run`, i.e. *not* `--release`), a set of
hidden hotkeys let a developer skip ahead and reset levels at will:

- **On the world map** — highlight a level marker and press <kbd>Tab</kbd> to mark it
  **cleared**, which unlocks the next one. Tab your way forward to open any level (all
  the way to the underworld's Charred Depths), then enter it as usual.
- **On the world map** — highlight a level marker and press <kbd>R</kbd> to **reset**
  it to its untouched state, so you can **replay it fresh**: its cleared flag is
  dropped, every defeated enemy respawns, and its intro and clear cutscenes are
  forgotten so they play again on re-entry. Handy for retuning a level (e.g. after
  editing an encounter) and walking straight back through it. This is the exact
  inverse of the <kbd>Tab</kbd> skip.
- **In a battle** — press <kbd>Tab</kbd> to **win instantly**. The fight resolves as a
  real victory: the encounter's full XP, gold, and rolled item drops are awarded, the
  roaming enemy clears from the map, and if it was the last foe the level's clear
  cutscene still fires. So you can walk a level, tapping <kbd>Tab</kbd> through each
  encounter, to clear it (and trigger things like the Demon Fortress → underworld
  portal cutscene) without actually fighting.

### The developer menu (F1)

For the state that is otherwise slow to set up, the world map also opens a small
**dev menu**: highlight nothing in particular and press <kbd>F1</kbd>. It's a plain
selectable menu (arrow keys / <kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> to
move, <kbd>Enter</kbd> to choose, <kbd>Esc</kbd> to back out) with three tools:

- **Change level** — a spinner (<kbd>←</kbd>/<kbd>→</kbd> by 1,
  <kbd>↑</kbd>/<kbd>↓</kbd> by 10) that sets **every** party member to the chosen
  level. Members are rebuilt up the normal growth curve, so their stats and learned
  skills match that level exactly (and it works downward too); equipment is kept.
- **Add party member** — pick any character from the roster (`characters` in the RON)
  to recruit it on the spot, joining at the party's current level like a normal
  mid-game recruit. Handy for testing a hero's kit without playing to their story
  recruitment. (No dedupe — choosing the same one twice adds two copies.)
- **Fight encounter** — pick any encounter (`encounters` in the RON) to jump straight
  into that battle, boss theme and all. It grants full spoils on victory and drops
  you back on the map afterward, so you can test a fight in isolation.

These are gated behind `#[cfg(debug_assertions)]`, so they are **compiled out of
`--release` builds entirely** — a shipped game keeps the normal linear progression
and players cannot skip, reset, auto-win, or open the dev menu. Both screens show a
small *"DEV: …"* hint in debug builds as a reminder.
