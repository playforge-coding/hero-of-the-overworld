---
comments: true
---

# Captain

The fourth hero to join the party — and the only one you **fight first**. The
**Captain** holds the last camp on the **[Castaway Shore](../world.md#progression-is-linear)**
as its [boss](#the-boss-fight); beat him (and clear the rest of the shore) and he
yields, throws in with the party, and takes the field at your side. A former officer
of the kingdom's navy turned corsair, he fights the way his gunners do, only far
harder: he opens with a **musket** and can rake a whole enemy line with a **broadside**.

!!! note "A late arrival — and a reserve"
    The Captain joins in **Chapter 2**, so a battle can now field more heroes than it
    seats. Only **[three fight at once](../gameplay.md#the-battle-line-up)**; the
    Captain lands on the **bench** when he joins, so swap him into the line-up from the
    **[party menu](../gameplay.md#the-battle-line-up)** if you want him in the fight.

## Stats

| Stat | Value |
|---|---|
| **HP** | 104 |
| **MP** | 20 |
| **Attack** | 24 |
| **Defense** | 15 |
| **Magic** | 6 |
| **Speed** | 14 |
| **Joins** | After clearing the Castaway Shore (Chapter 2) |

Like every recruit, he **joins at the party's current level**, his stats grown up the
same curve, so he arrives on par rather than as dead weight. He's a sturdy
ranged fighter — more armor and HP than his gunners, and an attack that leans on his
**musket** rather than raw melee.

## Skills

| Skill | Kind | MP | Target | Learned | Notes |
| ----- | ---- | -- | ------ | ------- | ----- |
| MUSKET SHOT | Physical | 0 | One enemy | Start | 175% power, **unblockable** — a heavy musket ball, harder-hitting than a gunner's pistol |
| BROADSIDE | Physical | 8 | All enemies | Level 5 | 130% power, unblockable — a raking volley that sweeps the whole enemy line |
| ALL HANDS | Physical | 14 | All enemies | Level 9 | 165% power, unblockable — his **finale**: he calls up the whole crew and a boarding party swarms every foe at once |

**MUSKET SHOT** is his bread and butter: a free, unblockable shot that outranges and
outhits the [pirate gunner](pirate_gunner.md)'s pistol. **BROADSIDE** is an MP-fed
volley that hits *every* foe, making him the party's answer to a crowded field. And
**ALL HANDS** is his **final move** and showpiece — a new **[Crowd](../battles.md#attack-animations)**
attack that floods the entire screen with his pirate crew as they fall on the whole
enemy line at once, then melt back to the ship.

## The boss fight

Before he's an ally he's a foe: the `pirate_captain` **boss** encounter, waiting at the
back of his camp on the last screen of the shore. He's a hard but **winnable** fight
(unlike the [Demon King](demon_king.md), he can be beaten) — a big HP pool and that
same lethal musket. Felling him is what triggers his recruitment, so the enemy and the
ally share **one sprite**: the foe you cut down looks exactly like the hero who stands
back up beside you.

## Appearance

The Captain uses the `captain` sprite sheet — an 8×8 grid of 16×16 frames on the same
playable layout as [Gareth](gareth.md)'s (walk rows 0–3, attack rows 4–7). Both his
boss and hero forms read the right-facing walk/attack rows, so he looks identical
whether he's shooting at you or for you.
