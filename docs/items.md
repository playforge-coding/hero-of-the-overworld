---
comments: true
---

# Items

Alongside the [weapons and armor](equipment.md) you *wear*, the party carries
**items** — consumables you *spend*. A potion mends a wound, a bomb blows up a
foe, a tonic steels a hero's arm for a few rounds. Unlike gear, an item is used up
when you use it, and the whole party draws from **one shared stash**.

Like everything else in the game, items are pure data, so adding your own is a
content edit rather than a code change (see
[Extending the Game](modding.md#add-an-item)).

## Using items in battle

On a hero's turn, pick **ITEM** from the [command menu](battles.md#the-command-menu)
to open the stash. Each line shows the item's name, how many you're carrying
(**×N**), and a short summary of what it does; the panel on the left spells out its
effect and description.

- **Up / Down** — highlight an item (or **BACK**).
- **Confirm** — use it. If it needs a target (a single foe or ally) you'll pick one
  next, exactly like a [skill](battles.md#the-command-menu); otherwise it resolves
  at once.
- **Cancel** — back out to the command menu.

A few things that make items distinct from skills:

- **The effect comes from the item, not the hero.** A potion heals the same amount
  whoever drinks it — item power is **flat**, not scaled off attack or magic — and
  items **never miss**.
- **They cost no MP**, only the item itself.
- **The stash is shared.** Using the last potion spends it for everyone; a hero
  later in the round sees the reduced count.
- **Backing out is free.** An item is only spent once its use actually resolves, so
  cancelling target selection costs you nothing.

## Using items outside battle

You don't have to be in a fight to heal up. Press **Menu** in a level to open the
**[party menu](gameplay.md#inventory-and-equipment)**, pick a hero, and choose
**USE ITEM** to spend a restorative item on any ally — a potion between fights saves
a trip back to camp. Only **restorative** items (anything that **heals HP or
restores MP**) appear there; offensive items and buffs need a foe or don't persist,
so they stay battle-only.

The same menu also lets a hero cast a **healing move** they know (**USE MOVE**) on
an ally for MP — so both kinds of healing work on the map, not just mid-battle. See
**[Gameplay → the party menu](gameplay.md#inventory-and-equipment)** for the full
flow.

## The bundled items

| Item | Target | Effect | Price | Buy at | Drops from |
| ---- | ------ | ------ | ----- | ------ | ---------- |
| **POTION** | One ally | Restore **40 HP** | 30 | OUTFITTER | [Slime](entities/slime.md), [Mountain Crab](entities/mountain_crab.md) |
| **HI-POTION** | One ally | Restore **100 HP** | 90 | — | [Demon](entities/demon.md), [Dark Knight](entities/dark_knight.md), [Dragon](entities/dragon.md) |
| **ETHER** | One ally | Restore **20 MP** | 60 | OUTFITTER | [Gargoyle](entities/gargoyle.md) |
| **BOMB** | One enemy | **35** fire damage + **BURN** | 40 | OUTFITTER | [Skeleton](entities/skeleton.md) |
| **MIGHT TONIC** | One ally | **MIGHT** — +8 ATTACK for 3 rounds | 50 | OUTFITTER | [Demon](entities/demon.md), [Dark Knight](entities/dark_knight.md) |
| **GUARD SALVE** | One ally | **GUARD** — +8 DEFENSE for 3 rounds | 50 | — | [Gargoyle](entities/gargoyle.md) |

The **BOMB** shows off how an item can do more than one thing at once — it deals
damage *and* leaves a [BURN](battles.md#status-effects) ticking. **MIGHT TONIC**
and **GUARD SALVE** show the other trick: an item that "changes stats" simply
applies a [status effect](battles.md#status-effects) whose bonus lasts a few rounds
— the same mechanism that powers slow, burn, and the rest.

## Getting items

Two ways, both covered above:

- **Buy** them at a **[shop](shops.md)** — the OUTFITTER in the Greenwood stocks a
  starter selection right where you begin.
- **Win** them as **[battle drops](battles.md#winning-and-losing)** — many foes have
  a chance to leave something behind, rolled per enemy when you win. Anything that
  drops is called out on the victory report (**FOUND …**) and added to your stash.

Your item stash is [saved](gameplay.md#saving) with the rest of your party.

## Adding your own

An item is an `ItemDef` in `assets/data/game.ron`: a **target** and a composable
**effect** (`heal`, `damage`, `restore_mp`, and/or `inflicts` status ids). Sell it
at a shop or hang it on an enemy's `drops` table. See
**[Extending the Game](modding.md#add-an-item)** for the full recipe.
