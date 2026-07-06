---
comments: true
---

# Dragon

The **boss of the Demon Fortress** — a hulking, fire-breathing wyrm coiled in the
fortress depths. It is the sternest fight in the game so far: a huge pool of HP,
a party-wide fire breath, and a bone-crushing tail. It prowls the deepest screen
of the fortress and, like every roaming foe, starts a [battle](../battles.md) on
contact — but this one swaps the usual battle track for a dedicated **boss theme**.

| Stat | Value |
|---|---|
| **HP** | 340 |
| **MP** | 80 |
| **Attack** | 26 |
| **Defense** | 17 |
| **Magic** | 24 |
| **Speed** | 11 |
| **AI** | Random (mixes its skills with plain attacks) |
| **Rewards** | 150 XP · 120 gold on defeat |
| **Drops** | [HI-POTION](../items.md) (guaranteed) |
| **Found in** | Demon Fortress (a single boss, deep in the last screen) |

With five times a demon's HP, the dragon is a war of attrition. Its **Random** AI
rotates its two skills with basic attacks, so you rarely get a quiet round. Bring
healing MP and be ready to spread damage: the fight rewards patience over a rush.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| FLAME BREATH | Magical | 10 | All enemies | 135% power to the whole party; leaves everyone **[BURNING](../battles.md#status-effects)** for 3 rounds |
| TAIL SWIPE | Physical | 0 | One enemy | Free 160%-power hit — the heaviest single blow in the game |

**FLAME BREATH** is the fight's defining threat: it hits your whole party *and*
stacks **BURN** on everyone, ticking 8 damage a round each, so a slow fight bleeds
the team dry. **TAIL SWIPE** costs no MP, so the dragon always has a big hit in
reserve. Keep everyone topped up with **MEND**, and clear burns before they pile on.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `dragon_boss` | 1 dragon (`boss`) | Guarding the depths of the Demon Fortress |

The `dragon_boss` encounter is flagged **`boss`** in
[`assets/data/game.ron`](../modding.md), which is what makes its battle play the
boss theme instead of the normal one. On the overworld the dragon roams at a
steady pace — you *can* slip past it, but the fortress is only cleared once every
foe, the dragon included, is beaten.

## Appearance

The dragon uses the `dragon` sprite sheet (a 4×6 grid of 32×32 frames — larger
than the other foes): front-facing poses in rows 0–1, drawn big in battle as its
idle and attack, and a side-on serpent in rows 2–3. Its overworld walk reads rows
0–3 as the down/up/right/left facings.
