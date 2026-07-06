---
comments: true
---

# Battles

Touching a roaming enemy on the [overworld](world.md#roaming-enemies) drops you
into a turn-based battle against that encounter's foes — [slimes](entities/slime.md),
[gargoyles](entities/gargoyle.md), or [demons](entities/demon.md). Your whole party
fights whoever is present.

## A round, step by step

Every living unit — heroes and enemies alike — takes a turn in **initiative
order**, and each turn resolves **immediately** before the next begins.
Initiative is a unit's **speed** with a **dash of randomness** mixed in, so the
swift usually act first but the exact order shifts round to round and near-equal
units trade the lead. A unit that's **well above the average speed** of everyone
in the fight also has a **chance at a second turn** that round — the further
above average, the better the odds — so a nimble hero can sometimes strike twice
while slow foes plod. There's no separate planning phase: you command a hero the
moment it's their turn, and the action plays out right away.

1. **Take turns.** Stepping down the initiative order:
   - On a **hero's** turn you pick **ATTACK**, a **SKILL**, or **DEFEND** (and a
     target if needed). It resolves at once — a little lunge animation and
     floating damage / heal numbers — before play moves on.
   - On an **enemy's** turn it chooses an action via its simple AI and acts
     immediately.
   Because turns resolve as they come, a fast enemy can strike between your
   heroes, and you command each hero with the up-to-the-moment state in view.
2. **End of round.** Once everyone has acted, lingering **status effects** (burn,
   regen, …) tick.
3. **Check the outcome.** If all enemies are down you **win**; if all heroes are
   down you **lose**. Otherwise the next round begins.

A committed turn can't be taken back — once you confirm a hero's action it
happens, so there's no cancelling back to a hero who has already moved.

## The command menu

| Command | What it does |
| ------- | ------------ |
| **ATTACK** | A basic physical strike on one enemy you choose. Free. |
| **SKILL** | Open the hero's skill list and pick one (costs MP). Then choose a target if the skill needs one. |
| **ITEM** | Open the party's [item](items.md) stash and use a consumable — a shared pool spent from your inventory. Then choose a target if it needs one. |
| **DEFEND** | Brace: incoming damage to this hero is reduced until their next turn. |

Choosing **SKILL** shows what that hero knows, each skill's MP cost, and a short
**description** of the highlighted skill. A skill you can't afford can't be
selected. Depending on the skill you'll then choose a **single** target or it will
hit **all** valid targets at once. While you're picking a command, a panel also
shows the acting hero's equipped **[weapon and armor](#weapons-armor)**.

Choosing **ITEM** lists the consumables the party is carrying, each with its
count (**×N**) and a summary of what it does — heal, damage, restore MP, or grant
a buff. Using one **spends it from the shared stash**, so it's gone from every
hero afterward; an item's effect and target come from the item, not the hero, and
it costs no MP. See **[Items](items.md)** for the full list and how to get more.

## Skills

Every skill is one of three kinds and hits a chosen target set:

