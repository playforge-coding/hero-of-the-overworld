---
comments: true
---

# Gameplay

Hero of the Overworld is a compact turn-based JRPG. You guide a party from a
**world map** into **levels**, explore their tile screens, fight the monsters that
roam them, and grow stronger — until every level is cleared.

## The title screen

The game opens on the title. It shows your current party and their levels; press
**Confirm** to head to the [world map](world.md#the-world-map). If you have a
saved game the prompt reads **CONTINUE** and picks up where you left off;
otherwise it reads **BEGIN**.

## The core loop

1. **Choose a level.** On the world map, move the cursor to a level marker and
   **Confirm** to enter it. See **[The Overworld](world.md)**.
2. **Explore.** Walk the party leader through the level's connected
   [screens](world.md#screens), stepping between them through mid-edge openings.
3. **Fight or dodge.** Enemies roam each screen and chase you when you get close.
   Touch one and a **[battle](battles.md)** begins — but you move faster than they
   do, so you can weave around them if you'd rather not fight.
4. **Win rewards.** Victory grants **XP** and **gold** to the party, and can push
   heroes up a level (raising their stats).
5. **Clear the level.** Beat *every* demon across all of a level's screens to
   mark it **cleared** on the map. Some levels reward a clear with a
   **[cutscene](battles.md#story-and-cutscenes)** — that's how new party members
   join.
6. **Move on.** Clearing a level **unlocks the next one** on the map (progression
   is linear — see [Progression](world.md#progression-is-linear)). Return to the
   map and take it. Clear them all.

## Your party

You start as a single hero, **ROLAND** the swordsman. Party members carry their
**HP**, **MP**, **level**, and **XP** between battles, so damage taken in one
fight persists into the next until you heal (with the MEND skill, or by being
revived — see below).

New heroes join through the story. After you clear the **Greenwood**, a cutscene
introduces **ELARA**, a mage, who joins your party for good. The battle system
simply iterates whoever is in the party, so a second (or third) hero fights
alongside you with no fuss. See **[Extending the Game](modding.md)** to add your
own.

## Levelling up

Winning a battle awards XP to every living hero. Enough XP raises a hero's
**level**, which bumps their max HP and MP, attack, defense, and magic, and
tops them back up to full. Speed is deliberately left alone — enemies don't gain
speed with level either, so raising it would skew turn order. Fallen heroes earn
no XP from that fight, so
keeping everyone alive pays off.

### Enemies scale with you

So the world doesn't fall behind your growth, **roaming enemies scale to your
party's level** when a battle begins — their HP, attack, defense and magic (and
their XP/gold rewards) grow with you, so a foe that was a threat early on stays a
fair fight later instead of becoming a one-hit pushover. Two things are held
fixed by design:

- **Speed never scales**, so the turn order you learned to exploit still holds —
  lumbering [gargoyles](entities/gargoyle.md) act last, mounted
  [dark knights](entities/dark_knight.md) act first.
- At **party level 1** the scaling is the identity: the opening region fights
  every foe at exactly its authored strength.

The stat blocks in the [Bestiary](entities/index.md) list those **base** (level-1)
numbers.

## If the party falls

There's no game over. If every hero is knocked out in a battle, the party is
**revived at full health "at camp"** and you're returned to the level to try
again. Losing a fight costs you the attempt, not your progress.

## Progress and scoring

Your goal is simply to **clear every level**. The world map shows a running
**CLEARED x/y** tally, and each level marker turns from red to green with a star
once you've wiped out its demons.

## Saving

Your progress is **saved automatically** — there's nothing to manage. The game
writes a save after each battle, whenever you leave a level, and after a story
cutscene, capturing:

- your **party** — members, levels, XP, live HP/MP, equipped gear, and gold;
- which **levels are cleared** (and therefore which are unlocked);
- your **in-level progress** — the specific demons you've already beaten, so
  quitting halfway through a level doesn't undo the fights you've won;
- your **exact position** — the level, screen, and spot you were standing on, so
  a save taken mid-level puts you right back there rather than on the world map.
  (Leaving a level to the map clears this, so you resume on the map instead.)

Next launch, the title offers **CONTINUE** and drops you back into that state —
straight into the level and spot where you saved, if you were in one.
The save lives in a small custom binary file: on desktop under your OS data
directory (e.g. `~/.local/share/hero-of-the-overworld/save.bin` on Linux,
`%APPDATA%` on Windows), and in the browser's **IndexedDB** for the
[web build](getting-started.md).
