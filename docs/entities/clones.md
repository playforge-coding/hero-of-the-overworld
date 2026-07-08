---
comments: true
---

# The Clones

A dark ritual turns the party's own shadows against them. Each **clone** is a
hostile double of a hero — the same stats, the same battle sprite, the same gear,
and the hero's **full** skill set, to the number — but sworn to the other side and
cast in a shadowy violet tint so a mirror reads apart from the real hero across the
field. They are straight stat-clones: ordinary foes with no special engine tricks,
their whole menace being that they fight *exactly as your party does*.

They are **not placed in any level**. The three meet the party only in the
**mirror match** — one [boss](../battles.md#winning-and-losing) encounter, all three
clones at once (reachable now through the
[developer fight menu](../modding.md#the-developer-menu-f1)).

## Dark Roland { #dark-roland }

| Stat | Value |
|---|---|
| **HP** | 120 |
| **MP** | 24 |
| **Attack** | 22 |
| **Defense** | 14 |
| **Magic** | 10 |
| **Speed** | 12 |
| **Skills** | POWER STRIKE, WHIRLWIND, SUNDER |
| **Gear** | Iron Sword, Leather Armor |
| **Rewards** | 70 XP · 55 gold |
| **Drops** | [HI-POTION](../items.md) (30%) |

A mirror of [Roland](roland.md): a sturdy front-liner who opens with POWER STRIKE and,
like the real thing, can bring down a crushing **SUNDER**. The brawn of the trio.

## Dark Elara { #dark-elara }

| Stat | Value |
|---|---|
| **HP** | 78 |
| **MP** | 60 |
| **Attack** | 10 |
| **Defense** | 9 |
| **Magic** | 26 |
| **Speed** | 13 |
| **Skills** | FIREBOLT, FROST, MEND |
| **Gear** | Traveler's Robe |
| **Rewards** | 68 XP · 55 gold |
| **Drops** | [ETHER](../items.md) (30%) |

A mirror of [Elara](elara.md): a glass-cannon mage who burns the party with FIREBOLT
and chills the whole line with **FROST** — and, crucially, **MENDs her fellow clones**,
healing (and even reviving) them. Cut her down first, or the fight drags forever.

## Dark Gareth { #dark-gareth }

| Stat | Value |
|---|---|
| **HP** | 94 |
| **MP** | 30 |
| **Attack** | 20 |
| **Defense** | 11 |
| **Magic** | 8 |
| **Speed** | 21 |
| **Skills** | QUICK SLASH, SWALLOW CUT, FINAL CUT |
| **Gear** | Scout's Edge, Scout's Cloak |
| **Rewards** | 68 XP · 55 gold |
| **Drops** | [HI-POTION](../items.md) (30%) |

A mirror of [Gareth](gareth.md): far and away the **fastest** of the three, so it
acts first each round, darting in with QUICK SLASH and finishing with a lethal
**FINAL CUT**.

## The mirror match

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `mirror_match` | Dark Roland, Dark Elara, Dark Gareth | Boss fight — not yet placed in a level |

Fielding all three at once, the mirror match is a **priorities fight** in the mould of
a [ballista emplacement](ballista.md): silence **Dark Elara** before her MEND turns the
fight into attrition, weather **Dark Gareth**'s first-strike speed, and trade blows
with **Dark Roland**'s bulk. Because a clone fights just as its hero does, the counter
is to out-play your own kit — time your [blocks](../battles.md#action-timing-strikes-and-blocks)
against moves you already know by heart.

## Appearance

Each clone borrows its hero's own battle sheet — `swordsman`, `mage`, or `hermit` —
under a violet **tint** that marks it as a corrupted reflection. Since clones sit on
the enemy (right) side, their sprites are mirrored to face the party, completing the
"dark twin across the field" look. No new art: a clone is a pure-data
[enemy](../modding.md#add-an-enemy-and-an-encounter) that reuses the hero's sheet, and
its stats and gear are kept in lockstep with the party by a test.
