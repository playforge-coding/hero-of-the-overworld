//! The controls / input-mapping screen.
//!
//! Opened with the **Menu** button from the title, this assigns each input
//! *source* — the keyboard and every connected gamepad — to a **player** number.
//! That's how two people share the party: put the keyboard on Player 1 and a pad
//! on Player 2 and each drives their own heroes; leave them on the same player and
//! one person drives everything. The party is then dealt across the players
//! round-robin, so two players cover three heroes with none left out (see
//! [`crate::input::InputAssignment`]).

use crate::input::{Button, Input, InputAssignment};
use crate::renderer::{color, Renderer, VIRTUAL_H, VIRTUAL_W};

/// What an [`InputConfig::update`] wants the game to do next.
pub enum InputConfigEvent {
    /// Close the screen, keeping this (possibly edited) mapping.
    Close(InputAssignment),
}

pub struct InputConfig {
    assign: InputAssignment,
    /// Number of connected gamepads (kept in step via [`Self::sync_gamepads`]).
    gamepad_count: usize,
    /// Row cursor: 0 = keyboard, 1..=gamepad_count = each pad, last = DONE.
    cursor: usize,
}

impl InputConfig {
    pub fn new(assign: InputAssignment) -> Self {
        InputConfig {
            assign,
            gamepad_count: 0,
            cursor: 0,
        }
    }

    /// Track the live number of connected pads, making sure the assignment has an
    /// editable entry for each. Called as pads connect/disconnect.
    pub fn sync_gamepads(&mut self, count: usize) {
        self.gamepad_count = count;
        self.assign.ensure_gamepads(count);
    }

    /// One player number per input source: the keyboard plus every pad. This is
    /// the ceiling on player count — you can't have more players than sources.
    fn source_count(&self) -> usize {
        1 + self.gamepad_count
    }

    /// Total selectable rows: one per source, plus the trailing DONE row.
    fn row_count(&self) -> usize {
        self.source_count() + 1
    }

    fn done_row(&self) -> usize {
        self.source_count()
    }

    pub fn update(&mut self, input: &Input) -> Option<InputConfigEvent> {
        let rows = self.row_count();
        self.cursor = self.cursor.min(rows - 1);

        // Cancel/Menu closes and keeps the mapping.
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            return Some(InputConfigEvent::Close(self.assign.clone()));
        }
        if input.pressed(Button::Up) {
            self.cursor = (self.cursor + rows - 1) % rows;
        }
        if input.pressed(Button::Down) {
            self.cursor = (self.cursor + 1) % rows;
        }

        if self.cursor == self.done_row() {
            if input.pressed(Button::Confirm) {
                return Some(InputConfigEvent::Close(self.assign.clone()));
            }
        } else {
            // A source row: Left/Right (or Confirm to advance) cycle its player.
            let players = self.source_count();
            let delta = if input.pressed(Button::Right) || input.pressed(Button::Confirm) {
                1
            } else if input.pressed(Button::Left) {
                players - 1 // -1 mod players
            } else {
                0
            };
            if delta != 0 {
                let slot = self.cursor_source_player_mut();
                *slot = (*slot + delta) % players;
            }
        }
        None
    }

    /// Mutable player number for the source the cursor is on (keyboard or a pad).
    fn cursor_source_player_mut(&mut self) -> &mut usize {
        if self.cursor == 0 {
            &mut self.assign.keyboard
        } else {
            // Rows 1..=gamepad_count map to pads 0..gamepad_count-1. `sync_gamepads`
            // guarantees the entry exists.
            &mut self.assign.gamepads[self.cursor - 1]
        }
    }

    pub fn draw(&self, r: &mut Renderer) {
        r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(14, 14, 26));
        r.draw_text_centered(
            "CONTROLS",
            VIRTUAL_W / 2.0,
            14.0,
            1.6,
            color::rgb(255, 226, 120),
        );
        r.draw_text_centered(
            "ASSIGN EACH INPUT TO A PLAYER",
            VIRTUAL_W / 2.0,
            34.0,
            1.0,
            color::rgb(170, 180, 210),
        );

        let x = 60.0;
        let mut y = 58.0;
        // Keyboard row, then one per gamepad.
        self.draw_row(r, x, y, 0, "KEYBOARD", Some(self.assign.keyboard));
        y += 16.0;
        for i in 0..self.gamepad_count {
            self.draw_row(
                r,
                x,
                y,
                i + 1,
                &format!("GAMEPAD {}", i + 1),
                Some(self.assign.gamepad_player(i)),
            );
            y += 16.0;
        }
        // DONE row.
        y += 6.0;
        self.draw_row(r, x, y, self.done_row(), "DONE", None);

        let hint = if self.gamepad_count == 0 {
            "CONNECT A GAMEPAD TO SPLIT THE PARTY   —   MENU/CANCEL: BACK"
        } else {
            "MOVE: PICK ROW   LEFT/RIGHT: CHANGE PLAYER   MENU/CANCEL: BACK"
        };
        r.draw_text_centered(
            hint,
            VIRTUAL_W / 2.0,
            VIRTUAL_H - 12.0,
            1.0,
            color::rgb(150, 150, 170),
        );
    }

    fn draw_row(
        &self,
        r: &mut Renderer,
        x: f32,
        y: f32,
        row: usize,
        label: &str,
        player: Option<usize>,
    ) {
        let selected = self.cursor == row;
        if selected {
            r.draw_text(">", x - 10.0, y, 1.0, color::rgb(255, 240, 150));
        }
        let label_col = if selected {
            color::rgb(255, 240, 150)
        } else {
            color::WHITE
        };
        r.draw_text(label, x, y, 1.0, label_col);
        if let Some(p) = player {
            let text = format!("< PLAYER {} >", p + 1);
            r.draw_text(&text, x + 90.0, y, 1.0, color::rgb(150, 220, 160));
        }
    }
}
