---
comments: true
---

# Roland

The young knight you start as — one month into mandatory service when the demons
invade. Roland is the party leader: the hero who walks the [overworld](../world.md),
and the sturdy front-liner your party is built around. His [story](../story.md) is
the game's story.

| Stat | Value |
|---|---|
| **HP** | 120 |
| **MP** | 24 |
| **Attack** | 22 |
| **Defense** | 14 |
| **Magic** | 10 |
| **Speed** | 12 |
| **Role** | Physical front-liner / party leader |
| **Joins** | Start of the game (in `starting_party`) |

Roland has the **highest HP and defense** in the party and a strong attack, so he
soaks hits and dishes out reliable physical damage. His magic is low — his few
spells lean on his attack instead, and MEND is a light patch rather than a full
heal. As the party leader he's the one you steer around the map; a roaming
[demon](demon.md) that touches him starts a battle.

## Skills

| Skill | Kind | MP | Target |
| ----- | ---- | -- | ------ |
| POWER STRIKE | Physical | 4 | One enemy |
| WHIRLWIND | Physical | 8 | All enemies |
| MEND | Heal | 6 | One ally |

**POWER STRIKE** is a heavy single-target hit for when one demon needs to fall now;
**WHIRLWIND** sweeps a whole pack of demons at once (great against a duo or trio);
**MEND** tops up a wounded ally in a pinch. See [Battles](../battles.md#skills) for
how each is resolved.

## Appearance

Roland uses the `swordsman` sprite sheet (a 5×12 grid of 16×16 frames): walk rows for
moving on the overworld, and a ready/attack pose in battle.
