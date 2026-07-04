---
comments: true
---

# Shops

Dotted around the [overworld](world.md) are **shops** — little stores you step
into to spend the gold you win in [battle](battles.md) on better
[weapons and armor](equipment.md). Like everything else in the game, a shop is
pure data, so adding one is a content edit rather than a code change (see
[Extending the Game](modding.md#add-a-shop)).

## Finding a shop

A shopkeeper stands on the map wherever a shop has been placed, under a floating
**SHOP** banner. Walk up to them and a **PRESS Z** prompt appears — press
**Confirm** to step inside.

The bundled world puts an **OUTFITTER** right beside where you start in the
[GREENWOOD](world.md#the-world-map), so you can gear up before the first fight.

## Inside the store

The interior is a small **wood-floored, stone-walled** room with the keeper at a
counter. There's exactly **one doorway — on the wall the keeper faces** — so
**you leave the way the keeper is looking**: just walk into the opening to step
back out onto the map, right where you entered.

- **Move** — walk around the room.
- **Confirm** at the counter — open the **buy menu**.
- Walk **out the doorway** — leave the shop.

## Buying

At the counter, the buy menu lists the keeper's wares:

- **Up / Down** — highlight an item. Its slot, stat bonuses, and description show
  on the right; items you can't afford are greyed out.
- **Left / Right** — choose **which party member** to outfit. Their current gear
  in that slot is shown as **NOW: …**.
- **Confirm** — **buy and equip**. The price is deducted from your gold and the
  item is worn immediately, replacing whatever was in that slot.
- **Cancel** — close the menu (back to walking the room).

Stock is **unlimited** — the only limit is the gold in your purse, so the same
keeper will happily sell you a second sword. Purchases are
[saved](gameplay.md#saving) as soon as you leave.

!!! note "No resale (yet)"
    Buying an item **equips it in place** of your current gear, and there's no
    inventory to stash or sell the old piece. Spend with intent — the item you
    replace is gone. Equipment you buy is the same gear described in
    **[Weapons & Armor](equipment.md)**.
