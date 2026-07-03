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

## Heroes

| Hero | HP | MP | ATK | DEF | MAG | SPD | Joins |
| ---- | -- | -- | --- | --- | --- | --- | ----- |
| [Roland](roland.md) | 120 | 24 | 22 | 14 | 10 | 12 | Start (party leader) |
| [Elara](elara.md) | 78 | 60 | 10 | 9 | 26 | 13 | After clearing Greenwood |

## Enemies

| Enemy | HP | MP | ATK | DEF | MAG | SPD | Rewards | Found in |
| ----- | -- | -- | --- | --- | --- | --- | ------- | -------- |
| [Slime](slime.md) | 22 | 0 | 8 | 4 | 0 | 8 | 4 XP · 3 gold | Greenwood, Stone Pass |
| [Gargoyle](gargoyle.md) | 64 | 0 | 20 | 14 | 0 | 3 | 18 XP · 14 gold | Stone Pass |
| [Demon](demon.md) | 68 | 30 | 18 | 9 | 16 | 10 | 16 XP · 12 gold | Greenwood (guardian), Demon Keep |

Each level fields its own foes: **slimes** swarm the Greenwood (with a lone demon
guarding its end), **gargoyles** hold the Stone Pass, and **demon** packs fill the
Demon Keep. Enemies are placed as roaming overworld sprites that chase you and start
a battle on contact — see [The Overworld](../world.md#roaming-enemies).
