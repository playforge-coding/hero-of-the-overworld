---
comments: true
---

# Gargoyle

The stone sentinels of the **Stone Pass**. A gargoyle hits like a landslide but is
**ponderously slow** and swings so clumsily it misses often — a study in trade-offs.

| Stat | Value |
|---|---|
| **HP** | 64 |
| **MP** | 0 |
| **Attack** | 20 (before its weapon) |
| **Defense** | 14 |
| **Magic** | 0 |
| **Speed** | 3 |
| **AI** | Basic (always a plain attack on a random hero) |
| **Gear** | [Stone Fists](../equipment.md) (+18 ATK, +12 crit, **−34 accuracy**) |
| **Rewards** | 18 XP · 14 gold on defeat |
| **Drops** | [ETHER](../items.md) (20%) · [GUARD SALVE](../items.md) (10%) |
| **Found in** | Stone Pass |

A gargoyle is a glass anvil in reverse: **tanky and hard-hitting** (its **Stone
Fists** push effective attack to a crushing 38, and can crit) but **slow and
inaccurate**. Speed 3 means it almost always acts **last** in a round — you'll get
your blows in first — and the Stone Fists' huge **−34 accuracy** penalty makes it
whiff constantly. Bank on it missing, and punish the openings.

On the overworld it's just as sluggish: its roaming speed is far below yours, so a
gargoyle is trivial to **outrun and dodge** if you'd rather skip the fight.

## Skills

None. A gargoyle only ever uses its (very heavy) basic attack.

## Encounters

Gargoyles patrol the Stone Pass alone or in pairs:

| Encounter | Gargoyles |
| --------- | --------- |
| `gargoyle_solo` | 1 |
| `gargoyle_duo` | 2 |

Because they're so slow and inaccurate, even a duo is more a war of attrition than a
real threat — keep everyone healthy and out-tempo them.

## Appearance

The gargoyle reuses the **demon's** sheet layout (`gargoyle.png`, a 6×8 grid of 16×16
frames): walk rows 0–3 for roaming, attack poses in rows 4–7, and an idle in battle.
