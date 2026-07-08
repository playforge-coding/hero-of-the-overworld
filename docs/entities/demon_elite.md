---
comments: true
---

# Demon Elite

A **[demon](demon.md) in war-plate** — the same fire and claws, bolted onto
orc-like brawn. Where a common demon is already the game's elite threat, the elite
is what a demon becomes when it straps on armor and steps up to the front line:
tougher, heavier-hitting, and still able to spend **magic**.

The demon elite is the rank and file of the **[Demon Facility](../world.md#progression-is-linear)**,
Chapter 1's final region — where the horde plates its demons for war and fields them
two and three abreast all the way down to the **[Demon King](demon_king.md)**'s throne.

| Stat | Value |
|---|---|
| **HP** | 140 |
| **MP** | 34 |
| **Attack** | 30 |
| **Defense** | 14 |
| **Magic** | 18 |
| **Speed** | 11 |
| **AI** | Random (mixes its skills with plain attacks) |
| **Rewards** | 40 XP · 34 gold on defeat |
| **Drops** | [MIGHT TONIC](../items.md) (18%) · [HI-POTION](../items.md) (20%) |
| **Found in** | Demon Facility (its rank-and-file patrols, singly and in packs) |

Compared with a plain [demon](demon.md), the elite roughly **doubles the HP** and
carries an **orc-brute-grade attack** (30, up from 18) behind a stiffer defense,
while keeping the demon's magic to fuel **FIREBALL**. It still acts on pace with the
party at speed 11, so it fights like a demon that simply refuses to go down.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| CLAW | Physical | 0 | One enemy | Free 125%-power hit — thrown every round |
| FIREBALL | Magical | 6 | One enemy | 150% power; leaves the target **[BURNING](../battles.md#status-effects)** for 3 rounds |

The elite shares the demon's exact kit: a free **CLAW** to fall back on, and a
**FIREBALL** that inflicts **BURN** (8 damage a round for three rounds). With far
more HP to hide behind, its burn has more time to bleed you — so the same advice
applies, only more so: **burn it down fast**.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `demon_elite_solo` | 1 demon elite | Developer fight menu |
| `elite_pair` | 2 demon elites | Demon Facility patrols |
| `elite_trio` | 3 demon elites | Demon Facility heavy patrols |
| `elite_demons` | 1 demon elite + 2 [demons](demon.md) | Demon Facility (an officer and its escort) |
| `elite_guard` | 2 demon elites + 1 [demon](demon.md) | Demon Facility (the throne-hall guard) |

## Appearance

The demon elite uses the `demon_elite` sprite sheet — the [demon](demon.md)'s own
6×8 grid of 16×16 frames **redrawn in armor**, with the layout unchanged:
directional walk rows (0–3) as it roams, and attack poses (rows 4–7) plus an idle in
battle. It is the third foe to share this sheet layout, after the
[gargoyle](gargoyle.md) and the original [demon](demon.md).
