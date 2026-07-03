---
comments: true
---

# Demon

The **elite threat** — and the whole reason for the war. Demons are the invaders
that poured out of the underworld when the king was [betrayed](../story.md). They
are held back in the **Demon Fortress**, with a single one guarding the end of the
Greenwood; they roam their levels and chase you when you come near, starting a
[battle](../battles.md) on contact.

| Stat | Value |
|---|---|
| **HP** | 68 |
| **MP** | 30 |
| **Attack** | 18 |
| **Defense** | 9 |
| **Magic** | 16 |
| **Speed** | 10 |
| **AI** | Random (mixes its skills with plain attacks) |
| **Rewards** | 16 XP · 12 gold on defeat |
| **Found in** | Greenwood (a lone guardian) and Demon Fortress (in packs) |

A demon is the game's real test: high HP, strong attack, and the only enemy with
**magic** to spend. Its **Random** AI means it sometimes opens with **CLAW** or a
**FIREBALL** instead of a basic swing, so it hits harder than its bare stats
suggest. At speed 10 it's roughly on pace with your heroes.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| CLAW | Physical | 0 | One enemy | Free 125%-power hit — thrown every round |
| FIREBALL | Magical | 6 | One enemy | 150% power; leaves the target **[BURNING](../battles.md#status-effects)** for 3 rounds |

**CLAW** costs no MP, so a demon can always fall back on it. **FIREBALL** is the
dangerous one: it inflicts **BURN**, ticking 8 damage a round for three rounds, so a
drawn-out fight bleeds you. Burning demons down quickly — before the damage-over-time
stacks up — pays off.

## Encounters

Demons are placed on the map in three encounter sizes:

| Encounter | Demons | Where |
| --------- | ------ | ----- |
| `demon_solo` | 1 | Guarding the far end of the Greenwood |
| `demon_duo` | 2 | Demon Fortress |
| `demon_trio` | 3 | Demon Fortress |

The **Demon Fortress** leans on the larger packs and opens with a **trio**. Against a
group, area skills like Roland's **WHIRLWIND** or Elara's **FROST** earn their MP. On
the overworld you can also just **outrun** them: you move faster than a demon, so you
can slip past one you'd rather not fight.

## Appearance

The demon uses the `demon` sprite sheet (a 6×8 grid of 16×16 frames): directional
walk rows (0–3) as it roams the map, and attack poses (rows 4–7) plus an idle in
battle. The **[gargoyle](gargoyle.md)** reuses this same sheet layout with different
art.
