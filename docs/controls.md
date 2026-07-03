---
comments: true
---

# Controls

The whole game is played on the keyboard — there is no mouse input. The same
handful of logical buttons drive every screen; only what they *do* changes with
context.

## Buttons

| Button | Keys | What it does |
| ------ | ---- | ------------ |
| **Move** | <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> / <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> | Move the party leader, the map cursor, or a menu selection |
| **Confirm** | <kbd>Enter</kbd> / <kbd>Z</kbd> / <kbd>Space</kbd> | Start, select, confirm a command, advance dialogue |
| **Cancel** | <kbd>Esc</kbd> / <kbd>X</kbd> / <kbd>Backspace</kbd> | Back out of a menu, leave a level to the map |
| **Menu** | <kbd>Shift</kbd> / <kbd>C</kbd> | Leave the current level and return to the world map |

Movement is analog-feeling: hold a direction to keep walking, and diagonals work
by holding two directions at once.

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
- **Cancel** / **Menu** — leave the level and go back to the world map. Progress
  in the level (which demons you've beaten) is kept for the rest of the session.

Touching a roaming demon starts a [battle](battles.md).

### Battle

- **Move** — move the highlight through the command menu, or through the list of
  targets when you're choosing who to hit or heal.
- **Confirm** — choose the highlighted command / skill / target.
- **Cancel** — back up one step (out of the skill or target menu, or back to the
  previous hero's turn to re-plan it).

See **[Battles](battles.md)** for the ATTACK / SKILL / DEFEND menu in detail.

### Dialogue & cutscenes

- **Confirm** (or **Cancel**) — reveal the rest of the current line instantly, or,
  once it's fully shown, advance to the next line.

### Victory / defeat report

- **Confirm** (any button) — dismiss the report after a short pause and continue.
