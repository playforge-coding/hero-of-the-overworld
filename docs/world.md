---
comments: true
---

# The Overworld

The world is hand-authored, not generated: everything you walk through is laid
out as tile maps in the game's data file. It's arranged in two tiers — a
**world map** of levels, and the **screens** inside each level.

## The world map

After the title you arrive at the world map: a set of **level markers** connected
by a faint dotted travel route. Each marker sits at a grid **node** and shows the
level's name beneath it.

- **Move** hops the cursor to the nearest marker in that direction.
- **Confirm** enters the highlighted level — if it's unlocked.
- A marker is **red** while the level still holds foes and turns **green with a
  star** once you've cleared it. A **CLEARED x/y** tally sits in the corner.
- A **grey marker with an X** is still **locked**.

### Progression is linear

You must **clear a level to unlock the next one**. The first level is always
open; each later one stays locked (and can't be entered) until the level before
it is fully cleared. Selecting a locked level shows *"CLEAR THE PREVIOUS LEVEL
FIRST"* instead of the usual prompt, so the world opens up as you win. And you
rarely need the map to move forward: clearing a level **carries you straight into
the next one** (see [Clearing a level](#roaming-enemies) below).

The bundled world has six levels across two realms — the surface, and the
**underworld** beneath it, reached once the Demon Fortress falls — ending at the
iron **Demon Facility** deep below, the seat of the horde's power and the climax of
**[Chapter 1](gameplay.md#chapters)**:

| Level | Screens | Enemies | Notes |
| ----- | ------- | ------- | ----- |
| **GREENWOOD** | 5 | [Slime](entities/slime.md) swarms + a lone [demon](entities/demon.md) guardian | The opening level: a straightforward forest walk with some winding parts, flowing east then bending north into the deep wood. Clearing it triggers the cutscene where ELARA joins. |
| **STONE PASS** | 5 | [Gargoyle](entities/gargoyle.md) patrols + a slime pack | A straight rocky descent (screens stacked north–south) that ends in a **boulder maze** you must snake through. Unlocks after Greenwood. |
| **TRAVELLER'S END** | 12 | [Crabs](entities/mountain_crab.md), [skeletons](entities/skeleton.md), [gargoyles](entities/gargoyle.md), mounted [dark knights](entities/dark_knight.md) + stray [demons](entities/demon.md) | The **longest trek** in the game: a switchbacking climb up a stony mountain range, grass and pines giving way to bare scree and boneyards. Foes escalate the higher you go — crabs and slimes at the foot, skeletons and gargoyles on the shelves, dark knights and demons holding the storm-lashed summit. Unlocks after Stone Pass. |
| **DEMON FORTRESS** | 6 | [Demon](entities/demon.md) packs (duos and trios) + a [dragon](entities/dragon.md) boss | A **very mazelike** warren of dark-brick corridors: a gatehouse into a crossing that branches to a dead-end cell block or deeper east through twisting galleries to the dragon's lair. Felling the dragon opens the advisor's **portal** and carries the party down into the underworld. Unlocks after Traveller's End. |
| **CHARRED DEPTHS** | 11 | [Club](entities/club_goblin.md) & [archer goblins](entities/archer_goblin.md), [orc brutes](entities/orc_brute.md) + [demons](entities/demon.md) | The **first level of the underworld** and the **toughest descent** in the game — a long warren of scorched flagstone that winds ever deeper, its foes escalating like the climb of Traveller's End. Goblin packs (clubbers screening their archers) hold the upper halls, orc brutes wall the middle chambers two and three abreast, and demon-led warbands hold the depths — down to the deep hall's guardian, a demon flanked by two remade orc thralls. Unlocks after the Demon Fortress — a **[portal cutscene](story.md)** takes you there, and it stands open on the map from then on. |
| **DEMON FACILITY** | 11 | [Demon elites](entities/demon_elite.md) (singly and in packs) + a [Demon King](entities/demon_king.md) boss | The **climax of Chapter 1**: past the Charred Depths the raw stone turns to **iron**, an underground forge-hall where the horde plates its demons for war. A long, winding descent in the mould of the Charred Depths — war-plated elites two and three abreast, a barracks cache guarded by a mimic, a throne-hall guard — ending at the throne of the invincible **[Demon King](entities/demon_king.md)**. That fight **cannot be won**: losing it hurls the party back to the surface and opens **[Chapter 2](gameplay.md#chapters)**. Unlocks after the Charred Depths. |

## Screens

A level is a handful of **screens** (rooms) you walk between, Zelda-style. Each
screen is a grid of 16×16 tiles. The party leader walks freely in pixels with
per-axis collision, sliding along walls rather than sticking, and the **camera
follows**, clamped so it never shows past the edge of a small screen.

Walk into an **opening in an edge** — anywhere the border tiles stop — and, if
that side links to another screen, you flip to it and step out of the opening on
the opposite edge **nearest to where you left**, so a maze can wind its doorways
wherever the layout wants rather than always at the mid-point. A brief grace
period after arriving stops a demon from instantly pouncing on you.

### Tiles

| Tile | Char | Walkable? |
| ---- | ---- | --------- |
| Grass | `.` (or space) | ✅ walkable base ground |
| Grass patch | `G` | ✅ walkable — draws grass over the base |
| Tree | `T` | ⛔ solid — units path around it |
| Rock | `R` | ⛔ solid |
| Water | `~` | ⛔ solid |
| Barricade | `#` | ⛔ solid |

Each level sets its own **base ground** (grassy Greenwood, stony Stone Pass and
Traveller's End, dark-tiled Demon Fortress, scorched **charred stone** in the
underworld's Charred Depths), drawn under everything; trees, rocks,
water and barricades are props on top of it that block movement. A **grass patch**
(`G`) is the exception that goes the other way — a *walkable* tuft of greenery
drawn over the base, which is how Traveller's End dots its bare stone with patches
of grass without re-theming the whole floor.

## Roaming enemies

Each screen can hold **roaming enemies**, placed at spawn points and tied to an
[encounter](battles.md). Each level fields its own kind — [slime](entities/slime.md)
swarms in the Greenwood, [gargoyle](entities/gargoyle.md) sentinels on the Stone
Pass, [demon](entities/demon.md) packs — and a lurking [dragon](entities/dragon.md)
boss — in the Demon Fortress, and the [goblin](entities/club_goblin.md) and
[orc](entities/orc_brute.md) families down in the underworld's Charred Depths (see the
[Bestiary](entities/index.md)). They wander
near home until you come within their **aggro radius**, then **chase** you —
routing *around* trees, rocks and barricades rather than snagging on them, so
there's no hiding behind a boulder in the Stone Pass maze. Touch one and its
encounter starts a **[battle](battles.md)** — the dragon's fight even swaps in
its own boss theme.

- You're **faster than they are** (gargoyles and crabs especially are a crawl), so
  you can outrun or juke them — clearing a level doesn't *require* fighting every one
  along the way, but a level is only marked cleared once they're all defeated. The
  mounted **[dark knights](entities/dark_knight.md)** of Traveller's End are the
  exception that nearly keeps pace, so there's little room to dawdle around them.
- **Win** a battle and that enemy is gone from the map for good — and that progress
  is [saved](gameplay.md#saving), so it stays beaten even if you leave and return.
- **Lose** and the enemies in that screen retreat to their starting spots, giving
  you room to regroup instead of being fought again on the spot.

Clear every enemy in every screen and the level is done: the victory report leads
with **"{LEVEL} CLEARED!  ONWARD TO {NEXT}"** and, after any [clear
cutscene](story.md), the party is **carried straight into the next region** — no trip
back to the map to pick it. (Clearing the *final* level ends the run instead —
**"THE OVERWORLD IS SAVED!"** — and leaves you free to return to the map with
**Cancel**/**Menu**.) Every cleared level still shows green on the map if you visit it,
and you can always **Cancel**/**Menu** out of a level to the map by hand.

## Shops

A screen can also hold a **[shop](shops.md)**: a keeper standing under a **SHOP**
banner. Walk up and press **Confirm** to step into a wood-floored store where you
spend the gold you've won on new [weapons and armor](equipment.md). You leave the
way the keeper faces — walk back out the doorway to return to the map. The
Greenwood's opening screen has an **OUTFITTER** right by your start, so you can
kit out before your first fight.

## Chests and mimics

Scattered across the screens are **treasure chests**. Walk up to one and — like a
shop — press **Confirm** on the **PRESS Z** prompt to open it and pocket what's
inside: a purse of **gold**, a consumable [item](items.md), a piece of
[equipment](equipment.md), or some combination. An opened chest is spent (it fades
grey) and, like a beaten enemy, that's [saved](gameplay.md#saving) — it stays
looted even if you leave and come back, so there's no farming the same box twice.
The Greenwood alone hides a starter purse, an **ether**, and a **[SCOUT'S
EDGE](equipment.md)** waiting past its lone demon.

But in the **deep** regions — Traveller's End, the Demon Fortress, the Charred
Depths, and the Demon Facility — not every chest is treasure. A **[mimic](entities/mimic.md)** sits perfectly
still wearing a chest's exact shape — drawn from the **very same sprite**, so from a
distance there is *no way to tell it from the real thing*. Those regions deliberately
sit real chests and mimics side by side, so you can never be sure which cache is safe.
Stray too close and the disguise drops: it reveals a mouthful of teeth and **gives
chase**. A woken mimic is quicker than an ordinary roaming foe, though still a hair
slower than you, so a prompt retreat can shake it — but let it catch you and it starts
a **[battle](battles.md)**.

And a mimic is no pushover in that fight: it is tanky, hurls **FIREBALL**, and — its
signature — **[copies the party's last move](entities/mimic.md#mimicry)**, taking on
that hero's very shape to turn a weakened version of their own skill back on them.
Beat one, though, and it's gone for good and generously paid. Since a dormant mimic
is indistinguishable until it springs, the only safe habit is to approach any deep
chest ready to run.
