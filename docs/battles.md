---
comments: true
---

# Battles

Touching a roaming enemy on the [overworld](world.md#roaming-enemies) drops you
into a turn-based battle against that encounter's foes — [slimes](entities/slime.md),
[gargoyles](entities/gargoyle.md), or [demons](entities/demon.md). Your whole party
fights whoever is present.

## A round, step by step

1. **Command phase.** For each living hero, in speed order, you pick an action:
   **ATTACK**, a **SKILL**, or **DEFEND**. You can **Cancel** back to the previous
   hero to re-plan their choice.
2. **Enemies plan.** Each enemy chooses an action via its simple AI.
3. **Resolve.** Everyone's actions are sorted by **speed** (fastest first) and
   played out one at a time, with a little lunge animation and floating damage /
   heal numbers.
4. **Check the outcome.** If all enemies are down you **win**; if all heroes are
   down you **lose**. Otherwise a new round begins.

## The command menu

| Command | What it does |
| ------- | ------------ |
| **ATTACK** | A basic physical strike on one enemy you choose. Free. |
| **SKILL** | Open the hero's skill list and pick one (costs MP). Then choose a target if the skill needs one. |
| **DEFEND** | Brace: incoming damage to this hero is reduced until their next turn. |

Choosing **SKILL** shows what that hero knows, each skill's MP cost, and a short
**description** of the highlighted skill. A skill you can't afford can't be
selected. Depending on the skill you'll then choose a **single** target or it will
hit **all** valid targets at once. While you're picking a command, a panel also
shows the acting hero's equipped **[weapon and armor](#weapons-armor)**.

## Skills

Every skill is one of three kinds and hits a chosen target set:

- **Physical** — scales off the user's **attack**.
- **Magical** — scales off the user's **magic**.
- **Heal** — restores HP to an ally, scaling off **magic**.

The bundled skills:

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| POWER STRIKE | Physical | 4 | One enemy | Heavy single-target hit |
| WHIRLWIND | Physical | 8 | All enemies | Sweep the whole enemy line |
| MEND | Heal | 6 | One ally | Patch up a wounded hero |
| FIREBOLT | Magical | 6 | One enemy | ELARA's hard-hitting bolt |
| FROST | Magical | 12 | All enemies | Chills every enemy at once |
| CLAW | Physical | 0 | One enemy | The demon's free claw swipe |
| FIREBALL | Magical | 6 | One enemy | The demon's spell — inflicts **BURN** (see below) |

(POWER STRIKE / WHIRLWIND / MEND are Roland's; FIREBOLT / FROST / MEND are Elara's;
CLAW and FIREBALL belong to the [demon](entities/demon.md). Slimes and gargoyles have
no skills — they only ever use a basic attack.)

## Status effects

Some skills leave a lingering **status** on their target that ticks at the end of
each round until it wears off:

| Status | Applied by | Effect |
| ------ | ---------- | ------ |
| **BURN** | Demon's **FIREBALL** | Deals 8 damage at the end of each round for 3 rounds |

Statuses are pure data too — a new condition (poison, regen, slow, …) is just an
entry in the data file referenced from a skill, with no engine change. See
**[Extending the Game](modding.md)**.

## Hit, miss, and crit

Before an attack deals damage, it has to **land** — and it might land **hard**.

- **Miss.** Every offensive hit rolls against an accuracy check. The chance to
  land starts around **92%**, rises with the attacker's **accuracy** (from
  weapons) and **speed**, and falls with the target's **evasion** (from armor) and
  speed. It's clamped so nothing is ever a sure thing or hopeless. A miss shows a
  grey **MISS** and deals no damage.
- **Critical hit.** A landed hit has a small chance (about **5%**, higher with a
  weapon's **crit** bonus) to **crit**, dealing **+50% damage**. Crits flash
  brighter and show their number in orange with an exclamation mark.

Heals always land and never crit.

## How damage works

A landed hit's damage is based on the attacker's offensive stat and the skill's
power, softened by the target's defense, boosted if it crit, then nudged by a
small random spread:

```
damage ≈ (offense × power / 100) − defense / 2      (at least 1)
        × 1.5 on a critical hit
        × 0.88–1.12 random variance
        × 0.66 if the target is defending
```

- **ATTACK** uses power 100; skills use their own power value.
- **offense** is **attack** for physical hits and **magic** for magical ones —
  both raised by your **[equipped weapon](#weapons-armor)**.
- **defense** is raised by your **[equipped armor](#weapons-armor)**.
- **MEND** and other heals restore HP up to the target's maximum.

So high **attack**/**magic** hits harder, high **defense** absorbs more, high
**speed** acts earlier *and* helps you hit and dodge, and the right **gear** tilts
all of it in your favour.

## Weapons & armor

Every hero can equip a **weapon** and a piece of **armor**, and each item has a
description you can read on the command screen. Broadly:

- **Weapons** raise **attack** or **magic** and add **crit** and **accuracy**.
- **Armor** raises **defense** and adds **evasion** (the chance to dodge a hit
  entirely).

ROLAND starts with an **IRON SWORD** and **LEATHER ARMOR**; ELARA joins in a
**TRAVELER'S ROBE**. For the full list — and how to add your own — see
**[Weapons & Armor](equipment.md)** and
**[Extending the Game](modding.md#add-equipment)**.

## Enemy behaviour

Each enemy has an AI setting:

- **Basic** — always uses its plain attack on a random hero. Slimes and gargoyles
  use this.
- **Random** — mixes in its skills (the demon uses this, so it sometimes leads with
  CLAW or a burning FIREBALL).

## Winning and losing

- **Victory** awards **XP** and **gold** to every living hero, shown on a short
  report. Enough XP levels a hero up on the spot. Clearing the *last* enemy of a
  level can also queue a story [cutscene](#story-and-cutscenes).
- **Defeat** revives the party at full health and returns you to the level to try
  again — see [If the party falls](gameplay.md#if-the-party-falls).

## Story and cutscenes

Cutscenes are scripted sequences of dialogue lines — each with an optional
speaker name and a character **portrait** — that can also **recruit** a new party
member. They fire at set moments:

- An **intro** cutscene the first time you enter a level (Greenwood opens with
  ROLAND vowing to drive the demons out).
- A **clear** cutscene the first time you defeat every enemy in a level (clearing
  Greenwood is where ELARA introduces herself and joins).

Press **Confirm** to reveal a line instantly, then again to advance. Because
cutscenes are pure data, adding your own story beats — and the allies they bring
— is just editing the data file. See **[Extending the Game](modding.md)**.