- **Physical** — scales off the user's **attack**.
- **Magical** — scales off the user's **magic**.
- **Heal** — restores HP to an ally, scaling off **magic**. A hero's **Heal** skills
  can *also* be cast **outside battle**, from the
  [party menu](gameplay.md#inventory-and-equipment) — handy for topping everyone up
  between fights.

The bundled skills:

| Skill | Kind | MP | Target | Notes |
| ----- | ---- | -- | ------ | ----- |
| POWER STRIKE | Physical | 4 | One enemy | Heavy single-target hit |
| WHIRLWIND | Physical | 8 | All enemies | Sweep the whole enemy line |
| MEND | Heal | 6 | One ally | Patch up a wounded hero |
| FIREBOLT | Magical | 6 | One enemy | ELARA's hard-hitting bolt |
| FROST | Magical | 12 | All enemies | Chills every enemy at once |
| CLAW | Physical | 0 | One enemy | The demon's free claw swipe |
| FIREBALL | Magical | 6 | One enemy | The demon's spell — inflicts **BURN** (see below) |
| REAP | Physical | 0 | One enemy | A [skeleton's](entities/skeleton.md) rusted blade |
| LANCE CHARGE | Physical | 6 | One enemy | A [dark knight's](entities/dark_knight.md) couched-lance charge |
| TRAMPLE | Physical | 8 | All enemies | A dark knight's warhorse tramples the party |
| FLAME BREATH | Magical | 10 | All enemies | The [dragon's](entities/dragon.md) gout of fire — inflicts **BURN** |
| TAIL SWIPE | Physical | 0 | One enemy | The dragon's crushing tail sweep |

(POWER STRIKE / WHIRLWIND / MEND are Roland's; FIREBOLT / FROST / MEND are Elara's;
CLAW and FIREBALL belong to the [demon](entities/demon.md); REAP to the
[skeleton](entities/skeleton.md); LANCE CHARGE and TRAMPLE to the mounted
[dark knight](entities/dark_knight.md); and FLAME BREATH and TAIL SWIPE to the
[dragon](entities/dragon.md). Slimes, crabs and gargoyles have no skills — they only
ever use a basic attack.)

A hero doesn't necessarily start with every skill above: some are **unlocked by
[levelling up](gameplay.md#levelling-up)**. Roland learns WHIRLWIND at level 3 and
Elara learns FROST at level 4, for instance — the move joins their command menu the
moment they earn it (and the victory report announces it). Each hero page lists
which of their skills are learned and when.

## Status effects

Some skills leave a lingering **status** on their target that ticks at the end of
each round until it wears off:

| Status | Applied by | Effect |
| ------ | ---------- | ------ |
| **BURN** | Demon's **FIREBALL** | Deals 8 damage at the end of each round for 3 rounds |

Statuses are pure data too — a new condition (poison, regen, slow, …) is just an
entry in the data file referenced from a skill, with no engine change. See
**[Extending the Game](modding.md)**.

## Hit, miss, and crit

Before an attack deals damage, it has to **land** — and it might land **hard**.

- **Miss.** Every offensive hit rolls against an accuracy check. The chance to
  land starts around **92%**, rises with the attacker's **accuracy** (from
  weapons) and **speed**, and falls with the target's **evasion** (from armor) and
  speed. It's clamped so nothing is ever a sure thing or hopeless. A miss shows a
  grey **MISS** and deals no damage.
- **Critical hit.** A landed hit has a small chance (about **5%**, higher with a
  weapon's **crit** bonus) to **crit**, dealing **+50% damage**. Crits flash
  brighter and show their number in orange with an exclamation mark.

Heals always land and never crit.

## Action timing: strikes and blocks

Damage isn't settled the instant a blow is thrown — you have a hand in it, the
way [Super Mario RPG](https://www.mariowiki.com/Super_Mario_RPG:_Legend_of_the_Seven_Stars)
does it. There's **no bar and no indicator**: you have to **read the animation**
and tap **Confirm** at the moment the blow *connects*. Learn the rhythm of each
swing — an early panic-tap simply whiffs, and **mashing the button earns
nothing**. You get exactly **one clean tap**; hammering the button forfeits the
bonus entirely, so it really is about timing.

- **Attacking.** Whenever one of your heroes deals damage — a basic **ATTACK** or
  any offensive **skill** — tap as your weapon lands and your blow hits harder:
    - **on time** (right on the connect) → **+100% damage** (double), shown
      as **PERFECT**;
    - **a hair off** → **+50% damage**, shown as **GOOD**;
    - **too early/late**, or don't tap → normal damage.
- **Blocking.** When an enemy throws a **blockable** blow at a hero, tap as it
  lands and you shrug off part of the hit:
    - **on time** → the whole hit is **subtracted** (a perfect block deals **0**,
      shown as **BLOCK**);
    - **a hair off** → **half** the damage is subtracted;
    - **too early/late**, or don't tap → you take it in full.

The window is tight and centred on the exact moment the animation connects, so
timing is a real skill — no meter to watch, just the swing. A well-timed block
stacks on top of **DEFEND**, and the bonuses ride along with crits and everything
else in the [damage formula](#how-damage-works).

!!! tip "Some heroes are easier to time"
    The timing window isn't the same for everyone. Most heroes share the default,
    tight window, but a character can be given a **more forgiving one** — the
    scout **[Gareth](entities/gareth.md)**, with his quick reflexes, lands GOOD and
    PERFECT much more easily than Roland does, on **both his attacks and his
    blocks**. It's a per-character setting (`timing` on a `CharacterDef`), so your
    own heroes can tune it too — see **[Extending the Game](modding.md)**.

### Taunting a foe

There's a **second** tap woven into a hero's basic **ATTACK**. After your blow
lands — as the hero pulls back out of the lunge — a brief window opens for a
follow-up tap on **Confirm**. Land it and you **taunt** the foe you just struck: it
will **prefer to attack that hero on its next turn** instead of picking a target at
random, and a pink **TAUNT!** flashes over it to confirm.

It's the same read-the-animation skill as the damage tap, just later in the swing:
tap once on the connect for extra damage, then again on the recovery to taunt. An
early panic-tap misses, and the taunt window sits well clear of the damage one, so
the two never collide. Use it to **pull a dangerous enemy onto your sturdiest
hero** — bait a hit onto whoever can best soak or [block](#action-timing-strikes-and-blocks)
it. A taunt lasts until the enemy takes that next turn, then wears off. (Only the
basic ATTACK taunts — skills don't.)

### Unblockable attacks

Some enemy attacks are **unblockable** — no block window appears and you take the
hit (you can still **DEFEND** to soften it the ordinary way). Piercing and magical
blows declare themselves unblockable: **FIREBALL**, **FLAME BREATH**, **FROST**,
**FIREBOLT**, and the piercing **LANCE CHARGE**. A plain melee swing, **CLAW**,
**REAP**, **TAIL SWIPE**, or the warhorse's **TRAMPLE** can all be blocked — brace
for the stomp as it lands. A skill opts in with a single `unblockable: true` data
flag — see **[Extending the Game](modding.md)**.

## How damage works

A landed hit's damage is based on the attacker's offensive stat and the skill's
power, softened by the target's defense, boosted if it crit, then nudged by a
small random spread:

```
damage ≈ (offense × power / 100) − defense / 2      (at least 1)
        × 1.5 on a critical hit
        × 0.88–1.12 random variance
        × 0.66 if the target is defending
        × 1.5 / 2.0 on a good / perfect attack-timing tap
        × 0.5 / 0.0 on a late / perfect block   (blocks can reach 0 damage)
```

- **ATTACK** uses power 100; skills use their own power value.
- **offense** is **attack** for physical hits and **magic** for magical ones —
  both raised by your **[equipped weapon](#weapons-armor)**.
- **defense** is raised by your **[equipped armor](#weapons-armor)**.
- **MEND** and other heals restore HP up to the target's maximum.

So high **attack**/**magic** hits harder, high **defense** absorbs more, high
**speed** acts earlier *and* helps you hit and dodge, and the right **gear** tilts
all of it in your favour.

## Weapons & armor

Every hero can equip a **weapon** and a piece of **armor**, and each item has a
description you can read on the command screen. Broadly:

- **Weapons** raise **attack** or **magic** and add **crit** and **accuracy**.
- **Armor** raises **defense** and adds **evasion** (the chance to dodge a hit
  entirely).

ROLAND starts with an **IRON SWORD** and **LEATHER ARMOR**; ELARA joins in a
**TRAVELER'S ROBE**. For the full list — and how to add your own — see
**[Weapons & Armor](equipment.md)** and
**[Extending the Game](modding.md#add-equipment)**.

## Enemy behaviour

Each enemy has an AI setting:

- **Basic** — always uses its plain attack on a random hero. Slimes and gargoyles
  use this.
- **Random** — mixes in its skills (the demon uses this, so it sometimes leads with
  CLAW or a burning FIREBALL).

## Winning and losing

- **Victory** awards **XP** and **gold** to every living hero, shown on a short
  report. Enough XP levels a hero up on the spot. Some foes also **drop an
  [item](items.md)** — each defeated enemy rolls its own drop chance, and anything
  that drops is listed on the report (**FOUND …**) and added to your stash.
  Clearing the *last* enemy of a level can also queue a story
  [cutscene](#story-and-cutscenes).
- **Defeat** revives the party at full health and returns you to the level to try
  again — see [If the party falls](gameplay.md#if-the-party-falls).

## Story and cutscenes

Cutscenes are scripted sequences of dialogue lines — each with an optional
speaker name and a character **portrait** — that can also **recruit** a new party
member. They fire at set moments:

- An **intro** cutscene the first time you enter a level (Greenwood opens with
  ROLAND vowing to drive the demons out).
- A **clear** cutscene the first time you defeat every enemy in a level (clearing
  Greenwood is where ELARA introduces herself and joins).

Press **Confirm** to reveal a line instantly, then again to advance. Because
cutscenes are pure data, adding your own story beats — and the allies they bring
— is just editing the data file. See **[Extending the Game](modding.md)**.
