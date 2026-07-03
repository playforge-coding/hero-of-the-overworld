---
comments: true
---

# Gameplay

Hero of the Overworld is a compact turn-based JRPG. You guide a party from a
**world map** into **levels**, explore their tile screens, fight the demons that
roam them, and grow stronger — until every level is cleared.

## The title screen

The game opens on the title. It shows your current party and their levels; press
**Confirm** to head to the [world map](world.md#the-world-map).

## The core loop

1. **Choose a level.** On the world map, move the cursor to a level marker and
   **Confirm** to enter it. See **[The Overworld](world.md)**.
2. **Explore.** Walk the party leader through the level's connected
   [screens](world.md#screens), stepping between them through mid-edge openings.
3. **Fight or dodge.** Demons roam each screen and chase you when you get close.
   Touch one and a **[battle](battles.md)** begins — but you move faster than they
   do, so you can weave around them if you'd rather not fight.
4. **Win rewards.** Victory grants **XP** and **gold** to the party, and can push
   heroes up a level (raising their stats).
5. **Clear the level.** Beat *every* demon across all of a level's screens to
   mark it **cleared** on the map. Some levels reward a clear with a
   **[cutscene](battles.md#story-and-cutscenes)** — that's how new party members
   join.
6. **Move on.** Return to the map and take the next level. Clear them all.

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
**level**, which bumps their max HP and MP, attack, defense, magic, and speed,
and tops them back up to full. Fallen heroes earn no XP from that fight, so
keeping everyone alive pays off.

## If the party falls

There's no game over. If every hero is knocked out in a battle, the party is
**revived at full health "at camp"** and you're returned to the level to try
again. Losing a fight costs you the attempt, not your progress.

## Progress and scoring

Your goal is simply to **clear every level**. The world map shows a running
**CLEARED x/y** tally, and each level marker turns from red to green with a star
once you've wiped out its demons.

Progress is tracked for the **current session** — which levels you've cleared and
your party's growth live in memory while the game runs. There is no save file, so
closing the game starts a fresh run next time.
