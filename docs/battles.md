---
comments: true
---

# Battles

Touching a roaming demon on the [overworld](world.md#roaming-demons) drops you
into a turn-based battle against that encounter's demons. Your whole party fights
whoever is present.

## A round, step by step

1. **Command phase.** For each living hero, in speed order, you pick an action:
   **ATTACK**, a **SKILL**, or **DEFEND**. You can **Cancel** back to the previous
   hero to re-plan their choice.
2. **Enemies plan.** Each demon chooses an action via its simple AI.
3. **Resolve.** Everyone's actions are sorted by **speed** (fastest first) and
   played out one at a time, with a little lunge animation and floating damage /
   heal numbers.
4. **Check the outcome.** If all demons are down you **win**; if all heroes are
   down you **lose**. Otherwise a new round begins.

## The command menu

| Command | What it does |
| ------- | ------------ |
| **ATTACK** | A basic physical strike on one enemy you choose. Free. |
| **SKILL** | Open the hero's skill list and pick one (costs MP). Then choose a target if the skill needs one. |
| **DEFEND** | Brace: incoming damage to this hero is reduced until their next turn. |

Choosing **SKILL** shows what that hero knows and each skill's MP cost. A skill
you can't afford can't be selected. Depending on the skill you'll then choose a
**single** target or it will hit **all** valid targets at once.

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
| CLAW | Physical | 0 | One enemy | The demon's attack |

## How damage works

A hit's damage is based on the attacker's offensive stat and the skill's power,
softened by the target's defense, then nudged by a small random spread:

```
damage ≈ (offense × power / 100) − defense / 2      (at least 1)
```

- **ATTACK** uses power 100; skills use their own power value.
- A little **±12% variance** keeps hits from being identical.
- A **defending** target takes reduced damage that round.
- **MEND** and other heals restore HP up to the target's maximum.

So high **attack**/**magic** hits harder, high **defense** absorbs more, and high
**speed** acts earlier in the round.

## Enemy behaviour

Each enemy has an AI setting:

- **Basic** — always uses its plain attack on a random hero.
- **Random** — mixes in its skills (the bundled demon uses this, so it sometimes
  leads with CLAW).

## Winning and losing

- **Victory** awards **XP** and **gold** to every living hero, shown on a short
  report. Enough XP levels a hero up on the spot. Clearing the *last* demon of a
  level can also queue a story [cutscene](#story-and-cutscenes).
- **Defeat** revives the party at full health and returns you to the level to try
  again — see [If the party falls](gameplay.md#if-the-party-falls).

## Story and cutscenes

Cutscenes are scripted sequences of dialogue lines — each with an optional
speaker name and a character **portrait** — that can also **recruit** a new party
member. They fire at set moments:

- An **intro** cutscene the first time you enter a level (Greenwood opens with
  ROLAND vowing to drive the demons out).
- A **clear** cutscene the first time you defeat every demon in a level (clearing
  Greenwood is where ELARA introduces herself and joins).

Press **Confirm** to reveal a line instantly, then again to advance. Because
cutscenes are pure data, adding your own story beats — and the allies they bring
— is just editing the data file. See **[Extending the Game](modding.md)**.
