---
comments: true
---

# Extending the Game

Almost everything in Hero of the Overworld is **data**, not code. Heroes,
enemies, skills, encounters, whole levels, and cutscenes each live in their own
plain-text [RON](https://github.com/ron-rs/ron) file, organised by kind under
`assets/data/`:

```
assets/data/
  meta.ron                     # starting party + the level progression order
  characters/<id>.ron          # one party member per file
  enemies/<id>.ron             # one enemy per file
  skills/<id>.ron              # one skill per file
  statuses/<id>.ron            # burn, might, guard, …
  equipment/<id>.ron           # weapons & armor
  items/<id>.ron               # consumables
  shops/<id>.ron               # shop stock
  npcs/<id>.ron                # townsfolk appearances (talkable NPCs)
  encounters/<id>.ron          # named enemy groups
  cutscenes/<id>.ron           # scripted dialogue / recruits
  levels/<id>.ron              # a stage: metadata + its screens (minus tilemaps)
  maps/<level>/<screen>.csv    # each screen's tile grid, as a CSV
```

**Adding content is dropping in a file.** To add an enemy, create
`enemies/goblin_king.ron` holding one `EnemyDef(…)`; to add a stage, create
`levels/my_stage.ron` plus a `maps/my_stage/0.csv` per screen. The whole tree is
embedded into the binary at compile time (via `include_dir`), so a new file is
picked up on the next `cargo build` — no registration list to maintain, and the
web build still ships every byte. The one thing that needs a (tiny) code change is
genuinely **new art** (see below).

!!! note "A few things are ordered, and live in `meta.ron`"
    Content is looked up by **id**, so the *order* of files within a folder doesn't
    matter — except **levels**, whose order is the progression (each unlocks the
    next, and screen links are by index). That order, plus the starting party, is
    stated explicitly in `meta.ron`. Add a new level id there to slot it into the
    world.

Each file holds exactly one def, written just as it was in the old single database
— e.g. `enemies/slime.ron` is one `EnemyDef(…)`. The snippets below show a single
def; drop each into the matching folder.

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
it via a cutscene `Recruit` step (below). A character can also start with a `weapon`
and/or `armor` — see [Add equipment](#add-equipment).

The roster can hold as many heroes as you recruit, but a **battle seats only the
first `ACTIVE_PARTY` (3)** of them — the rest wait in reserve and are swapped in from
the [party menu](gameplay.md#the-battle-line-up) (MOVE UP / MOVE DOWN reorder the
line-up). Reserves still earn XP, so benching a hero never leaves them behind. Adding
a fourth-and-beyond hero needs **no engine changes** — the line-up cap is `ACTIVE_PARTY`
in `src/party.rs` if you ever want to widen it. **Slot 0 is the party leader** (the
first `starting_party` member, Roland): he walks the overworld, so he's pinned to the
front of the line-up and can't be reordered — only slots `1..` shuffle.

!!! tip "A boss who joins you"
    A recruit doesn't have to be met in a quiet cutscene — you can **fight** them
    first, like the Castaway Shore's [captain](entities/captain.md). Define **two**
    things that share a sprite: an `EnemyDef` (the boss you fight, in a `boss`
    [encounter](#add-an-enemy-and-an-encounter)) and a `CharacterDef` (the ally). Place
    the boss in the level, and put a `Recruit(character: …)` step in the level's
    **`clear_cutscene`** — so felling the boss (and clearing the level) is what makes
    them join. The enemy and character are separate ids (`pirate_captain` the foe,
    `captain` the hero), so their battle stats and their party stats can differ.

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
      target, landing as it arrives (one bolt per target, so an `AllEnemies` skill
      fans a volley). `texture` is a key in `embedded_texture` (the bundled
      `fireball` art lives in `assets/textures/entities/animation_helpers/`). A
      multi-frame **spin sheet** (see below) tumbles as it flies — that's the thrown
      `axe`; a single-frame sprite just flies straight.
    - `Boomerang(texture: "axe")` — hurls a **single** spinning projectile out
      across the whole line and loops it back to the caster in a curving arc, the
      blow landing on every target as it sweeps out through them (BRENN's **AXE
      BOOMERANG**). Unlike `Projectile` it spawns one sprite no matter the target
      count; pairs naturally with an `AllEnemies` target.
    - `Charge` — the attacker dashes through the target(s), wraps around the
      screen, and returns.
    - `Crowd(texture: "pirate_grunt")` — the caster holds its post and summons a
      **swarm of allies** that floods the whole screen for a beat and then clears out,
      the blow landing at the crowded peak (the Captain's **ALL HANDS**). `texture` is
      a **16px-frame walk sheet** — a roaming/character sprite like `pirate_grunt` —
      and the crowd is drawn from its front-facing walk row. Pairs naturally with an
      `AllEnemies` target.

    A `Projectile` / `Boomerang` sprite may be a **spin sheet**: a grid of equal
    frames the projectile cycles through as it flies, read left-to-right then
    top-to-bottom (the `axe` is a 4×2 sheet — an 8-frame tumble). A texture's grid
    layout is registered in code beside its bytes — in `projectile_grid` in
    [`src/data.rs`](https://github.com/playforge-coding/hero-of-the-overworld/blob/main/src/data.rs)
    as `(columns, rows)` (default `(1, 1)` = a still sprite like the `fireball` or
    `bullet`). Add one line there for a new spinning projectile.

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
data edits: a `ShopDef` in `shops/<id>.ron`, and a **placement** on a screen (in
that level's `levels/<id>.ron`) so a keeper stands there. Buying deducts the price and equips the item — see
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
    // tilemap lives in maps/<level>/<screen>.csv, not here
    spawns: [ /* ... */ ],
    shops: [
        ShopSpawn(col: 5, row: 6, shop: "greenwood_outfitter"),
    ],
),
```

- `col`/`row` is the standable tile the keeper stands on (walk up + **Confirm**
  to enter). Keep it on open ground, like a spawn.
- The keeper's on-map sprite is the shared `shopkeeper` texture.

## Add a chest or a mimic

Both are **screen placements** — no new list of their own — and both are pure data.

A **chest** is opened with **Confirm** (like a shop) and pours its contents into the
party. Its reward fields are additive and each optional, so one chest can hold gold,
a consumable, and equipment at once:

```ron
ScreenDef(
    // tilemap lives in maps/<level>/<screen>.csv, not here
    chests: [
        ChestSpawn(col: 16, row: 3, gold: 30, item: Some("potion")),
        ChestSpawn(col: 3, row: 5, gold: 50, equipment: Some("scouts_edge")),
    ],
),
```

- `gold` (default `0`) is added to the purse; `item` is an [item](#add-an-item) id
  into the shared stash; `equipment` is an [equipment](#add-equipment) id dropped
  into the bag. Give a chest **at least one** of the three.
- Opened chests are **[saved](gameplay.md#saving)** per level, like beaten enemies,
  so a looted box stays looted across a save/reload — no re-farming.
- Chests draw the shared `chest` texture, bottom-centered like a prop.

A **mimic** looks exactly like a chest on the map (it shares the chest's footprint)
but is really a monster in disguise. It sits dormant until the player strays within
range, then wakes, gives chase, and starts an encounter on contact — reusing the
whole [roaming-enemy](world.md#roaming-enemies) chase/battle pipeline:

```ron
ScreenDef(
    // tilemap lives in maps/<level>/<screen>.csv, not here
    mimics: [
        MimicSpawn(col: 16, row: 8, encounter: "mimic_solo"),
    ],
),
```

- `encounter` is any [encounter](#add-an-enemy-and-an-encounter) id — typically a
  solo mimic, but nothing stops you fielding a bigger ambush (`mimic_pair`, …).
- On the map a *dormant* mimic borrows the real `chest` prop so it's pixel-identical
  to nearby treasure, switching to the `mimic` sheet's toothy row 1 only once it
  wakes. The enemy behind the encounter uses that same sheet in battle — see the
  [mimic](entities/mimic.md) bestiary page for the layout.
- The copy-the-party's-moves ability lives on the **enemy**, not the placement —
  see [Make it a mimic](#make-it-a-mimic) below.
- Slain mimics are saved per level, exactly like chests and enemies.

## Add townsfolk and houses

A level with **no `spawns` anywhere** is treated as a peaceful **town** — the
overworld drops the "cleared" banner and the foe counter, so you can build a hub
(like [Harborwatch](world.md#townsfolk-and-towns)) with nothing to fight, only
people to meet. Two pure-data pieces furnish one: **townsfolk** you talk to, and
**houses** you walk past.

A **townsfolk** is two things, mirroring shops: an `NpcDef` **appearance** in
`npcs/<id>.ron` (a reusable look), and a **placement** on a screen. The appearance
is just a name and an [`OverworldWalk`](#sprite-sheets) sheet (register the texture
key like any other art); one look can dress a whole town:

```ron
// npcs/farmer.ron — a reusable villager look (5×12 walk sheet, rows 0-3)
NpcDef(
    id: "farmer", name: "TOWNSFOLK",
    sprite: OverworldWalk(
        texture: "farmer",              // register the key in embedded_texture
        frame_w: 16, frame_h: 16,
        draw_w: 20.0, draw_h: 20.0,
        row_down: 0, row_up: 1, row_right: 2, row_left: 3,
        frames: 4, fps: 8.0,
    ),
),
```

Then place them on a screen with an `npcs` entry (alongside `spawns`, `shops`, …).
Everything but the look is per-placement, so the same sprite populates a town with
distinct lines:

```ron
ScreenDef(
    // tilemap lives in maps/<level>/<screen>.csv, not here
    npcs: [
        // Ambient villager: repeatable dialogue, an emote bubble over the head.
        NpcSpawn(
            col: 8, row: 7, npc: "farmer",
            facing: Down,                // Down (default) / Up / Left / Right
            emote: "question",           // talk / exclaim / question / love → *_emote texture
            lines: ["STRANGERS OFF A PIRATE DECK? STRANGE DAYS INDEED."],
        ),
        // A recruit: a one-time scripted talk that adds them to the party.
        NpcSpawn(
            col: 11, row: 7, npc: "axeman",
            facing: Down, emote: "exclaim",
            cutscene: Some("axeman_joins"),   // played once (may Recruit — below)
            recruits: Some("axeman"),         // hidden once this character is in the party
        ),
    ],
),
```

- Walk up and press **Confirm** on the **PRESS Z** prompt to talk. `col`/`row` must
  be **standable**, like a spawn.
- `emote` is a short key — `talk`, `exclaim`, `question`, or `love` — resolved to the
  `<emote>_emote` texture and bobbed over the NPC's head as a talk-to-me marker.
- `lines` (optional) are shown one box at a time and are **repeatable** — talk again
  and they replay. `portrait` (optional) draws a character/enemy sprite beside them.
- `cutscene` (optional) is a **one-time** [scripted talk](#add-a-cutscene) played the
  first time you speak to them (tracked like any played cutscene); afterwards they
  fall back to their `lines`. Use it for a scene that should happen once.
- `recruits` (optional) names the character an NPC **joins as**. Pair it with a
  `cutscene` whose `Recruit` step does the joining: talking starts the scene, and once
  that character is in the party the NPC is **no longer spawned** (they've moved into
  the ranks). This is how Harborwatch's **BRENN** the axeman is enlisted.

A **house** is a decorative building stamped from a shared **6×4 tileset** (its 24
tiles are `assets/textures/tiles/house/0.png`..`23.png`, laid out row-major). Place
one by its **top-left tile**; its grassy upper rows blend into the map and only the
**stone base row** is made solid, so the player walks in front of the door (houses
aren't entered):

```ron
ScreenDef(
    houses: [
        HouseSpawn(col: 2,  row: 1),   // occupies a 6-wide × 4-tall block
        HouseSpawn(col: 12, row: 1),
    ],
),
```

- Keep the whole 6×4 footprint **on the map** (the tests check this) and clear of
  spawns/NPCs/chests you want reachable, since its base row becomes wall.
- The tileset ships ready; no new art or `embedded_texture` edit is needed to place
  houses (the pieces are served by index as `house_0`..`house_23`).

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

### A scripted, unwinnable boss

Some fights are meant to be **lost** — the **[Demon King](entities/demon_king.md)**
who ends Chapter 1 is invincible and wipes the party on purpose, turning the defeat
into a story beat. Two pure-data hooks build the whole thing, **no engine change**:

An **`invincible`** enemy can never be brought below 1 HP, so it simply cannot be
killed:

```ron
EnemyDef(
    id: "demon_king", name: "DEMON KING",
    stats: Stats(max_hp: 9999, max_mp: 0, attack: 45, defense: 30, magic: 60, speed: 40),
    sprite: BattlerSprite( /* ... */ ),
    invincible: true,                    // damage can never drop it below 1 HP
    skills: ["oblivion", "sovereign_smite"],   // a free, unblockable party-wide sweep
    ai: Random,
    xp: 0, gold: 0,                      // never defeated, so never paid out
),
```

Give it a lethal, party-wide skill (high `power`, `target: AllEnemies`,
`unblockable: true`) and a `speed` above every hero's, and it will one-shot the team
before they can act. Then the **encounter** scripts what the loss *means*:

```ron
EncounterDef(
    id: "demon_king",
    enemies: ["demon_king"],
    boss: true,
    defeat_cutscene: Some("demon_king_rise"),   // plays on the party wipe...
    defeat_advances_chapter: true,              // ...then ticks over to the next chapter
),
```

- **`defeat_cutscene`** replaces the usual revive-at-camp: when the party is wiped in
  this encounter, that [cutscene](#add-a-cutscene) plays instead.
- **`defeat_advances_chapter`** flings the party back to the surface and bumps the
  game's [chapter](#add-a-level), so every region they'd unlocked falls out of reach
  and the world map moves on. Pair the two to script a chapter transition; use
  `defeat_cutscene` alone for a scripted loss that stays in the same chapter.

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

### Make it a mimic

Give an enemy a `mimicry` field and it gains the **[mimic](entities/mimic.md#mimicry)**
trick: on some turns it **apes the party's last move**, taking on that hero's very
sprite as it casts a nerfed copy of their skill. It's layered *on top of* the enemy's
ordinary AI — the mimic still uses its own `skills` and basic attack — and only fires
once the party has actually cast a copyable move this battle.

```ron
EnemyDef(
    id: "mimic", name: "MIMIC",
    stats: Stats(max_hp: 132, max_mp: 40, attack: 26, defense: 14, magic: 24, speed: 14),
    sprite: BattlerSprite( /* ... */ ),
    skills: ["fireball"],       // its own moves (plus the basic-attack fallback)
    ai: Random,                 // so it mixes its own cast in with the mimicry
    xp: 55, gold: 80,
    mimicry: Some(MimicryDef(
        copyable: [             // the safe allow-list — only these can be aped
            "power_strike", "whirlwind",
            "firebolt", "frost",
            "quick_slash", "swallow_cut",
        ],
        power_pct: 65,          // the copy lands at 65% of the move's real power
        chance: 0.5,            // 0.0–1.0: how often it apes instead of acting normally
    )),
),
```

- **`copyable`** is an allow-list of skill ids: the mimic can only parrot these, so
  leave out the party's finishers to keep it from nuking you with your own hits. A
  copied move must be on this list *and* have been cast by a hero this fight.
- **`power_pct`** scales the copied move's power (the nerf); **`chance`** is the
  per-turn odds it apes rather than taking its ordinary turn. The copy is **free**
  (costs the mimic no MP) and otherwise behaves exactly like the original — a copied
  projectile still flies, a blockable copy can still be blocked.
- The whole ability is **data** — a "greater mimic" that copies the finishers at full
  strength, or an "arcane mimic" that apes only spells, is just another `MimicryDef`,
  **no engine change**. The mimic wears the copied hero's sprite only for the strike,
  then reverts.

## Add a level

A level is **two things**: a `levels/<id>.ron` describing the stage and its
connected **screens**, and one **CSV tilemap** per screen under
`maps/<id>/<screen index>.csv`. Finally, add the level's id to the ordered
`levels:` list in `meta.ron` so it slots into the progression.

The `levels/<id>.ron` holds a `LevelDef` — a marker on the world map plus the
screens, each with its enemy **spawns** and links to its neighbours. The screens'
**tilemaps are not here** (they live in the CSVs), so a screen lists only its
spawns, shops, chests, and links:

```ron
// levels/greenwood.ron
LevelDef(
    id: "greenwood", name: "GREENWOOD",
    node: (1, 2),                    // marker position on the world map
    start_screen: 0, start: (2, 5),  // which screen, and the tile you spawn on
    intro_cutscene: Some("greenwood_intro"),
    clear_cutscene: Some("mage_joins"),
    screens: [
        ScreenDef(                   // screen 0 → its map is maps/greenwood/0.csv
            east: Some(1),           // walking through the east opening flips to screen 1
            spawns: [
                SpawnDef(col: 11, row: 2, encounter: "demon_solo"),
            ],
        ),
        // screen 1 → maps/greenwood/1.csv …
    ],
),
```

Each screen's tilemap is a **CSV** where every line is a row and every
comma-separated cell is one tile character (an empty cell reads as grass). Screen
`i` in the `screens` list uses `maps/<id>/<i>.csv`. The tile legend:

| Char | Tile |
| ---- | ---- |
| `.` (or empty cell) | grass (walkable) |
| `T` | tree (solid) |
| `R` | rock (solid) |
| `~` | water (solid) |
| `#` | barricade (solid) |
| `G` | grass patch (walkable greenery over the base ground) |

```
# maps/greenwood/0.csv  — a gap in the right wall (row 2) is an east exit
T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T
T,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,T
T,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.
T,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,.,T
T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T,T
```

- `node` places the marker; the map cursor moves between markers by direction.
- `ground`, `wall`, and `tree` (all optional) **re-theme the region** by swapping the
  texture drawn for the base ground, `#` walls, and `T` trees — with no change to the
  tile legend. They default to `grass` / `barricade` / `tree`; the Castaway Shore, for
  instance, sets `ground: Some("sand")`, `wall: Some("barricade")` (its pirate
  palisades), and `tree: Some("coconut_tree")` (palms).
- `north`/`south`/`east`/`west` are **indices into this level's `screens`**. Leave
  an opening anywhere in the matching wall (a walkable cell on that edge) so the
  player can walk through it; on arrival they step out of the opening on the far
  screen's edge nearest to where they left, so the gaps needn't line up.
- A roaming enemy takes its on-map look from its encounter's **first** enemy.
- A screen can also carry `shops`, [`chests`, and `mimics`](#add-a-chest-or-a-mimic),
  and [`npcs` and `houses`](#add-townsfolk-and-houses), alongside its `spawns`. A
  level with **no spawns at all** becomes a peaceful [town](#add-townsfolk-and-houses).
- Remember to add the level id to `meta.ron`'s `levels:` list, in the position you
  want it in the progression.
- `chapter` (optional, defaults to `1`) groups the level into a story **chapter**.
  The world map only offers the *current* chapter's regions; within a chapter,
  progression is linear as usual. The party advances a chapter by facing a
  chapter-advancing boss (see [`defeat_advances_chapter`](#a-scripted-unwinnable-boss)),
  at which point earlier chapters' levels fall out of reach. All bundled levels are
  chapter 1; give a new region `chapter: 2` to start building the next arc.

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
- Reference the cutscene from a level's `intro_cutscene` or `clear_cutscene`, or
  from a [townsfolk](#add-townsfolk-and-houses)'s `cutscene` (a recruit-on-talk).
  Each fires only once per run.

### Choreograph the map

A cutscene plays out **on the live level** — the map stays on screen behind the
dialogue box. Extra **cast actors** can be walked around it, timed against the
lines, so a scene is a little scripted play rather than a wall of text. In this
GREENWOOD intro a SLIME oozes across the trail while ROLAND warns of the swarm:

```ron
CutsceneDef(
    id: "greenwood_intro",
    steps: [
        Say(speaker: Some("ROLAND"), portrait: Some("swordsman"),
            text: "THE GREENWOOD SWARMS WITH SLIMES - AND A DEMON LURKS DEEP WITHIN."),
        Place(actor: "slime", character: "slime", at: (17, 2), facing: Left),
        Walk(actor: "slime", to: (11, 5)),          // creeps onto the trail...
        Turn(actor: "slime", facing: Down),
        Wait(secs: 0.4),                            // ...a beat to be noticed
        Say(speaker: Some("ROLAND"), portrait: Some("swordsman"),
            text: "THERE'S ONE NOW. STAY SHARP - WHERE ONE SLIME OOZES, A DOZEN FOLLOW."),
        Walk(actor: "slime", to: (11, 9), speed: Some(40.0)),   // slips away south
        Leave(actor: "slime"),
    ],
),
```

The choreography steps fall into two groups:

| Step | Effect | Timing |
|---|---|---|
| `Place(actor, character, at: (col, row), facing?)` | Bring a cast actor on, drawn from any character/enemy's overworld sprite, at a tile. Re-placing the same `actor` snaps it. | instant |
| `Turn(actor, facing)` | Face a placed actor `Down`/`Up`/`Left`/`Right`. | instant |
| `Leave(actor)` | Remove a cast actor from the screen. | instant |
| `Pan(at: (col, row))` | Ease the camera to centre a tile (a no-op on maps smaller than the screen, which stay centred). | instant — drifts under the following steps |
| `Walk(actor, to: (col, row), speed?)` | Walk a placed actor to a tile, animating its walk cycle and turning to face the way it moves. | **waits** for arrival |
| `Wait(secs)` | Hold the scene for a beat with no dialogue. | **waits** `secs` |

- `actor` is a handle you pick, used to address the same cast member across steps;
  `character` is the content id its sprite comes from. The roaming player and the
  screen's own townsfolk are left untouched — cast actors are extra players
  brought on just for the scene.
- **Instant** steps fire the moment they're reached and the scene runs straight on.
  **Waiting** steps (`Say`, `Walk`, `Wait`) hold until the line is dismissed, the
  actor arrives, or the beat ends — a <kbd>Z</kbd>/<kbd>X</kbd> press skips ahead in
  every case. Interleaving the two is the whole trick: place and walk actors between
  (and during) the lines that narrate them.
- Cast walks are scripted straight lines that ignore tile collision, so route them
  over open ground (the `(col, row)` tiles are the same grid the level's
  [maps](#add-a-level) use).

## Check your work

The fast test suite parses the data file and cross-checks every reference — that
each skill (including a character's `learnset` and any `Projectile` animation
texture), enemy, encounter, texture, cutscene, shop, item, and drop id resolves,
that shop wares and keeper placements are valid, that chest loot and mimic
encounters resolve and sit on standable ground, that **townsfolk** name a real
appearance / emote / cutscene (and that a recruiting NPC's scene actually adds
them), that every cutscene `Recruit`/`portrait`/`Place` names a real
character/enemy, that every **house** fits on its screen, that item effects and
enemy drops
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
- **Fight encounter** — pick any encounter (any `encounters/<id>.ron`) to jump straight
  into that battle, boss theme and all. It grants full spoils on victory and drops
  you back on the map afterward, so you can test a fight in isolation.

These are gated behind `#[cfg(debug_assertions)]`, so they are **compiled out of
`--release` builds entirely** — a shipped game keeps the normal linear progression
and players cannot skip, reset, auto-win, or open the dev menu. Both screens show a
small *"DEV: …"* hint in debug builds as a reminder.
