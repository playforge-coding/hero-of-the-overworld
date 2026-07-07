---
comments: true
---

# Ballista

A **siege engine**, not a creature — the first of the game's **[tool
enemies](../battles.md#tool-enemies)**. Wheeled into the passes of
[Traveller's End](../world.md) and dug into the deep halls of the
[Charred Depths](../story.md), a ballista never fights on its own. It sits inert
until one of the **foes beside it** spends a turn to crank and loose a heavy
**BOLT** down the field — a shot that hits as hard as the engine is built, whoever
is working it.

| Stat | Value |
|---|---|
| **HP** | 160 |
| **MP** | 0 |
| **Attack** | 24 (drives the BOLT) |
| **Defense** | 18 |
| **Magic** | 0 |
| **Speed** | 3 (it never acts on its own) |
| **AI** | — (a **tool**; the aware foes work it) |
| **Rewards** | 30 XP · 22 gold when it falls |
| **Drops** | [HI-POTION](../items.md) (20%) |
| **Found in** | Traveller's End (the summit shelves) · Charred Depths (the deep emplacements) |

## How it fights

The ballista is a [tool enemy](../battles.md#tool-enemies), so it plays by
different rules than an ordinary foe:

- **It takes no turn.** It's skipped in the initiative order entirely.
- **Its crew fire it.** On an **aware** enemy's turn (any living, non-tool foe
  beside it), that enemy **strongly prefers** to spend the turn loosing a **BOLT**
  at one hero rather than acting normally — the crew works the engine on **most** of
  their turns while it stands, only falling back to their own attacks now and then.
- **The bolt can be blocked.** Read the shot and [time your
  block](../battles.md#action-timing-strikes-and-blocks) to blunt a hit that
  otherwise lands *very* hard.
- **It's tanky.** 160 HP behind a defense of 18 — you can destroy it directly, but
  it takes real commitment.
- **It crumbles when abandoned.** The instant its **last aware crew member falls**,
  the ballista has no one to work it and **falls apart on its own** — so cutting
  down the operators silences it just as surely as smashing the engine.

That makes an emplacement a **priorities puzzle**: kill the crew and the ballista
crumbles for free, or tank the bolts and burn the engine down first. It never
appears alone — always with the foes who work it.

## Skills

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| BOLT | Physical | 0 | One enemy | A heavy 200%-power siege shot, fired from the ballista |

**BOLT** is loosed as a flying **[projectile](../battles.md#attack-animations)** — a
bolt streaks from the engine to its mark and lands as it arrives. It's a blockable
blow, so brace for it; a well-timed block can shrug off most of the hit.

## Encounters

| Encounter | Enemies | Where |
| --------- | ------- | ----- |
| `ballista_nest` | 1 [dark knight](dark_knight.md) + 1 [skeleton](skeleton.md) + 1 ballista | Traveller's End (the summit) |
| `ballista_crew` | 2 [skeletons](skeleton.md) + 1 ballista | Traveller's End (the last climb) |
| `ballista_battery` | 1 [orc brute](orc_brute.md) + 1 [archer goblin](archer_goblin.md) + 1 ballista | Charred Depths (the orc pit) |
| `ballista_pit` | 1 [club goblin](club_goblin.md) + 1 [archer goblin](archer_goblin.md) + 1 ballista | Charred Depths (the bone gallery) |

## Appearance

The ballista uses the `ballista` sprite sheet — a single row of 16×16 frames: the
**attack** frames (columns 0–2) as it looses a bolt, then an **idle** wind
(columns 3–6) while it waits for its crew. It has no overworld walk sprite (it
never roams the map), so it never leads a roaming encounter — the foes who tow it
do.
