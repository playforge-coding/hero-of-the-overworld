---
comments: true
---

# Archer Goblin

The **goblin family's skirmisher** — a glass cannon that strikes from range. Like the
[club goblins](club_goblin.md), archer goblins are prisoners the
[underworld](../story.md) broke and **indoctrinated into demon society**; where the
clubbers rush in, the archers hang back and loose a piercing **ARROW SHOT**. Thin on
HP and defense, an archer folds fast once you close the distance — the trick is
surviving the volley on the way in.

| Stat | Value |
|---|---|
| **HP** | 46 |
| **MP** | 0 |
| **Attack** | 20 |
| **Defense** | 9 |
| **Magic** | 0 |
| **Speed** | 16 |
| **AI** | Random (mixes ARROW SHOT with plain swings) |
| **Rewards** | 24 XP · 18 gold on defeat |
| **Drops** | [POTION](../items.md) (20%) |
| **Found in** | Charred Depths (screened by club goblins) |

At speed 16 an archer goblin acts before nearly all of your party, so its arrow almost
always lands first. It is the **priority target** in any goblin pack: kill the archers
early and the clubbers become a straightforward brawl.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| ARROW SHOT | Physical | 0 | One enemy | 120% power; a **piercing** arrow that flies to its mark and **ignores a timed block** |

**ARROW SHOT** is loosed as a flying **arrow** [projectile](../battles.md) — it lands
as it arrives, and like every ranged or piercing blow it is **unblockable**, so there
is no timing window to null it. You cannot brace against the arrow; you can only end
the archer before it fires again.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `goblin_pair` | 1 [club goblin](club_goblin.md) + 1 archer goblin | Charred Depths (upper halls) |
| `goblin_pack` | 2 [club goblins](club_goblin.md) + 1 archer goblin | Charred Depths (upper halls, the nest) |
| `archer_duo` | 2 archer goblins | Charred Depths (the crossing) |
| `orc_goblins` | 1 [orc brute](orc_brute.md) + 1 [club goblin](club_goblin.md) + 1 archer goblin | Charred Depths (deeper chambers) |

## Appearance

The archer goblin uses the `archer_goblin` sprite sheet (a 5×8 grid of 16×16 frames):
directional walk rows (0–3) as it roams the map, and attack (bow-draw) poses (rows
4–7) plus an idle in battle — the same layout the [demon](demon.md) sheet uses. Its
arrow is the `arrow` projectile sprite.
