---
comments: true
---

# Mountain Crab

The skittering, shell-backed fodder of **Traveller's End**. A mountain crab is
weak and slow, but its hard shell (a stubbornly high **defense**) makes it a chore
to crack — the low-slope answer to the Greenwood's slimes.

| Stat | Value |
|---|---|
| **HP** | 30 |
| **MP** | 0 |
| **Attack** | 11 |
| **Defense** | 13 |
| **Magic** | 0 |
| **Speed** | 5 |
| **AI** | Basic (always a plain attack on a random hero) |
| **Rewards** | 6 XP · 5 gold on defeat |
| **Found in** | Traveller's End |

Where a [slime](slime.md) is a glass cannon in reverse — soft and quick to pop — a
crab is the opposite: it barely dents you (attack 11) but shrugs off blows behind
its shell. Bring a hard-hitting strike or a spell rather than chipping at it, and
don't let a whole pack pin you down.

On the overworld it scuttles slowly, well below your pace, so a crab is easy to
**sidestep** if you'd rather not bother.

## Skills

None. A mountain crab only ever uses its basic attack — a pinch of the claws.

## Encounters

Crabs turn up in twos and threes on the lower slopes:

| Encounter | Crabs |
| --------- | ----- |
| `crab_pair` | 2 |
| `crab_trio` | 3 |

## Appearance

The crab uses a tiny **3×1** sheet (`mountain_crab.png`, three 16×16 frames in a
single front-facing row). With no directional or attack rows, its battle "attack"
just replays that idle row a touch faster — a quick scuttling pinch — and every
overworld facing reads the same row.
