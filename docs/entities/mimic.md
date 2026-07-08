---
comments: true
---

# Mimic

A chest that bites back — and one of the deep world's nastiest surprises. A mimic
lies in wait on the [overworld](../world.md#chests-and-mimics) disguised as a
treasure chest, drawn from the **very same sprite** as a real one, so there is no
telling them apart from a distance. Wander inside its striking distance and the
disguise drops: it reveals a mouthful of teeth and **chases you down** to start a
fight. It haunts the **late** regions — [Traveller's End](../world.md), the
[Demon Fortress](../world.md), and the underworld's [Charred Depths](../world.md) —
where every chest is a gamble.

| Stat | Value |
|---|---|
| **HP** | 132 |
| **MP** | 40 |
| **Attack** | 26 |
| **Defense** | 14 |
| **Magic** | 24 |
| **Speed** | 14 |
| **AI** | Random (mixes FIREBALL, mimicry, and a heavy bite) |
| **Rewards** | 55 XP · 80 gold on defeat |
| **Drops** | [HI-POTION](../items.md) (50%) · [ETHER](../items.md) (40%) |
| **Found in** | Traveller's End, Demon Fortress, Charred Depths (disguised as chests) |

A mimic is a genuine threat: tanky, hard-hitting, and equipped with real tricks. On
its turn it may hurl a **[FIREBALL](../battles.md)**, sink its teeth in with a heavy
bite (its basic attack), or — its signature — **ape the party's own last move**.
Beat one, though, and the payoff is fat: a purse heavier than most late foes' and a
good shot at a **HI-POTION** or **ETHER**, the reward for gambling on a suspicious box.

## Mimicry — it copies your moves { #mimicry }

A mimic's defining trick is **mimicry**. On roughly half its turns — once the party
has actually cast something it can copy — it **takes on the shape of the hero who
last used a move** and strikes with a copy of that very skill. You'll see Roland's
own **POWER STRIKE**, Elara's **FROST**, or Gareth's **QUICK SLASH** turned back on
the party, cast by a monster wearing that hero's face.

Two limits keep it fair:

- **A safe allow-list.** A mimic can only copy a fixed subset of the party's skills —
  mid-power strikes and area moves (POWER STRIKE, WHIRLWIND, FIREBOLT, FROST, QUICK
  SLASH, SWALLOW CUT). It **cannot** parrot the big finishers (SUNDER, FINAL CUT) or
  MEND, so it can't nuke you with your own strongest hits.
- **A power nerf.** The copy lands at **65%** of the move's real power.

The copy otherwise behaves like the original — a copied FIREBALL still flies as a
fireball and can't be blocked; a copied POWER STRIKE can. If the party hasn't cast a
copyable move yet, the mimic simply falls back to its FIREBALL and bite.

This is a **data-driven** ability: a future mimic variant could widen the allow-list,
copy at full strength, or ape only spells — all as a content edit, no engine change.
See **[Extending the Game](../modding.md#add-a-chest-or-a-mimic)**.

## On the map

A dormant mimic is **indistinguishable from a real [chest](../world.md#chests-and-mimics)** —
same sprite, same footprint — so there's no reading it before it moves; the safe
habit is simply to approach any chest ready to bolt. The deep regions deliberately
place real chests and mimics side by side, so you can never be sure which cache is
which. Once it wakes it **stays awake** and pursues with no aggro cap — you can't
stroll back out of range. You are still fractionally faster, so a committed retreat
shakes it; dawdle and it lands on you. If it *does* win a fight, it retreats to its
spot and re-disguises, ready to spring again.

## Skills

**FIREBALL** — its own cast, a blazing orb that leaves the target BURNING. On top of
this it can **[mimic](#mimicry)** a copyable party skill (at reduced power), and
falls back to a plain heavy bite.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `mimic_solo` | 1 mimic | Traveller's End, Demon Fortress, Charred Depths |
| `mimic_pair` | 2 mimics | Charred Depths (the deepest hoards) |

## Appearance

The mimic uses the `mimic` sprite sheet — a 16×32 image that is a single column of
two 16×16 frames. **Row 0** is its idle/watching pose; **row 1** is the toothy
open-mouthed attack. On the **map**, a *dormant* mimic isn't drawn from its own sheet
at all — it borrows the real `chest` prop sprite so it's a perfect match for nearby
treasure, and only switches to its sheet's toothy row 1 — bobbing menacingly — the
moment it wakes and gives chase. In **battle** it uses the two sheet frames as its
idle (row 0) and attack (row 1) poses — except mid-**[mimicry](#mimicry)**, when it
wears the copied hero's sprite for the duration of the strike, then reverts.
