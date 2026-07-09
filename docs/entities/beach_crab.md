---
comments: true
---

# Beach Crab

The shore's answer to the **[mountain crab](mountain_crab.md)** — a hardier,
salt-crusted shell-back that skitters the tideline of the **[Castaway
Shore](../world.md#progression-is-linear)** in Chapter 2. Weak and slow on offense,
but its thick shell (high defense) makes it a chore to crack, just like its mountain
cousin, only tougher.

| Stat | Value |
|---|---|
| **HP** | 42 |
| **MP** | 0 |
| **Attack** | 15 |
| **Defense** | 17 |
| **Magic** | 0 |
| **Speed** | 6 |
| **AI** | Basic (always attacks) |
| **Rewards** | 14 XP · 10 gold on defeat |
| **Drops** | [POTION](../items.md) (22%) |
| **Found in** | Castaway Shore (the tideline) |

Like the mountain crab it's a **damage sponge, not a threat**: low attack and the
slowest thing on the beach, but that hard shell soaks physical blows. It scuttles
slowly on the map, so it's easy to sidestep — the danger is only ever that a **pirate**
catches you while you're busy prying one open.

## Skills

The beach crab has no skills — it only ever throws a plain **pinch** (a basic attack).

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `beachcrab_pair` | 2 beach crabs | Castaway Shore |
| `beachcrab_trio` | 3 beach crabs | Castaway Shore |

## Appearance

The beach crab uses the `beach_crab` sprite sheet — a tiny **3×1 grid** of 16×16
frames (one front-facing scuttle row), exactly like the [mountain
crab](mountain_crab.md). With only the one row, its battle "attack" clip just replays
the idle row a touch faster (a pinch), and every overworld facing reads the same
scuttle.
