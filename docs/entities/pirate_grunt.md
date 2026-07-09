---
comments: true
---

# Pirate Grunt

The cutlass-swinging muscle of the marauders who hold the **[Castaway
Shore](../world.md#progression-is-linear)**. No tricks — just a hard, reliable melee
bruiser who wades in and hacks. Tougher and harder-hitting than the goblins of the
depths, the grunt anchors the pirate crews while the **[gunners](pirate_gunner.md)**
plink from behind it.

| Stat | Value |
|---|---|
| **HP** | 78 |
| **MP** | 0 |
| **Attack** | 26 |
| **Defense** | 13 |
| **Magic** | 0 |
| **Speed** | 13 |
| **AI** | Basic (always attacks) |
| **Rewards** | 28 XP · 22 gold on defeat |
| **Drops** | [POTION](../items.md) (20%) |
| **Found in** | Castaway Shore (pirate patrols and camps) |

The grunt is a **pure melee threat**: a stiff attack of 26 behind a decent chunk of
HP, and a blockable cutlass swing — so a well-timed **block** shaves its damage. It
carries no skills and no ranged option, so the counter is simple: close with it and
trade, and keep an eye out for the gunner it's usually screening.

## Skills

The pirate grunt has no skills — it only ever throws a plain **cutlass swing** (a
basic, blockable attack).

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `pirate_duo` | 2 pirate grunts | Castaway Shore patrols |
| `pirate_ambush` | 1 grunt + 1 [gunner](pirate_gunner.md) | Castaway Shore |
| `pirate_squad` | 2 grunts + 1 [gunner](pirate_gunner.md) | Castaway Shore camps |
| `pirate_crew` | 2 grunts + 2 [gunners](pirate_gunner.md) | Castaway Shore (the camp muster) |

## Appearance

The pirate grunt uses the `pirate_grunt` sprite sheet, drawn on the **hermit's sheet
layout** — a 5×8 grid of 16×16 frames: directional walk rows (0–3) as it roams the
sand, and attack poses (rows 4–7) plus a front-facing idle in battle. It shares that
layout exactly with the **[pirate gunner](pirate_gunner.md)**.
