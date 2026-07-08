---
comments: true
---

# Weapons & Armor

Every hero can carry one **weapon** and one piece of **armor**. Equipment folds
its bonuses into that hero's battle stats and adds combat attributes on top, so
the right gear makes a real difference in a fight. You review and change what each
hero has equipped on the **[inventory / equipment](gameplay.md#inventory-and-equipment)**
screen (press **Menu** in a level), where every item shows its bonuses.

## What gear does

Equipment tunes your **combat** stats (it doesn't change max HP or MP — that
growth comes from [levelling up](gameplay.md#levelling-up)):

- **Weapons** raise **attack** (or **magic** for casters) and add:
    - **Crit** — a higher chance for hits to [critically strike](battles.md#hit-miss-and-crit) for +50% damage.
    - **Accuracy** — a better chance for your hits to land.
- **Armor** raises **defense** and adds:
    - **Evasion** — a chance to dodge an incoming hit entirely.

See **[Battles](battles.md#hit-miss-and-crit)** for exactly how accuracy, evasion,
and crit are rolled.

## The bundled gear

| Item | Slot | Bonuses | Worn by | Description |
| ---- | ---- | ------- | ------- | ----------- |
| **IRON SWORD** | Weapon | +6 ATK, +6% crit, +4% accuracy | [Roland](entities/roland.md) | Standard-issue soldier's blade. |
| **LEATHER ARMOR** | Armor | +5 DEF, +4% evasion | [Roland](entities/roland.md) | Boiled leather; turns a glance. |
| **TRAVELER'S ROBE** | Armor | +3 DEF, +2 MAG, +8% evasion | [Elara](entities/elara.md) | Light, enchanted travel cloth. |
| **SCOUT'S EDGE** | Weapon | +7 ATK, +3 SPD, +14% crit, +10% accuracy | [Gareth](entities/gareth.md) | A keen, light back-slung blade. |
| **SCOUT'S CLOAK** | Armor | +3 DEF, +2 SPD, +14% evasion | [Gareth](entities/gareth.md) | A light, silent ranger's cloak. |
| **STONE FISTS** | Weapon | +18 ATK, +12% crit, **−34% accuracy** | [Gargoyle](entities/gargoyle.md) | Crushing stone knuckles; heavy and clumsy. |
| **CAVALRY LANCE** | Weapon | +10 ATK, +8% crit, +6% accuracy | [Dark Knight](entities/dark_knight.md) | A mounted knight's warlance; long and deadly. |

Fitting the story, ROLAND marches out in the plain starter kit of a first-month
conscript, while ELARA's robe leans on evasion and a touch of magic rather than
heavy protection. **GARETH** — the former king's scout — keeps the tools of his old
trade: a light, back-slung **SCOUT'S EDGE** and a **SCOUT'S CLOAK** that trade
armor for speed, crit, and the evasion of a man who lived by not being caught.
Equipment isn't hero-only: the **[gargoyle](entities/gargoyle.md)**
swings the brutal but wildly inaccurate **STONE FISTS**, which is exactly why it
hits like a truck yet misses so often — the accuracy penalty is a real stat, and
the **[dark knights](entities/dark_knight.md)** of Traveller's End ride into battle
behind a keen, reliable **CAVALRY LANCE**.

## Getting new gear

A hero can **start** with gear (set on their `CharacterDef`), or you can **buy**
it at a **[shop](shops.md)**. The Greenwood's OUTFITTER sells the kit above
(alongside a few **[items](items.md)**); buying a piece equips it to the hero you
choose and spends the gold you've won in battle. Whatever it replaces isn't thrown
away — it drops into the party's shared **bag**.

Gear is different from **[items](items.md)**: items are consumables you *spend* in
battle (potions, bombs, tonics), not equipment you *wear*. This page is only about
weapons and armor.

## Changing gear anywhere

You don't need a shop to shuffle equipment around. Press **Menu**
(<kbd>Shift</kbd>/<kbd>C</kbd>, or Start on a gamepad) while walking a level to open
the **[inventory / equipment](gameplay.md#inventory-and-equipment)** screen. The
party owns a shared **bag** of unequipped gear; there you equip bag items onto any
hero, unequip pieces back into the bag, or hand a weapon down from one hero to
another. Equipping **swaps** the displaced item into the bag, so nothing is ever
lost — and the whole bag rides along in your [save](gameplay.md#saving).

## Adding your own

Weapons and armor are pure data, like everything else — an `EquipmentDef` in its
own `assets/data/equipment/<id>.ron` with a slot, an icon, stat bonuses, and a description.
Characters (and enemies) equip them by id. See
**[Extending the Game](modding.md#add-equipment)**.
