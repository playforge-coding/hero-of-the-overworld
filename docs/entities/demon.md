---
comments: true
---

# Demon

The only enemy in the game — and the whole reason for it. Demons are the invaders
that poured out of the underworld when the king was [betrayed](../story.md); they
**roam every level** and chase you when you come near, starting a
[battle](../battles.md) on contact.

| Stat | Value |
|---|---|
| **HP** | 58 |
| **MP** | 12 |
| **Attack** | 16 |
| **Defense** | 8 |
| **Magic** | 9 |
| **Speed** | 9 |
| **AI** | Random (mixes its skill with plain attacks) |
| **Rewards** | 12 XP · 8 gold on defeat |
| **Found in** | Every level, as roaming overworld enemies |

A single demon is a modest threat — [Roland](roland.md) out-HPs and out-damages it —
but they rarely come alone. Its **Random** AI means it sometimes opens with **CLAW**
instead of a basic swing, so it hits a little harder than its stats suggest. Its low
speed (9) means your heroes usually act first.

## Skills

| Skill | Kind | MP | Target |
| ----- | ---- | -- | ------ |
| CLAW | Physical | 0 | One enemy |

**CLAW** is a free physical hit at 125% power — noticeably stronger than a basic
attack. Because it costs no MP, a demon can throw it every round.

## Encounters

Demons are placed on the map in three encounter sizes:

| Encounter | Demons |
| --------- | ------ |
| `demon_solo` | 1 |
| `demon_duo` | 2 |
| `demon_trio` | 3 |

Later levels lean on the larger packs — [Demon Keep](../world.md#the-world-map) opens
with a **trio**. Against a group, area skills like Roland's **WHIRLWIND** or Elara's
**FROST** earn their MP. On the overworld you can also just **outrun** them: you move
faster than a demon, so you can slip past one you'd rather not fight.

## Appearance

The demon uses the `demon` sprite sheet (a 6×8 grid of 16×16 frames): directional
walk rows as it roams the map, and an idle/attack pose in battle.
