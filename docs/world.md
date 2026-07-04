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
- A marker is **red** while the level still holds demons and turns **green with a
  star** once you've cleared it. A **CLEARED x/y** tally sits in the corner.
- A **grey marker with an X** is still **locked**.

### Progression is linear

You must **clear a level to unlock the next one**. The first level is always
open; each later one stays locked (and can't be entered) until the level before
it is fully cleared. Selecting a locked level shows *"CLEAR THE PREVIOUS LEVEL
FIRST"* instead of the usual prompt, so the world opens up as you win.

The bundled world has three levels — and the Demon Fortress is not the last
stop, only the deepest reached so far:

| Level | Screens | Enemies | Notes |
| ----- | ------- | ------- | ----- |
| **GREENWOOD** | 5 | [Slime](entities/slime.md) swarms + a lone [demon](entities/demon.md) guardian | The opening level: a straightforward forest walk with some winding parts, flowing east then bending north into the deep wood. Clearing it triggers the cutscene where ELARA joins. |
| **STONE PASS** | 5 | [Gargoyle](entities/gargoyle.md) patrols + a slime pack | A straight rocky descent (screens stacked north–south) that ends in a **boulder maze** you must snake through. Unlocks after Greenwood. |
| **DEMON FORTRESS** | 6 | [Demon](entities/demon.md) packs (duos and trios) + a [dragon](entities/dragon.md) boss | The toughest so far — a **very mazelike** warren of dark-brick corridors: a gatehouse into a crossing that branches to a dead-end cell block or deeper east through twisting galleries to the dragon's lair. Unlocks after Stone Pass. |

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
| Grass | `.` (or space) | ✅ the only walkable ground |
| Tree | `T` | ⛔ solid — units path around it |
| Rock | `R` | ⛔ solid |
| Water | `~` | ⛔ solid |
| Barricade | `#` | ⛔ solid |

Grass is the base layer drawn under everything; trees and rocks are drawn as
props sitting on top of it. Anything that isn't grass blocks movement, so screens
are shaped by their walls of trees, scattered rocks, ponds, and barricades.

## Roaming enemies

Each screen can hold **roaming enemies**, placed at spawn points and tied to an
[encounter](battles.md). Each level fields its own kind — [slime](entities/slime.md)
swarms in the Greenwood, [gargoyle](entities/gargoyle.md) sentinels on the Stone
Pass, [demon](entities/demon.md) packs — and a lurking [dragon](entities/dragon.md)
boss — in the Demon Fortress (see the [Bestiary](entities/index.md)). They wander
near home until you come within their **aggro radius**, then **chase** you. Touch
one and its encounter starts a **[battle](battles.md)** — the dragon's fight even
swaps in its own boss theme.

- You're **faster than they are** (gargoyles especially are a crawl), so you can
  outrun or juke them — clearing a level doesn't *require* fighting every one along
  the way, but a level is only marked cleared once they're all defeated.
- **Win** a battle and that enemy is gone from the map for good — and that progress
  is [saved](gameplay.md#saving), so it stays beaten even if you leave and return.
- **Lose** and the enemies in that screen retreat to their starting spots, giving
  you room to regroup instead of being fought again on the spot.

Clear every enemy in every screen and the level shows **LEVEL CLEARED!** — press
**Cancel**/**Menu** to return to the map (now green), which **unlocks the next
level** to pick.

## Shops

A screen can also hold a **[shop](shops.md)**: a keeper standing under a **SHOP**
banner. Walk up and press **Confirm** to step into a wood-floored store where you
spend the gold you've won on new [weapons and armor](equipment.md). You leave the
way the keeper faces — walk back out the doorway to return to the map. The
Greenwood's opening screen has an **OUTFITTER** right by your start, so you can
kit out before your first fight.
