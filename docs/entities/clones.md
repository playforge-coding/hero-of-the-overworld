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

There is a clone for **every recruitable hero** — five in all, one per character.
They are **not placed in any level**; they meet the party only in the
**mirror match** — one [boss](../battles.md#winning-and-losing) encounter
(reachable now through the [developer fight menu](../modding.md#the-developer-menu-f1)).
Crucially, the mirror match reflects **exactly the party you bring**: each clone
takes the field only when the hero it doubles is in the **active line-up**, so a hero
left in reserve brings no shadow to the fight — and, since the line-up seats at most
three, you never face more than three doubles at once.

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
like the real thing, can bring down a crushing **SUNDER**.

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

A mirror of [Gareth](gareth.md): far and away the **fastest** clone, so it
acts first each round, darting in with QUICK SLASH and finishing with a lethal
**FINAL CUT**.

## Dark Brenn { #dark-brenn }

| Stat | Value |
|---|---|
| **HP** | 138 |
| **MP** | 18 |
| **Attack** | 26 |
| **Defense** | 16 |
| **Magic** | 4 |
| **Speed** | 9 |
| **Skills** | AXE THROW, AXE BOOMERANG, CLEAVE |
| **Gear** | Battle Axe, Leather Armor |
| **Rewards** | 74 XP · 60 gold |
| **Drops** | [HI-POTION](../items.md) (30%) |

A mirror of [Brenn](../world.md#the-world-map), the garrison axeman: the **heavy** of
the doubles — the most HP and a crushing swing, but the slowest to act, so it lands its
line-clearing **AXE BOOMERANG** and brutal CLEAVE late in the round.

## Dark Captain { #dark-captain }

| Stat | Value |
|---|---|
| **HP** | 104 |
| **MP** | 20 |
| **Attack** | 24 |
| **Defense** | 15 |
| **Magic** | 6 |
| **Speed** | 14 |
| **Skills** | MUSKET SHOT, BROADSIDE, ALL HANDS |
| **Gear** | *none* |
| **Rewards** | 74 XP · 60 gold |
| **Drops** | [HI-POTION](../items.md) (30%) |

A mirror of the recruited [Captain](captain.md) — **not** the weaker
[Pirate Captain](captain.md#the-boss-fight) boss he was beaten as, but the stronger
officer he becomes, carrying that hero's stats to the number. He opens with the
long-reach MUSKET SHOT and, at the height of the fight, calls up his whole crew with
**ALL HANDS** — the most dangerous kit of any clone.

## The mirror match

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `mirror_match` | A dark double of each **active** party member | Boss fight — not yet placed in a level |

The encounter lists all five clones, but the match is **conditional on your active
line-up**: it fields one clone for every active party member and none for a hero in
reserve, so the exact fight you get depends on the party you bring (at most three
doubles, the size of the line-up). Each clone carries a `mirrors` target — the `def_id`
of the hero it doubles — and is filtered in when a battle begins if that hero is active.

Whichever doubles you draw, it is a **priorities fight** in the mould of a
[ballista emplacement](ballista.md): silence **Dark Elara** before her MEND turns the
fight into attrition, weather **Dark Gareth**'s first-strike speed, blunt **Dark
Captain**'s ALL HANDS volley, and trade blows with the bulk of **Dark Roland** or
**Dark Brenn**. Because a clone fights just as its hero does, the counter is to
out-play your own kit — time your [blocks](../battles.md#action-timing-strikes-and-blocks)
against moves you already know by heart.

## Appearance

Each clone borrows its hero's own battle sheet — `swordsman`, `mage`, `hermit`,
`axeman`, or `captain` — under a violet **tint** that marks it as a corrupted reflection. Since clones sit on
the enemy (right) side, their sprites are mirrored to face the party, completing the
"dark twin across the field" look. No new art: a clone is a pure-data
[enemy](../modding.md#add-an-enemy-and-an-encounter) that reuses the hero's sheet, and
its stats and gear are kept in lockstep with the party by a test.
