---
comments: true
---

# Shops

Dotted around the [overworld](world.md) are **shops** — little stores you step
into to spend the gold you win in [battle](battles.md) on better
[weapons and armor](equipment.md) and on [items](items.md). Like everything else
in the game, a shop is pure data, so adding one is a content edit rather than a
code change (see [Extending the Game](modding.md#add-a-shop)).

## Finding a shop

A shopkeeper stands on the map wherever a shop has been placed, under a floating
**SHOP** banner. Walk up to them and a **PRESS Z** prompt appears — press
**Confirm** to step inside.

The bundled world puts an **OUTFITTER** right beside where you start in the
[GREENWOOD](world.md#the-world-map), so you can gear up — and stock a few
[potions and a bomb](items.md) — before the first fight.

## Inside the store

The interior is a small **wood-floored, stone-walled** room with the keeper at a
counter. There's exactly **one doorway — on the wall the keeper faces** — so
**you leave the way the keeper is looking**: just walk into the opening to step
back out onto the map, right where you entered.

- **Move** — walk around the room.
- **Confirm** at the counter — open the **buy menu**.
- Walk **out the doorway** — leave the shop.

## Buying

At the counter, the buy menu lists the keeper's wares — both **gear** and
**[items](items.md)** sit on the same counter:

- **Up / Down** — highlight a ware. Its type (**WEAPON** / **ARMOR** / **ITEM**),
  stat bonuses or effect, and description show on the right; wares you can't afford
  are greyed out.
- **Left / Right** — for **gear**, choose **which party member** to outfit (their
  current piece in that slot shows as **NOW: …**). An **item** ignores this — it
  just goes into your shared stash (**ADDED TO YOUR ITEMS**).
- **Confirm** — **buy**. The price is deducted from your gold. Gear is **worn
  immediately**, and whatever it replaces drops into your party's shared
  **[bag](gameplay.md#inventory-and-equipment)** rather than being lost; an item is
  **added to your [item stash](items.md)** (stacking if you already hold some).
- **Cancel** — close the menu (back to walking the room).

Stock is **unlimited** — the only limit is the gold in your purse, so the same
keeper will happily sell you a second sword or a fistful of potions. Purchases are
[saved](gameplay.md#saving) as soon as you leave.

!!! note "Old gear is kept"
    Buying an item equips it and stows the piece it replaces in the party's shared
    **[item bag](gameplay.md#inventory-and-equipment)** — so you can re-equip it, or
    hand it to another hero, from the **Menu** inventory screen at any time. (There's
    no selling yet, so gold only flows one way.) Equipment you buy is the same gear
    described in **[Weapons & Armor](equipment.md)**.
