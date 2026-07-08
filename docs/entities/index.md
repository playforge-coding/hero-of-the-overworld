---
comments: true
---

# Bestiary

Every character in Hero of the Overworld — hero and demon alike — has a page here
with its stat block, skills, and notes on how it fights. All of these numbers come
straight from the game's [content files](../modding.md) (each enemy is its own
`assets/data/enemies/<id>.ron`), so if you add one your own page belongs here too.

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
| [Demon](demon.md) | 68 | 30 | 18 | 9 | 16 | 10 | 16 XP · 12 gold | Greenwood (guardian), Traveller's End, Demon Fortress, Demon Facility |
| [Dark Knight](dark_knight.md) | 100 | 16 | 22 | 16 | 4 | 15 | 34 XP · 28 gold | Traveller's End |
| [Dragon](dragon.md) | 340 | 80 | 26 | 17 | 24 | 11 | 150 XP · 120 gold | Demon Fortress (boss) |
| [Club Goblin](club_goblin.md) | 58 | 0 | 22 | 11 | 0 | 14 | 22 XP · 16 gold | Charred Depths |
| [Archer Goblin](archer_goblin.md) | 46 | 0 | 20 | 9 | 0 | 16 | 24 XP · 18 gold | Charred Depths |
| [Orc Brute](orc_brute.md) | 150 | 0 | 34 | 19 | 0 | 9 | 48 XP · 40 gold | Charred Depths |
| [Ballista](ballista.md) | 160 | 0 | 24 | 18 | 0 | 3 | 30 XP · 22 gold | Traveller's End, Charred Depths |
| [Mimic](mimic.md) | 132 | 40 | 26 | 14 | 24 | 14 | 55 XP · 80 gold | Traveller's End, Demon Fortress, Charred Depths (disguised as chests) |
| [Demon Elite](demon_elite.md) | 140 | 34 | 30 | 14 | 18 | 11 | 40 XP · 34 gold | Demon Facility (its rank-and-file patrols) |
| [Demon King](demon_king.md) | ∞ | 0 | 45 | 30 | 60 | 40 | — (never defeated) | Demon Facility (unwinnable throne-room boss) |
| [Minotaur](minotaur.md) | 240 | 50 | 30 | 18 | 20 | 10 | 90 XP · 70 gold | Unplaced (dev fight menu, boss theme) |
| [Dark Roland](clones.md#dark-roland) | 120 | 24 | 22 | 14 | 10 | 12 | 70 XP · 55 gold | Unplaced (mirror-match boss) |
| [Dark Elara](clones.md#dark-elara) | 78 | 60 | 10 | 9 | 26 | 13 | 68 XP · 55 gold | Unplaced (mirror-match boss) |
| [Dark Gareth](clones.md#dark-gareth) | 94 | 30 | 20 | 11 | 8 | 21 | 68 XP · 55 gold | Unplaced (mirror-match boss) |

The **[minotaur](minotaur.md)** miniboss is **defined but not yet placed** in any
level, and neither are the three **[clones](clones.md)** — dark stat-mirrors of the
party that meet you only in the **mirror-match** boss. Both are playable now through
the **[developer fight menu](../modding.md#the-developer-menu-f1)**, waiting to be
dropped into a future region.

Each level fields its own foes: **slimes** swarm the Greenwood (with a lone demon
guarding its end), **gargoyles** hold the Stone Pass, the long climb of **Traveller's
End** throws **crabs**, **skeletons** and mounted **[dark knights](dark_knight.md)**
at you (with stray demons up from below), and **demon** packs fill the Demon Fortress
— where a lone **[dragon](dragon.md)** boss waits in the depths. Beyond it, in the
underworld's **Charred Depths**, the war's remade prisoners lie in wait: the
**[goblin](club_goblin.md) family** ([clubbers](club_goblin.md) and
[archers](archer_goblin.md)) and the hulking **[orc brutes](orc_brute.md)**. Past
even that, in the iron halls of the **[Demon Facility](../world.md#progression-is-linear)**,
war-plated **[demon elites](demon_elite.md)** muster two and three abreast, all the
way down to the throne of the **[Demon King](demon_king.md)** — an unwinnable boss
whose fight ends Chapter 1. Enemies are placed as roaming overworld sprites that chase
you and start a battle on contact — see [The Overworld](../world.md#roaming-enemies).
The dragon's and the Demon King's fights each play a dedicated **boss theme** in place
of the usual battle music.

The **deep** regions — Traveller's End, the Demon Fortress, the Charred Depths, and
the Demon Facility — hide **[mimics](mimic.md)**: powerful ambush predators disguised as
[treasure chests](../world.md#chests-and-mimics). A dormant mimic is drawn from the
very same sprite as a real chest, so you can't tell them apart at a glance; it lies
still until you stray close, then reveals its teeth and gives chase. In battle it
hurls FIREBALL, bites hard, and — its signature — **[copies the party's last move](mimic.md#mimicry)**,
taking on that hero's shape to turn a nerfed version of their own skill back on them.

Not every foe is a creature: the **[ballista](ballista.md)** is a **[tool
enemy](../battles.md#tool-enemies)** — an inert siege engine worked by the foes
beside it, hammering the party with heavy **BOLT**s until you fell its crew (at
which point it crumbles) or smash the engine itself. It holds the shelves of
Traveller's End and the deep emplacements of the Charred Depths.
