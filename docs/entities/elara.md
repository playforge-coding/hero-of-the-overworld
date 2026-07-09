---
comments: true
---

# Elara

A **wandering mage** who joins Roland once the [Greenwood](../world.md#the-world-map)
is cleared. She isn't in the starting party — she introduces herself in the
level-clear [cutscene](../battles.md#story-and-cutscenes) ("*I am Elara, a wandering
mage. Your cause is mine now.*") and fights alongside you for the rest of the run.

??? warning "Who is Elara, really? (Chapter 2 spoiler)"
    "Elara" is an **alter ego**. She is the **princess of the kingdom**, travelling
    under a plain name since the throne fell — a crown with a price on it is safer left
    unspoken. The [Captain](captain.md) recognises her on the Castaway Shore, and
    [Gareth](gareth.md) — her father's old scout — turns out to have known since
    Greenwood and kept it quiet to keep the party's focus on the mission. It's a story
    beat only: **nothing about how she plays changes.** See [The Story](../story.md).

| Stat | Value |
|---|---|
| **HP** | 78 |
| **MP** | 60 |
| **Attack** | 10 |
| **Defense** | 9 |
| **Magic** | 26 |
| **Speed** | 13 |
| **Role** | Magic damage & support |
| **Gear** | [Traveler's Robe](../equipment.md) |
| **Joins** | After clearing Greenwood (via the `mage_joins` cutscene) |

Elara is Roland's opposite: **fragile but devastating**. Her magic is the highest in
the party and her MP pool is deep, so she can throw spells all fight — but her low HP
and defense mean you don't want a demon reaching her. She's slightly faster than
Roland, so she often acts first. Pair her ranged magic with Roland's staying power and
most battles fall quickly.

## Skills

| Skill | Kind | MP | Target | Learned |
| ----- | ---- | -- | ------ | ------- |
| FIREBOLT | Magical | 6 | One enemy | Start |
| FROST | Magical | 12 | All enemies | **Level 4** |
| MEND | Heal | 10 | One ally | **Level 7** |

**FIREBOLT** is a hard single-target nuke off her high magic; at **level 4** she
learns **FROST**, which hits every demon at once (a duo or trio can be softened or
finished in one cast); and at **level 7** she masters **MEND**, a potent heal that
also **revives a downed ally** (bringing them back with the healed HP). MEND is
**hers alone** — no other hero can heal or revive by magic, so Elara is the party's
lifeline. It's also her **last-unlocked** move. See
[Battles](../battles.md#skills) and [Levelling up](../gameplay.md#levelling-up).

## Gear

Elara joins wearing a **[Traveler's Robe](../equipment.md)** — light enchanted
cloth that leans on **evasion** (and a touch of magic) rather than heavy plate, so
she dodges more than she blocks. Fitting for a wanderer who fights with spells, she
carries no weapon; her power is in her magic stat.

## Appearance

Elara has her own **purple mage sprite sheet** (`mage.png`), a 6×8 grid of 16×16
frames. The walk rows (0–3) carry her around the overworld if she leads the party,
and the cast rows (4–7) play when she acts in battle. One quirk of the sheet: the
cast rows are ordered down/up/**left/right**, swapping left and right compared to the
walk rows.
