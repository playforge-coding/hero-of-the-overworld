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
- **Confirm** enters the highlighted level.
- A marker is **red** while the level still holds demons and turns **green with a
  star** once you've cleared it. A **CLEARED x/y** tally sits in the corner.

The bundled world has three levels:

| Level | Screens | Notes |
| ----- | ------- | ----- |
| **GREENWOOD** | 3 | The opening level. Clearing it triggers the cutscene where ELARA joins. |
| **STONE PASS** | 2 | A rockier route, screens stacked north–south. |
| **DEMON KEEP** | 2 | The toughest, thick with demons (including trios). |

## Screens

A level is a handful of **screens** (rooms) you walk between, Zelda-style. Each
screen is a grid of 16×16 tiles. The party leader walks freely in pixels with
per-axis collision, sliding along walls rather than sticking, and the **camera
follows**, clamped so it never shows past the edge of a small screen.

Walk into the **opening in the middle of an edge** — where the border tiles stop
— and, if that side links to another screen, you flip to it and appear at the
opposite edge. A brief grace period after arriving stops a demon from instantly
pouncing on you.

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

## Roaming demons

Each screen can hold **roaming demons**, placed at spawn points and tied to an
[encounter](battles.md). They wander near home until you come within their
**aggro radius**, then **chase** you. Touch one and its encounter starts a
**[battle](battles.md)**.

- You're **faster than the demons**, so you can outrun or juke them — clearing a
  level doesn't *require* fighting every one along the way, but a level is only
  marked cleared once they're all defeated.
- **Win** a battle and that demon is gone from the map for good.
- **Lose** and the demons in that screen retreat to their starting spots, giving
  you room to regroup instead of being fought again on the spot.

Clear every demon in every screen and the level shows **LEVEL CLEARED!** — press
**Cancel**/**Menu** to return to the map (now green) and pick your next level.
