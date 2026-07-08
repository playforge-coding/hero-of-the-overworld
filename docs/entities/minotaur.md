---
comments: true
---

# Minotaur

A **lumbering miniboss**: a horned bull-man that lowers its head to **charge clean
through** a hero, **tramples** the whole party underhoof, and — surprisingly — hurls
a **fireball**. It fights in a **boss-flagged encounter**, so its battle plays the
dedicated **boss theme** in place of the usual battle music, exactly like the
[dragon](dragon.md).

!!! note "Future foe"
    The minotaur is **defined but not yet placed** in any level's spawns. It is
    fully playable now through the **[developer fight menu](../modding.md#the-developer-menu-f1)**
    (press <kbd>F1</kbd> on the world map in a debug build → **FIGHT ENCOUNTER** →
    `minotaur_boss`), ready to anchor a future region as a miniboss.

| Stat | Value |
|---|---|
| **HP** | 240 |
| **MP** | 50 |
| **Attack** | 30 |
| **Defense** | 18 |
| **Magic** | 20 |
| **Speed** | 10 |
| **AI** | Random (rotates its three skills with plain swings) |
| **Rewards** | 90 XP · 70 gold on defeat |
| **Drops** | [MIGHT TONIC](../items.md) (25%) · [HI-POTION](../items.md) (20%) |
| **Found in** | Unplaced — reachable via the developer fight menu |

The minotaur sits between an [orc brute](orc_brute.md) and the [dragon](dragon.md):
a **240-HP wall** with a broad, mixed threat kit. It's lumbering — speed 10, and
**slower than the player** on the map (like a [gargoyle](gargoyle.md) or an
[orc brute](orc_brute.md)), so you can dodge it while roaming — but in a fight its
**Random** AI cycles single-target, party-wide, and magical damage, so you can't
plan around a single incoming blow.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| GORING CHARGE | Physical | 6 | One enemy | 150% power; **piercing** (can't be blocked) — barrels through a hero, wrapping the field |
| TRAMPLE | Physical | 8 | All enemies | 90% power; a screen-wrapping charge across the **whole party** |
| FIREBALL | Magical | 6 | One enemy | 150% power; leaves the target **[BURNING](../battles.md#status-effects)** for 3 rounds |

Two of its three moves are **[screen-wrapping charges](../battles.md)**: **GORING
CHARGE** picks one hero and can't be timed-blocked, while **TRAMPLE** sweeps the
entire party at once. Between them it also throws a **FIREBALL** that inflicts
**BURN**. Because **TRAMPLE** hits everyone, keeping the party topped up — and
racing the burn — matters more than bracing for any one strike.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `minotaur_boss` | 1 minotaur (**boss theme**) | Unplaced (developer fight menu) |

## Appearance

The minotaur uses the `minotaur` sprite sheet — the [demon](demon.md)'s sheet with
the **walk-left row removed**, making it a **6×7** grid of 16×16 frames (one row
shorter than the demon's 6×8). Rows 0–2 are the walk-down/up/right cycles (four
frames each) and rows 3–6 hold the attack poses (shifted up one from the demon's
4–7). With no dedicated left-walk row, the overworld map **reuses the right-walk
row** for leftward movement, so the roaming minotaur faces right whichever way it
strides — a small quirk of the trimmed sheet.
