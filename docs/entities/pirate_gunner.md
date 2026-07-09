---
comments: true
---

# Pirate Gunner

The crew's flintlock skirmisher — a **glass cannon** that cracks a **PISTOL SHOT**
across the sand from range. Like the goblins' [archer](archer_goblin.md), it's thin
on HP and armor and folds fast once you close the gap, but let it plink you from afar
and it stings. It holds the **[Castaway Shore](../world.md#progression-is-linear)**
alongside the cutlass-swinging **[grunts](pirate_grunt.md)** it fires from behind.

| Stat | Value |
|---|---|
| **HP** | 54 |
| **MP** | 0 |
| **Attack** | 22 |
| **Defense** | 9 |
| **Magic** | 0 |
| **Speed** | 16 |
| **AI** | Random (mixes its shot with close-in swings) |
| **Rewards** | 30 XP · 24 gold on defeat |
| **Drops** | [ETHER](../items.md) (15%) · [POTION](../items.md) (20%) |
| **Found in** | Castaway Shore (pirate camps, behind the barricades) |

The gunner is fast (speed 16, so it often acts first) but **fragile** — the least
armor on the beach. Its **Random** AI mixes plain swings with the **PISTOL SHOT**, so
it won't always fire, but when it does the shot **can't be blocked**. The play is the
same as against any skirmisher: **close the distance and drop it early**, before it
lines up too many shots.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| PISTOL SHOT | Physical | 0 | One enemy | 135% power, **unblockable** — a lead ball cracked across the sand |

**PISTOL SHOT** is a ranged, piercing attack, so (like every ranged blow) it ignores
your timed **block** — you can't shrug it off, only avoid taking too many by killing
the gunner quickly.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `gunner_line` | 2 pirate gunners | Castaway Shore (a firing line) |
| `pirate_ambush` | 1 [grunt](pirate_grunt.md) + 1 gunner | Castaway Shore |
| `pirate_squad` | 2 [grunts](pirate_grunt.md) + 1 gunner | Castaway Shore camps |
| `pirate_crew` | 2 [grunts](pirate_grunt.md) + 2 gunners | Castaway Shore (the camp muster) |

## Appearance

The pirate gunner uses the `pirate_gunner` sprite sheet, drawn on the **hermit's
sheet layout** — a 5×8 grid of 16×16 frames: directional walk rows (0–3) on the sand,
attack poses (rows 4–7) plus a front-facing idle in battle. It shares that layout with
the **[pirate grunt](pirate_grunt.md)**.
