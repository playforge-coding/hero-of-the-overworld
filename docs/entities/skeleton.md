---
comments: true
---

# Skeleton

The restless undead of **Traveller's End**, rattling up out of the high boneyards.
A skeleton is a middling all-round fighter — no glaring weakness, no great strength
— that swings a **REAP** with its unpredictable AI.

| Stat | Value |
|---|---|
| **HP** | 46 |
| **MP** | 8 |
| **Attack** | 17 |
| **Defense** | 7 |
| **Magic** | 2 |
| **Speed** | 12 |
| **AI** | Random (mixes its basic attack with REAP) |
| **Rewards** | 15 XP · 11 gold on defeat |
| **Found in** | Traveller's End |

Skeletons are the backbone of the pass's mid-tier packs. They act at a fair clip
(speed 12, on par with your heroes) and their **REAP** hits noticeably harder than
a plain swing, so a trio can wear you down if you let them all connect. Their thin
bones give them **low defense**, though — they fold fast to focused fire.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| **REAP** | Physical | 0 | One enemy | A rusted blade dragged across one foe (power 135) |

## Encounters

Skeletons roam the shelves in twos and threes, and stand alongside the dark
knights deeper in:

| Encounter | Makeup |
| --------- | ------ |
| `skeleton_duo` | 2 skeletons |
| `skeleton_trio` | 3 skeletons |
| `knight_skeletons` | 1 [dark knight](dark_knight.md) + 2 skeletons |
| `knight_guard` | 2 [dark knights](dark_knight.md) + 1 skeleton |

## Appearance

The skeleton shares **Roland's** sheet layout exactly (`skeleton.png`, a 5×12 grid
of 16×16 frames): walk rows 0–3 (down/up/right/left) for roaming, attack rows 4–7,
and — like Roland — its walk-right row reads as the battle idle with the
attack-right row (6) as the swing. As an enemy it's flipped to face the party.
