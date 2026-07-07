---
comments: true
---

# Orc Brute

The **hulking heart of the orc family**. Where the [goblins](club_goblin.md) were
prisoners the [underworld](../story.md) broke and loosed, the orcs are the same
prisoners the change hit **hardest** — twisted whole into slabs of muscle. (In the
lore a handful were later released to the surface, but only under the army's **heavy
watch**; down here in the Charred Depths there is no such guard.) An orc brute is a
slow, punishing wall of HP that lumbers after you and hammers with a two-fisted
**BRUTE SLAM**.

| Stat | Value |
|---|---|
| **HP** | 150 |
| **MP** | 0 |
| **Attack** | 34 |
| **Defense** | 19 |
| **Magic** | 0 |
| **Speed** | 9 |
| **AI** | Random (mixes BRUTE SLAM with plain swings) |
| **Rewards** | 48 XP · 40 gold on defeat |
| **Drops** | [MIGHT TONIC](../items.md) (18%) · [HI-POTION](../items.md) (12%) |
| **Found in** | Charred Depths (the deeper chambers) |

An orc brute is the Charred Depths' anchor and the toughest regular foe in the game:
at 150 HP behind a defense of 19 it is a genuine wall, and its **BRUTE SLAM** hits
harder than anything short of a boss. The deep chambers muster them **two and three
abreast**, so a room of brutes is a war of attrition. But at speed 9 it still acts
**late** in the round and **crawls** on the map (slower than the player, like a
[gargoyle](gargoyle.md)) — so you can sidestep one you would rather not fight, and in
battle you often get a full turn in before it swings.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| BRUTE SLAM | Physical | 0 | One enemy | Free 170%-power slam — the depths' heaviest blow |

**BRUTE SLAM** costs no MP and lands hard, but it is an ordinary (non-piercing) blow,
so you can **time a block** to blunt it. Because the brute is slow, brace for the slam
and pour damage in on the turns it hasn't acted yet.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `orc_solo` | 1 orc brute | Charred Depths (the nest, the galleries) |
| `orc_duo` | 2 orc brutes | Charred Depths (the orc pit, the deep halls) |
| `orc_trio` | 3 orc brutes | Charred Depths (the deep warren, the guardian's hall) |
| `orc_goblins` | 1 orc brute + 1 [club goblin](club_goblin.md) + 1 [archer goblin](archer_goblin.md) | Charred Depths (deeper chambers) |
| `demon_orc` | 1 [demon](demon.md) + 1 orc brute | Charred Depths (the chasm walk, the bone gallery) |
| `demon_orcs` | 1 [demon](demon.md) + 1 orc brute + 1 [club goblin](club_goblin.md) | Charred Depths (the antechamber) |
| `demon_guard` | 1 [demon](demon.md) + 2 orc brutes | Charred Depths (the depths' guardian) |

## Appearance

The orc brute uses the `orc_brute` sprite sheet (a 6×8 grid of 16×16 frames, the same
size as the [demon](demon.md)'s): directional walk rows (0–3) as it lumbers across the
map, and its big attack poses (rows 4–7) plus an idle in battle.
