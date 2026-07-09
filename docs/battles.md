---
comments: true
---

# Battles

Touching a roaming enemy on the [overworld](world.md#roaming-enemies) drops you
into a turn-based battle against that encounter's foes — [slimes](entities/slime.md),
[gargoyles](entities/gargoyle.md), or [demons](entities/demon.md). Your active
line-up — up to **[three heroes](gameplay.md#the-battle-line-up)** — fights whoever is
present; once the party runs deeper than three, you pick which three take the field
from the party menu.

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
| SUNDER | Physical | 12 | One enemy | ROLAND's ultimate — a crushing cleave |
| FIREBOLT | Magical | 6 | One enemy | ELARA's hard-hitting bolt |
| FROST | Magical | 12 | All enemies | Chills every enemy at once |
| MEND | Heal | 10 | One ally | ELARA's potent heal — **hers alone**, and **revives** the downed |
| QUICK SLASH | Physical | 3 | One enemy | GARETH's cheap, spammable strike |
| SWALLOW CUT | Physical | 7 | All enemies | GARETH's darting sweep |
| FINAL CUT | Physical | 10 | One enemy | GARETH's ultimate — the party's hardest hit |
| CLAW | Physical | 0 | One enemy | The demon's free claw swipe |
| FIREBALL | Magical | 6 | One enemy | The demon's spell — inflicts **BURN** (see below) |
| REAP | Physical | 0 | One enemy | A [skeleton's](entities/skeleton.md) rusted blade |
| LANCE CHARGE | Physical | 6 | One enemy | A [dark knight's](entities/dark_knight.md) couched-lance charge |
| TRAMPLE | Physical | 8 | All enemies | A dark knight's warhorse tramples the party |
| FLAME BREATH | Magical | 10 | All enemies | The [dragon's](entities/dragon.md) gout of fire — inflicts **BURN** |
| TAIL SWIPE | Physical | 0 | One enemy | The dragon's crushing tail sweep |
| BOLT | Physical | 0 | One enemy | The [ballista's](entities/ballista.md) heavy siege shot — fired by the foes working it |

(POWER STRIKE / WHIRLWIND / SUNDER are Roland's; FIREBOLT / FROST / MEND are Elara's;
QUICK SLASH / SWALLOW CUT / FINAL CUT are Gareth's; CLAW and FIREBALL belong to the
[demon](entities/demon.md); REAP to the [skeleton](entities/skeleton.md); LANCE
CHARGE and TRAMPLE to the mounted [dark knight](entities/dark_knight.md); and FLAME
BREATH and TAIL SWIPE to the [dragon](entities/dragon.md). Slimes, crabs and
gargoyles have no skills — they only ever use a basic attack.) **MEND is Elara's
alone** — she is the party's only source of healing magic, so between fights the
others lean on [items](items.md).

A hero doesn't start with every skill above: most are **unlocked by
[levelling up](gameplay.md#levelling-up)**. Roland learns WHIRLWIND at level 3 and
Elara FROST at level 4, for instance, and each hero's **most powerful move is their
last** — Roland's SUNDER, Elara's MEND, and Gareth's FINAL CUT all unlock at level
7. The move joins their command menu the moment they earn it (and the victory report
announces it). Each hero page lists which of their skills are learned and when.

## Attack animations

Skills don't all swing the same way. Each one has an **attack animation** that
gives it a signature motion (it's purely cosmetic — damage, targets and
[timing](#action-timing-strikes-and-blocks) are unchanged):

| Animation | Looks like | Used by |
| --------- | ---------- | ------- |
| **Lunge** (default) | The attacker steps in, strikes, and steps back. | Most melee skills and the basic attack. |
| **Projectile** | A sprite (a fireball) flies from the caster to the target, and the blow lands as it arrives. An all-targets skill fans one out at **every** foe (or hero) at once. | ELARA's **FIREBOLT**, the demon's **FIREBALL**, and the dragon's **FLAME BREATH** (a fireball at each party member). |
| **Charge** | The attacker dashes clear across the battlefield through its target(s), off the edge, **wrapping around the screen** and back to its post — striking as it sweeps past. | The dark knight's **LANCE CHARGE** and **TRAMPLE**, and GARETH's darting **SWALLOW CUT**. |
| **Crowd** | The attacker holds its post and calls in a **swarm of allies** who flood the whole screen for a beat and then clear out, the blow landing at the crowded peak. | The [Captain](entities/captain.md)'s finale **ALL HANDS** — his whole pirate crew boils up over the rail onto every foe at once. |

Like everything else, an animation is a data choice on the skill — so giving a new
move its own projectile, charge, or crowd is a content edit, no engine change. See
**[Extending the Game](modding.md#add-a-skill)**.

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
- **MEND** and other heals restore HP up to the target's maximum. **MEND also
  revives** — cast on a **downed** ally it brings them back up (with the healed HP).

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

## Tool enemies

Some foes aren't fighters at all but **tools** — inert battlefield engines like the
**[ballista](entities/ballista.md)** — that only ever appear **alongside other
enemies**. A tool:

- **never takes its own turn.** Instead, the **aware** enemies fighting beside it
  (any living, non-tool foe) can spend *their* turn to **work it**, loosing its
  attack at your party. The ballista's **BOLT** hits hard and fires **from the
  ballista itself**, so the engine's own stats drive the damage no matter who cranks
  it.
- **can be attacked directly** like any enemy — but it's built to weather fire, so
  destroying it takes real commitment.
- **crumbles the moment no aware enemy is left to work it.** Cut down its whole crew
  and the abandoned engine falls apart on its own — you don't have to grind through
  its HP if you'd rather kill the operators.

So a ballista emplacement is a **priorities puzzle**: race to silence the crew, tank
and [block](#action-timing-strikes-and-blocks) the bolts while you burn the engine
down, or split the difference. A tool is pure data (a `tool` field on an
[enemy](modding.md#add-an-enemy-and-an-encounter)), so new siege engines are a
content edit — see **[Extending the Game](modding.md)**.

## Mimics { #mimics }

The **[mimic](entities/mimic.md)** — the deep world's chest-shaped ambusher — brings
a trick of its own into the fight: **mimicry**. On some of its turns it **copies the
last skill a party member used**, taking on that hero's very sprite as it strikes with
a copy of their move. You'll see a monster wearing Roland's face swing his own **POWER
STRIKE**, or Elara's shape hurl her **FROST** — back at the party.

It's kept fair, so it never becomes "the party's own kit, but worse to face":

- It can only copy a **safe subset** of your skills (mid-power strikes and area moves),
  never the big finishers (**SUNDER**, **FINAL CUT**) or **MEND**.
- The copy lands at **reduced power** (65%).
- The copy otherwise plays by the normal rules — a copied **FIREBALL** still can't be
  [blocked](#action-timing-strikes-and-blocks); a copied **POWER STRIKE** still can.

If the party hasn't cast a copyable move yet, the mimic just uses its own **FIREBALL**
and a heavy bite. Like everything else here, mimicry is pure data (a `mimicry` field on
an [enemy](modding.md#make-it-a-mimic)), so new mimic variants are a content edit — see
**[Extending the Game](modding.md#make-it-a-mimic)**.

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
