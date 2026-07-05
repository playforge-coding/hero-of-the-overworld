---
comments: true
---

# Controls

The game plays on the keyboard, **a gamepad**, or **on-screen touch controls** —
there is no mouse input. The same handful of logical buttons drive every screen;
only what they *do* changes with context.

## Buttons

| Button | Keys | Gamepad | What it does |
| ------ | ---- | ------- | ------------ |
| **Move** | <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> / <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> | D-pad or left stick | Move the party leader, the map cursor, or a menu selection |
| **Confirm** | <kbd>Enter</kbd> / <kbd>Z</kbd> / <kbd>Space</kbd> | A (south button) | Start, select, confirm a command, advance dialogue |
| **Cancel** | <kbd>Esc</kbd> / <kbd>X</kbd> / <kbd>Backspace</kbd> | B (east button) | Back out of a menu, leave a level to the map |
| **Menu** | <kbd>Shift</kbd> / <kbd>C</kbd> | Start / Select | Open the party [inventory / equipment](gameplay.md#inventory-and-equipment) screen while in a level |

Movement is analog-feeling: hold a direction to keep walking, and diagonals work
by holding two directions at once (a diagonal on the stick counts too).

## Gamepads

Plug in a controller and it just works — anywhere the keyboard does, a gamepad
does too, and you can use both at once. Gamepads work on **both** the desktop and
the **[web build](getting-started.md)** (in the browser you may need to press a
button on the pad once so the browser reveals it). See
[local co-op](#local-co-op-share-the-party) below for playing with two people.

## Touch controls

On a touchscreen — a phone, a tablet, or the [web build](getting-started.md) —
the game shows on-screen controls: a directional control in the bottom-left,
**Confirm** (A) and **Cancel** (B) in the bottom-right, and **Menu** (☰) in the
top-right. The overlay stays hidden until the first touch, so a desktop keyboard
or gamepad session never sees it. Like the keyboard, the touch controls command
party member 1. They sit in the letterbox margins around the 320×180 game area,
and the layout tracks the screen as you rotate or resize it.

The directional control fits what you're doing:

- **Overworld** — a floating **joystick**. Press anywhere in the lower-left and
  the stick anchors under your thumb, then follows that finger (even past the
  ring) until you lift it, so walking never cuts out mid-drag. Push in any
  direction, diagonals included.
- **Menus** (title, world map, shop, dialogue) — a plain **d-pad**.
- **Battle** — just **up/down**, since the command menu is a vertical list.
  Left/right and Menu are hidden there; A confirms and B backs out.

### Local co-op: share the party

Two people can play at once. Each **input source** — the keyboard and every
connected gamepad — is a **player**, and party members are dealt out to players
**round-robin**. This matters in the [battle](battles.md) command phase — when it's
a hero's turn to choose ATTACK / SKILL / DEFEND (and to land its
[timed hits](battles.md#action-timing-strikes-and-blocks)), *that hero's* player
drives it, so a couch of players each plan their own characters.

When there are **more party members than players**, the extra members wrap back
around rather than being left unowned — so **two players share three heroes
cleanly**: player 1 handles members 1 & 3, player 2 handles member 2. With a single
input source, every member maps to it, so a lone player still commands the whole
party in turn.

#### Assigning inputs to players

By default the keyboard and the first gamepad are the **same** player 1 (so one
person can use either), and each further gamepad is its own player. To change that
— for instance to have **one person on the keyboard and another on a controller** —
open the **CONTROLS** screen: press **Menu** (<kbd>Shift</kbd>/<kbd>C</kbd>, or
Start) on the **title screen**. Pick each row (KEYBOARD, GAMEPAD 1, …) and use
**Left/Right** to set which **player** it belongs to, then **DONE**. Put the
keyboard on Player 1 and the pad on Player 2 and the two of you split the party.
The mapping is [saved](gameplay.md#saving) with your game.

Everywhere outside battle (the world map, walking a level, the inventory,
dialogue) any connected controller — or the keyboard — works, since those screens
are single-player.

## What the buttons do, screen by screen

### Title

- **Confirm** — begin (or **CONTINUE** a saved game), opening the
  [world map](world.md#the-world-map).
- **Menu** — open the **CONTROLS** screen to
  [assign inputs to players](#assigning-inputs-to-players) for local co-op.

### World map

- **Move** — jump the cursor to the nearest [level marker](world.md#the-world-map)
  in that direction.
- **Confirm** — enter the selected level.
- **Cancel** — return to the title.

### Inside a level

- **Move** — walk the party leader around the [tile screen](world.md#screens).
  Walking into a mid-edge opening flips to the neighbouring screen.
- **Confirm** — when standing by a [shopkeeper](shops.md) (a **PRESS Z** prompt
  shows), step into their shop.
- **Menu** — open the party [inventory / equipment](gameplay.md#inventory-and-equipment)
  screen to change gear on the go.
- **Cancel** — leave the level and go back to the world map. Progress in the level
  (which demons you've beaten) is [saved](gameplay.md#saving), so it's kept even
  across sessions.

Touching a roaming enemy starts a [battle](battles.md).

### Inventory / equipment

- **Move** — Up/Down pick a hero, Left/Right pick their weapon or armor slot.
- **Confirm** — open the bag chooser for that slot, then Confirm again to equip the
  highlighted item (or **(UNEQUIP)** to stow the current one).
- **Cancel** — back out of the chooser, or (from the hero list) **Menu**/**Cancel**
  closes the screen and returns you to the level.

### Inside a shop

- **Move** — walk the room, or walk **out the doorway** (the wall the keeper
  faces) to leave.
- **Confirm** at the counter — open the [buy menu](shops.md#buying).
- In the buy menu: **Up/Down** pick an item, **Left/Right** pick which hero to
  outfit, **Confirm** buys and equips, **Cancel** closes the menu.

### Battle

- **Move** — move the highlight through the command menu, or through the list of
  targets when you're choosing who to hit or heal.
- **Confirm** — choose the highlighted command / skill / target. It's also the
  **timed-hit** tap: press it as a blow connects to add damage (or, when bracing,
  to block), and again on an attack's recovery to [taunt](battles.md#taunting-a-foe)
  the foe you just struck.
- **Cancel** — back up one step (out of the skill or target menu, or back to the
  previous hero's turn to re-plan it).

With multiple gamepads connected, the hero currently choosing is driven by the
controller assigned to their party slot (see [Gamepads](#gamepads) above).

See **[Battles](battles.md)** for the ATTACK / SKILL / DEFEND menu in detail.

### Dialogue & cutscenes

- **Confirm** (or **Cancel**) — reveal the rest of the current line instantly, or,
  once it's fully shown, advance to the next line.

### Victory / defeat report

- **Confirm** (any button) — dismiss the report after a short pause and continue.
