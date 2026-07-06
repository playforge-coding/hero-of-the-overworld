---
comments: true
---

# Slime

The weakest foe in the game — swarm fodder. Slimes are individually harmless, but
they **come three at a time**, and a full pack can whittle a careless party down.
They fill the opening **Greenwood** and turn up again on the **Stone Pass**.

| Stat | Value |
|---|---|
| **HP** | 22 |
| **MP** | 0 |
| **Attack** | 8 |
| **Defense** | 4 |
| **Magic** | 0 |
| **Speed** | 8 |
| **AI** | Basic (always a plain attack on a random hero) |
| **Rewards** | 4 XP · 3 gold on defeat |
| **Drops** | [POTION](../items.md) (25%) |
| **Found in** | Greenwood and Stone Pass |

A slime has no skills and no MP — its **Basic** AI just swings at a random hero every
turn. With only 22 HP each, they fall fast, but three of them attacking at once still
adds up, so don't ignore a swarm. They make good, low-risk XP for levelling early.

## Skills

None. Slimes only ever use the basic attack.

## Encounters

Slimes always appear as a **trio**:

| Encounter | Slimes | Where |
| --------- | ------ | ----- |
| `slime_trio` | 3 | Greenwood (several packs) and one pack on the Stone Pass |

Against three at once, a single area skill — Roland's **WHIRLWIND** or Elara's
**FROST** — can clear or badly soften the whole group in one cast.

## Appearance

The slime uses the `slime` sprite sheet (a 6×4 grid of 16×16 frames): rows 0–3 are
the walk cycles (down/up/right/left) it uses both roaming the map and standing in
battle. It has no dedicated attack rows, so its "attack" just replays the walk row a
little faster — a quick hop.
