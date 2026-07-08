//! **DEV-ONLY** developer menu, opened with F1 from the world map.
//!
//! Compiled out of `--release` builds entirely (the module is declared behind
//! `#[cfg(debug_assertions)]` in `lib.rs`), so a shipped game has no such menu.
//! It lets a developer jump the party to any level, add any character to the
//! party, or start a battle against any encounter — the three fiddly bits of
//! state that are otherwise slow to reach while testing.
//!
//! Like the other menu scenes ([`crate::input_config`], [`crate::inventory`]),
//! its [`update`](DevTools::update) returns an `Option<DevToolsEvent>` describing
//! what the game should do — the menu itself never touches the party, registry,
//! or battle directly, keeping the borrows clean. The game applies the event and
//! usually re-enters the menu, so several tweaks can be made in one visit.

use crate::input::{Button, Input};
use crate::renderer::{color, virtual_w, Renderer, VIRTUAL_H};

/// What a [`DevTools::update`] wants the game to do next.
pub enum DevToolsEvent {
    /// Close the menu, back to the map.
    Close,
    /// Set every party member to this level.
    SetLevel(i32),
    /// Recruit the character with this id into the party.
    AddMember(String),
    /// Start a battle against the encounter with this id.
    Fight(String),
}

/// Which sub-screen of the dev menu is showing.
enum Page {
    /// The top-level list of actions.
    Main,
    /// A level-number spinner.
    Level,
    /// A picker over every character definition.
    Members,
    /// A picker over every encounter definition.
    Encounters,
}

/// The four rows of the main menu, in order.
const MAIN_ITEMS: [&str; 4] = [
    "CHANGE LEVEL",
    "ADD PARTY MEMBER",
    "FIGHT ENCOUNTER",
    "DONE",
];

/// How many list rows are visible at once on the picker pages.
const VISIBLE: usize = 11;

pub struct DevTools {
    page: Page,
    /// Cursor within the current page (reused as each page is entered).
    cursor: usize,
    /// The level the spinner is set to (starts at the party's current level).
    level: i32,
    /// Every character: `(id, display name)`, for the add-member picker.
    characters: Vec<(String, String)>,
    /// Every encounter id, for the fight picker.
    encounters: Vec<String>,
}

impl DevTools {
    /// Build the menu from the registry's content lists. `current_level` seeds the
    /// level spinner so it opens on the party's actual level.
    pub fn new(
        current_level: i32,
        characters: Vec<(String, String)>,
        encounters: Vec<String>,
    ) -> Self {
        DevTools {
            page: Page::Main,
            cursor: 0,
            level: current_level.max(1),
            characters,
            encounters,
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<DevToolsEvent> {
        match self.page {
            Page::Main => self.update_main(input),
            Page::Level => self.update_level(input),
            Page::Members => self.update_members(input),
            Page::Encounters => self.update_encounters(input),
        }
    }

    fn update_main(&mut self, input: &Input) -> Option<DevToolsEvent> {
        // Cancel/Menu from the top level closes the whole menu.
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            return Some(DevToolsEvent::Close);
        }
        let rows = MAIN_ITEMS.len();
        self.move_cursor(input, rows);
        if input.pressed(Button::Confirm) {
            match self.cursor {
                0 => self.enter(Page::Level),
                1 => self.enter(Page::Members),
                2 => self.enter(Page::Encounters),
                _ => return Some(DevToolsEvent::Close),
            }
        }
        None
    }

    fn update_level(&mut self, input: &Input) -> Option<DevToolsEvent> {
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            self.enter(Page::Main);
            return None;
        }
        // Left/Right nudge by one, Up/Down by ten; clamp to 1..=99.
        let delta = if input.pressed(Button::Right) {
            1
        } else if input.pressed(Button::Left) {
            -1
        } else if input.pressed(Button::Up) {
            10
        } else if input.pressed(Button::Down) {
            -10
        } else {
            0
        };
        if delta != 0 {
            self.level = (self.level + delta).clamp(1, 99);
        }
        if input.pressed(Button::Confirm) {
            let level = self.level;
            self.enter(Page::Main);
            return Some(DevToolsEvent::SetLevel(level));
        }
        None
    }

