---
comments: true
---

# Controls

The game plays on the keyboard **or a gamepad** — there is no mouse input. The
same handful of logical buttons drive every screen; only what they *do* changes
with context.

## Buttons

| Button | Keys | Gamepad | What it does |
| ------ | ---- | ------- | ------------ |
| **Move** | <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> / <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> | D-pad or left stick | Move the party leader, the map cursor, or a menu selection |
| **Confirm** | <kbd>Enter</kbd> / <kbd>Z</kbd> / <kbd>Space</kbd> | A (south button) | Start, select, confirm a command, advance dialogue |
| **Cancel** | <kbd>Esc</kbd> / <kbd>X</kbd> / <kbd>Backspace</kbd> | B (east button) | Back out of a menu, leave a level to the map |
| **Menu** | <kbd>Shift</kbd> / <kbd>C</kbd> | Start / Select | Leave the current level and return to the world map |

Movement is analog-feeling: hold a direction to keep walking, and diagonals work
by holding two directions at once (a diagonal on the stick counts too).

## Gamepads

Plug in a controller and it just works — anywhere the keyboard does, a gamepad
does too, and you can use both at once. Gamepad support is native-only; the
[web build](getting-started.md) stays keyboard-only.

### One controller per party member

Connect **more than one** gamepad and each one takes over a **party member**:
the first gamepad (and the keyboard) drives member 1, the second drives member 2,
and so on. This matters in the [battle](battles.md) command phase — when it's a
hero's turn to choose ATTACK / SKILL / DEFEND, *that hero's* controller drives
the menu, so a couch of players can each plan their own character. Everywhere
else (the world map, walking a level, dialogue) any connected controller works,
since those screens are single-player.

If there are more party members than gamepads, the extra members fall back to
the shared input, so a lone player still commands the whole party in turn — the
game plays exactly as it does on a single controller.

## What the buttons do, screen by screen

### Title

- **Confirm** — begin, opening the [world map](world.md#the-world-map).

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
- **Cancel** / **Menu** — leave the level and go back to the world map. Progress
  in the level (which demons you've beaten) is [saved](gameplay.md#saving), so
  it's kept even across sessions.

Touching a roaming enemy starts a [battle](battles.md).

### Inside a shop

- **Move** — walk the room, or walk **out the doorway** (the wall the keeper
  faces) to leave.
- **Confirm** at the counter — open the [buy menu](shops.md#buying).
- In the buy menu: **Up/Down** pick an item, **Left/Right** pick which hero to
  outfit, **Confirm** buys and equips, **Cancel** closes the menu.

### Battle

- **Move** — move the highlight through the command menu, or through the list of
  targets when you're choosing who to hit or heal.
- **Confirm** — choose the highlighted command / skill / target.
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
