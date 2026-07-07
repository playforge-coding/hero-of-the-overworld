---
comments: true
---

# Club Goblin

The **goblin family's front line**. Goblins are prisoners of the war that the
[underworld](../story.md) broke and **indoctrinated into demon society**, then loosed
to swarm the **Charred Depths**. The club goblin is the numerous, close-in one: small
and quick, it rushes in and hammers with a crude **CLUB SMASH**. Alone it is little
threat — but it rarely comes alone, and an [archer goblin](archer_goblin.md) is
usually loosing arrows from behind it.

| Stat | Value |
|---|---|
| **HP** | 42 |
| **MP** | 0 |
| **Attack** | 16 |
| **Defense** | 7 |
| **Magic** | 0 |
| **Speed** | 13 |
| **AI** | Random (mixes CLUB SMASH with plain swings) |
| **Rewards** | 14 XP · 10 gold on defeat |
| **Drops** | [POTION](../items.md) (22%) |
| **Found in** | Charred Depths (in packs) |

Fast on its feet (speed 13) but thin on defense, a club goblin trades HP for tempo:
it acts early and hits harder than its bare attack suggests, then folds quickly once
you focus it. Against a **pack**, area skills like Roland's **WHIRLWIND** or Elara's
**FROST** clear the clubbers before their archers stack up damage.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| CLUB SMASH | Physical | 0 | One enemy | Free 130%-power blow — its bread and butter |

**CLUB SMASH** costs no MP, so a club goblin can throw it every round. It is an
ordinary melee blow, so you can **time a block** against it — unlike the archers'
piercing arrows.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `goblin_pair` | 1 club goblin + 1 [archer goblin](archer_goblin.md) | Charred Depths (upper halls) |
| `goblin_pack` | 2 club goblins + 1 [archer goblin](archer_goblin.md) | Charred Depths (upper halls, the nest) |
| `orc_goblins` | 1 [orc brute](orc_brute.md) + 1 club goblin + 1 [archer goblin](archer_goblin.md) | Charred Depths (deeper chambers) |
| `demon_orcs` | 1 [demon](demon.md) + 1 [orc brute](orc_brute.md) + 1 club goblin | Charred Depths (the depths' guardian) |

## Appearance

The club goblin uses the `club_goblin` sprite sheet (a 5×8 grid of 16×16 frames):
directional walk rows (0–3) as it roams the map, and attack poses (rows 4–7) plus an
idle in battle — the same layout the [demon](demon.md) sheet uses.
