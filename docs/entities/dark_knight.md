---
comments: true
---

# Dark Knight

The mounted lords of the pass and the **elite threat** of Traveller's End. A dark
knight rides a warhorse, strikes early, and hits from range with a **CAVALRY
LANCE** — able to run down a single hero with a **LANCE CHARGE** or **TRAMPLE** the
whole party underhoof.

| Stat | Value |
|---|---|
| **HP** | 100 |
| **MP** | 16 |
| **Attack** | 22 (before its weapon) |
| **Defense** | 16 |
| **Magic** | 4 |
| **Speed** | 15 |
| **AI** | Random (mixes basic attack, LANCE CHARGE, and TRAMPLE) |
| **Gear** | [Cavalry Lance](../equipment.md) (+10 ATK, +8% crit, +6% accuracy) |
| **Rewards** | 34 XP · 28 gold on defeat |
| **Drops** | [HI-POTION](../items.md) (25%) · [MIGHT TONIC](../items.md) (15%) |
| **Found in** | Traveller's End |

A dark knight is the closest thing to a boss you'll meet before the Demon Fortress.
Its high **speed** (15) means it usually acts **first**, its **lance** pushes
effective attack to 32 with a solid crit chance, and its **TRAMPLE** hits your
whole line at once. Between its bulk (100 HP) and its defense it also takes a while
to bring down — treat a pair of them, or a knight-and-demon, as a real fight, not a
speed bump.

Its piercing **LANCE CHARGE** is **unblockable** — you can only soften it with
**DEFEND** — so it's the hit to respect. Its **TRAMPLE**, though, is a heavy stomp
you can **block**: time the brace as it lands and a whole-party wipe becomes a
scratch. Against a knight pack — the `knight_guard` on the summit especially —
blocking every TRAMPLE and basic swing is what keeps you standing long enough to
whittle the knights down.

On the overworld it is **mounted and swift** — its roaming speed is close to yours,
so it nearly runs you down. You can still just outpace it, but there's little room
to dawdle.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| **LANCE CHARGE** | Physical | 6 | One enemy | A thundering couched-lance charge (power 145) — **unblockable** |
| **TRAMPLE** | Physical | 8 | All enemies | The warhorse tramples the whole party (power 90) — **blockable**: brace and time it |

## Encounters

Dark knights hold the heights alone, in pairs, and leading mixed bands into the
summit:

| Encounter | Makeup |
| --------- | ------ |
| `knight_solo` | 1 dark knight |
| `knight_duo` | 2 dark knights |
| `knight_skeletons` | 1 dark knight + 2 [skeletons](skeleton.md) |
| `knight_demon` | 1 dark knight + 1 [demon](demon.md) |
| `knight_guard` | 2 dark knights + 1 [skeleton](skeleton.md) |

## Appearance

The dark knight uses a larger **4×12** sheet (`dark_knight.png`, 32×32 frames) with
**three rows per facing** — walk, walk, attack: down 0–2, right 3–5, left 6–8, up
9–11. The right-facing walk and attack rows serve as its battle idle and lance
thrust (flipped to face the party); roaming reads the first walk row of each facing.
