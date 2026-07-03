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

See **[Battles](../battles.md)** for how these combine into damage.

## Heroes

| Hero | HP | MP | ATK | DEF | MAG | SPD | Joins |
| ---- | -- | -- | --- | --- | --- | --- | ----- |
| [Roland](roland.md) | 120 | 24 | 22 | 14 | 10 | 12 | Start (party leader) |
| [Elara](elara.md) | 78 | 60 | 10 | 9 | 26 | 13 | After clearing Greenwood |

## Enemies

| Enemy | HP | MP | ATK | DEF | MAG | SPD | Rewards |
| ----- | -- | -- | --- | --- | --- | --- | ------- |
| [Demon](demon.md) | 58 | 12 | 16 | 8 | 9 | 9 | 12 XP · 8 gold |

Demons appear in three encounter sizes — **solo**, **duo**, and **trio** — placed as
roaming enemies across the levels. See [The Overworld](../world.md#roaming-demons).
