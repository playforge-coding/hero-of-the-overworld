---
comments: true
---

# Bestiary

Every character in Hero of the Overworld — hero and demon alike — has a page here
with its stat block, skills, and notes on how it fights. All of these numbers come
straight from the game's content file, [`assets/data/game.ron`](../modding.md), so
if you edit that file your own additions belong here too.

Stats are the six values every battler shares:

- **HP / MP** — health and skill points (carried between battles for heroes).
- **Attack** — powers physical hits.
- **Defense** — softens incoming damage.
- **Magic** — powers spells and heals.
- **Speed** — decides who acts first each round.

The tables below list **base** stats. On top of these, a hero's equipped
**[weapon and armor](../equipment.md)** add to attack/defense/magic/speed and grant
combat attributes — **crit**, **accuracy**, and **evasion** — that drive hits,
misses, and critical strikes. See **[Battles](../battles.md#hit-miss-and-crit)** for
how everything combines into damage.

Enemy numbers here are their **base (party-level-1)** stats. In play, roaming
enemies are **[scaled to your party's level](../gameplay.md#enemies-scale-with-you)**
so they keep pace as you grow — every stat except **speed**, which stays fixed to
preserve the turn order.

## Heroes

| Hero | HP | MP | ATK | DEF | MAG | SPD | Joins |
| ---- | -- | -- | --- | --- | --- | --- | ----- |
| [Roland](roland.md) | 120 | 24 | 22 | 14 | 10 | 12 | Start (party leader) |
| [Elara](elara.md) | 78 | 60 | 10 | 9 | 26 | 13 | After clearing Greenwood |
| [Gareth](gareth.md) | 94 | 30 | 20 | 11 | 8 | 21 | After clearing Traveller's End |

## Enemies

| Enemy | HP | MP | ATK | DEF | MAG | SPD | Rewards | Found in |
| ----- | -- | -- | --- | --- | --- | --- | ------- | -------- |
| [Slime](slime.md) | 22 | 0 | 8 | 4 | 0 | 8 | 4 XP · 3 gold | Greenwood, Stone Pass, Traveller's End |
| [Mountain Crab](mountain_crab.md) | 30 | 0 | 11 | 13 | 0 | 5 | 6 XP · 5 gold | Traveller's End |
| [Skeleton](skeleton.md) | 46 | 8 | 17 | 7 | 2 | 12 | 15 XP · 11 gold | Traveller's End |
| [Gargoyle](gargoyle.md) | 64 | 0 | 20 | 14 | 0 | 3 | 18 XP · 14 gold | Stone Pass, Traveller's End |
| [Demon](demon.md) | 68 | 30 | 18 | 9 | 16 | 10 | 16 XP · 12 gold | Greenwood (guardian), Traveller's End, Demon Fortress |
| [Dark Knight](dark_knight.md) | 100 | 16 | 22 | 16 | 4 | 15 | 34 XP · 28 gold | Traveller's End |
| [Dragon](dragon.md) | 340 | 80 | 26 | 17 | 24 | 11 | 150 XP · 120 gold | Demon Fortress (boss) |
| [Club Goblin](club_goblin.md) | 42 | 0 | 16 | 7 | 0 | 13 | 14 XP · 10 gold | Charred Depths |
| [Archer Goblin](archer_goblin.md) | 34 | 0 | 14 | 6 | 0 | 14 | 15 XP · 11 gold | Charred Depths |
| [Orc Brute](orc_brute.md) | 96 | 0 | 25 | 13 | 0 | 8 | 30 XP · 24 gold | Charred Depths |

Each level fields its own foes: **slimes** swarm the Greenwood (with a lone demon
guarding its end), **gargoyles** hold the Stone Pass, the long climb of **Traveller's
End** throws **crabs**, **skeletons** and mounted **[dark knights](dark_knight.md)**
at you (with stray demons up from below), and **demon** packs fill the Demon Fortress
— where a lone **[dragon](dragon.md)** boss waits in the depths. Beyond it, in the
underworld's **Charred Depths**, the war's remade prisoners lie in wait: the
**[goblin](club_goblin.md) family** ([clubbers](club_goblin.md) and
[archers](archer_goblin.md)) and the hulking **[orc brutes](orc_brute.md)**. Enemies
are placed as roaming overworld sprites that chase you and start a battle on contact —
see [The Overworld](../world.md#roaming-enemies). The dragon's fight plays a dedicated
**boss theme** in place of the usual battle music.
