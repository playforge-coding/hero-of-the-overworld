---
comments: true
---

# Weapons & Armor

Every hero can carry one **weapon** and one piece of **armor**. Equipment folds
its bonuses into that hero's battle stats and adds combat attributes on top, so
the right gear makes a real difference in a fight. While you're choosing a
command, a panel shows the acting hero's weapon and armor — icon, bonuses, and a
short **description** of each.

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
| **STONE FISTS** | Weapon | +18 ATK, +12% crit, **−34% accuracy** | [Gargoyle](entities/gargoyle.md) | Crushing stone knuckles; heavy and clumsy. |

Fitting the story, ROLAND marches out in the plain starter kit of a first-month
conscript, while ELARA's robe leans on evasion and a touch of magic rather than
heavy protection. Equipment isn't hero-only: the **[gargoyle](entities/gargoyle.md)**
swings the brutal but wildly inaccurate **STONE FISTS**, which is exactly why it
hits like a truck yet misses so often — the accuracy penalty is a real stat.

## Adding your own

Weapons and armor are pure data, like everything else — an `EquipmentDef` in
`assets/data/game.ron` with a slot, an icon, stat bonuses, and a description.
Characters (and enemies) equip them by id. See
**[Extending the Game](modding.md#add-equipment)**.
