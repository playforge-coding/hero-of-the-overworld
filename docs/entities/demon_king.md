---
comments: true
---

# Demon King

The **boss of the Demon Facility** — and the first fight in the game you are
**meant to lose**. He waits on his throne at the end of the facility's long iron
descent, and when the party reaches him the battle that begins is not a contest at
all: the Demon King is **invincible**, and he ends it on his own terms.

!!! warning "An unwinnable fight — by design"
    The Demon King cannot be beaten. He is flagged **`invincible`** in
    [`assets/data/enemies/demon_king.ron`](../modding.md#a-scripted-unwinnable-boss),
    so no amount of damage can drop him below 1 HP. His party-wide **OBLIVION**
    one-shots the whole team, and he is faster than every hero, so he always acts
    first. **Losing is the story:** the wipe throws the party back to the surface
    and opens **[Chapter 2](../story.md#chapter-two-the-throne-beneath-the-world)**.
    Don't spend your potions trying to win — you can't.

| Stat | Value |
|---|---|
| **HP** | ∞ (invincible — damage can never bring him below 1) |
| **MP** | 0 (his power costs him nothing) |
| **Attack** | 45 |
| **Defense** | 30 |
| **Magic** | 60 |
| **Speed** | 40 (acts before any hero) |
| **AI** | Random (mixes OBLIVION and SOVEREIGN SMITE with plain attacks) |
| **Rewards** | — (he is never defeated, so none are ever paid) |
| **Found in** | Demon Facility (throne-room boss, end of Chapter 1) |

Everything about his stat block is theatrical rather than tactical: the huge HP is
there so his bar sits full and immovable, the towering magic makes his sweep lethal,
and the speed guarantees he moves first. You will see one or two rounds of him, at
most, before the party falls — and that is exactly as intended.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| OBLIVION | Magical | 0 | All enemies | 900% power, unblockable — a single sweep that fells the whole party |
| SOVEREIGN SMITE | Magical | 0 | One enemy | 900% power, unblockable — the same annihilating force, focused on one soul |

Both moves are **free** (no MP) and **unblockable** (no timed-block window), so
there is no defensive play that saves you. **OBLIVION** is the signature: it hits
every hero at once for far more than any of them can survive.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `demon_king` | 1 demon king (`boss`) | The throne hall at the end of the Demon Facility |

The `demon_king` encounter is flagged **`boss`** (so the fight plays the boss
theme) and carries two extra hooks that make its defeat a story beat rather than a
game-over: **`defeat_advances_chapter`** and a **`defeat_cutscene`**. When the party
is wiped, the game plays the launch cutscene and ticks the party into the next
**[chapter](../gameplay.md#chapters)** — see
**[Extending the Game](../modding.md#a-scripted-unwinnable-boss)** for how the whole
mechanism is wired from pure data.

## Appearance

The Demon King uses the `demon_king` sprite sheet, drawn on the **demon family's
6×8 layout** of 16×16 frames — the same arrangement as the [demon](demon.md) and
[demon elite](demon_elite.md): directional walk rows (0–3) as he stirs on the
overworld, and an idle plus attack row (row 4) for battle, where he is drawn larger
than any common foe to loom over the party.