    fn update_members(&mut self, input: &Input) -> Option<DevToolsEvent> {
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            self.enter(Page::Main);
            return None;
        }
        let rows = self.characters.len();
        if rows == 0 {
            return None;
        }
        self.move_cursor(input, rows);
        if input.pressed(Button::Confirm) {
            // Stay on the page so several members can be added in a row.
            return Some(DevToolsEvent::AddMember(
                self.characters[self.cursor].0.clone(),
            ));
        }
        None
    }

    fn update_encounters(&mut self, input: &Input) -> Option<DevToolsEvent> {
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            self.enter(Page::Main);
            return None;
        }
        let rows = self.encounters.len();
        if rows == 0 {
            return None;
        }
        self.move_cursor(input, rows);
        if input.pressed(Button::Confirm) {
            return Some(DevToolsEvent::Fight(self.encounters[self.cursor].clone()));
        }
        None
    }

    /// Enter a page, resetting the cursor to the top.
    fn enter(&mut self, page: Page) {
        self.page = page;
        self.cursor = 0;
    }

    /// Wrapping up/down cursor movement over `rows` rows.
    fn move_cursor(&mut self, input: &Input, rows: usize) {
        if rows == 0 {
            self.cursor = 0;
            return;
        }
        self.cursor = self.cursor.min(rows - 1);
        if input.pressed(Button::Up) {
            self.cursor = (self.cursor + rows - 1) % rows;
        }
        if input.pressed(Button::Down) {
            self.cursor = (self.cursor + 1) % rows;
        }
    }

    pub fn draw(&self, r: &mut Renderer) {
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(14, 14, 26));
        r.draw_text_centered(
            "DEV TOOLS",
            virtual_w() / 2.0,
            14.0,
            1.6,
            color::rgb(255, 226, 120),
        );
        match self.page {
            Page::Main => self.draw_main(r),
            Page::Level => self.draw_level(r),
            Page::Members => self.draw_members(r),
            Page::Encounters => self.draw_encounters(r),
        }
    }

    fn draw_main(&self, r: &mut Renderer) {
        let x = 70.0;
        let mut y = 58.0;
        for (i, item) in MAIN_ITEMS.iter().enumerate() {
            self.draw_row(r, x, y, i == self.cursor, item);
            y += 18.0;
        }
        self.footer(r, "MOVE: PICK   CONFIRM: SELECT   CANCEL: CLOSE");
    }

    fn draw_level(&self, r: &mut Renderer) {
        r.draw_text_centered(
            "SET PARTY LEVEL",
            virtual_w() / 2.0,
            40.0,
            1.0,
            color::rgb(170, 180, 210),
        );
        r.draw_text_centered(
            &format!("< {} >", self.level),
            virtual_w() / 2.0,
            90.0,
            2.0,
            color::rgb(150, 220, 160),
        );
        self.footer(
            r,
            "LEFT/RIGHT: ±1   UP/DOWN: ±10   CONFIRM: APPLY   CANCEL: BACK",
        );
    }

    fn draw_members(&self, r: &mut Renderer) {
        r.draw_text_centered(
            "ADD PARTY MEMBER",
            virtual_w() / 2.0,
            34.0,
            1.0,
            color::rgb(170, 180, 210),
        );
        let labels: Vec<String> = self
            .characters
            .iter()
            .map(|(id, name)| format!("{name}  ({id})"))
            .collect();
        self.draw_list(r, &labels);
        self.footer(r, "MOVE: PICK   CONFIRM: ADD   CANCEL: BACK");
    }

    fn draw_encounters(&self, r: &mut Renderer) {
        r.draw_text_centered(
            "FIGHT ENCOUNTER",
            virtual_w() / 2.0,
            34.0,
            1.0,
            color::rgb(170, 180, 210),
        );
        let labels: Vec<String> = self
            .encounters
            .iter()
            .map(|id| id.replace('_', " ").to_uppercase())
            .collect();
        self.draw_list(r, &labels);
        self.footer(r, "MOVE: PICK   CONFIRM: FIGHT   CANCEL: BACK");
    }

    /// Draw a scrolling list of `labels`, windowed around the cursor.
    fn draw_list(&self, r: &mut Renderer, labels: &[String]) {
        let n = labels.len();
        if n == 0 {
            r.draw_text_centered(
                "(NONE DEFINED)",
                virtual_w() / 2.0,
                90.0,
                1.0,
                color::rgb(150, 150, 170),
            );
            return;
        }
        // Center the cursor in the window, clamped so it stays within the list.
        let offset = self
            .cursor
            .saturating_sub(VISIBLE / 2)
            .min(n.saturating_sub(VISIBLE));
        let x = 60.0;
        let mut y = 50.0;
        for (i, label) in labels.iter().enumerate().skip(offset).take(VISIBLE) {
            self.draw_row(r, x, y, i == self.cursor, label);
            y += 11.0;
        }
        // Scroll affordances when there's more above or below the window.
        if offset > 0 {
            r.draw_text_centered("^", virtual_w() / 2.0, 42.0, 1.0, color::rgb(150, 150, 170));
        }
        if offset + VISIBLE < n {
            r.draw_text_centered(
                "v",
                virtual_w() / 2.0,
                50.0 + VISIBLE as f32 * 11.0,
                1.0,
                color::rgb(150, 150, 170),
            );
        }
    }

    fn draw_row(&self, r: &mut Renderer, x: f32, y: f32, selected: bool, label: &str) {
        let col = if selected {
            r.draw_text(">", x - 10.0, y, 1.0, color::rgb(255, 240, 150));
            color::rgb(255, 240, 150)
        } else {
            color::WHITE
        };
        r.draw_text(label, x, y, 1.0, col);
    }

    fn footer(&self, r: &mut Renderer, hint: &str) {
        r.draw_text_centered(
            hint,
            virtual_w() / 2.0,
            VIRTUAL_H - 12.0,
            1.0,
            color::rgb(150, 150, 170),
        );
    }
}
