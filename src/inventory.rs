//! The party inventory / equipment screen.
//!
//! Opened with the **Menu** button while walking a level, this is where you manage
//! gear outside of a shop. The party owns a shared **bag** of unequipped items
//! (see [`crate::party::Party::bag`]); here you swap each member's weapon and armor
//! with items from that bag, or unequip them back into it. Nothing is bought or
//! lost — gear only moves between members and the bag, so the count is conserved.
//!
//! Flow is two steps, like the shop's buy menu: pick a member + slot (PARTY
//! focus), then pick a bag item or UNEQUIP for it (BAG focus).

use crate::data::{EquipSlot, Registry};
use crate::input::{Button, Input};
use crate::party::Party;
use crate::renderer::{color, Renderer, VIRTUAL_H, VIRTUAL_W};
use crate::shop::summarize;

/// What an [`Inventory::update`] wants the game to do next.
pub enum InventoryEvent {
    /// Close the screen and return to the level.
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    /// Choosing which member and slot to change.
    Party,
    /// Choosing a bag item (or UNEQUIP) to put in the chosen member's slot.
    Bag,
}

/// Sentinel first entry of the bag chooser: "take the current item off".
const UNEQUIP: usize = usize::MAX;

pub struct Inventory {
    focus: Focus,
    /// Selected party member (row).
    member: usize,
    /// Selected equipment slot for that member.
    slot: EquipSlot,
    /// Cursor into the slot-filtered bag chooser (index 0 is UNEQUIP).
    bag_cursor: usize,
    time: f32,
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            focus: Focus::Party,
            member: 0,
            slot: EquipSlot::Weapon,
            bag_cursor: 0,
            time: 0.0,
        }
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(
        &mut self,
        input: &Input,
        party: &mut Party,
        reg: &Registry,
        dt: f32,
    ) -> Option<InventoryEvent> {
        self.time += dt;
        let members = party.members.len().max(1);
        self.member = self.member.min(members - 1);

        match self.focus {
            Focus::Party => {
                if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                }
                if input.pressed(Button::Up) {
                    self.member = (self.member + members - 1) % members;
                }
                if input.pressed(Button::Down) {
                    self.member = (self.member + 1) % members;
                }
                if input.pressed(Button::Left) || input.pressed(Button::Right) {
                    self.slot = other_slot(self.slot);
                }
                if input.pressed(Button::Confirm) {
                    self.focus = Focus::Bag;
                    self.bag_cursor = 0;
                }
            }
            Focus::Bag => {
                let choices = self.choices(party, reg);
                let n = choices.len();
                self.bag_cursor = self.bag_cursor.min(n - 1);
                if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
                    self.focus = Focus::Party;
                } else if input.pressed(Button::Up) {
                    self.bag_cursor = (self.bag_cursor + n - 1) % n;
                } else if input.pressed(Button::Down) {
                    self.bag_cursor = (self.bag_cursor + 1) % n;
                } else if input.pressed(Button::Confirm) {
                    match choices[self.bag_cursor] {
                        UNEQUIP => party.unequip(self.member, self.slot),
                        bag_idx => {
                            party.equip_from_bag(reg, self.member, bag_idx);
                        }
                    }
                    self.focus = Focus::Party;
                }
            }
        }
        None
    }

    /// The bag chooser for the selected slot: a leading [`UNEQUIP`] sentinel, then
    /// the bag indices of items that fit the slot.
    fn choices(&self, party: &Party, reg: &Registry) -> Vec<usize> {
        let mut v = vec![UNEQUIP];
        v.extend(party.bag_indices_for(reg, self.slot));
        v
    }

    pub fn draw(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        r.draw_rect(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, color::rgb(14, 14, 26));

        // Header.
        r.draw_text("EQUIPMENT", 8.0, 6.0, 1.2, color::rgb(255, 226, 120));
        let gold = format!("GOLD {}", party.gold);
        let gw = r.text_width(&gold, 1.0);
        r.draw_text(
            &gold,
            VIRTUAL_W - gw - 8.0,
            7.0,
            1.0,
            color::rgb(240, 220, 130),
        );

        self.draw_members(r, party, reg);
        self.draw_side(r, party, reg);
        self.draw_hint(r);
    }

    /// Left panel: each member with their weapon and armor.
    fn draw_members(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        let (px, py, pw) = (6.0, 20.0, 176.0);
        let rows = party.members.len().max(1);
        let ph = rows as f32 * 34.0 + 6.0;
        r.draw_rect(px, py, pw, ph, color::rgba(10, 12, 24, 230));
        r.draw_rect_outline(px, py, pw, ph, 1.0, color::rgba(80, 90, 140, 255));

        for (i, m) in party.members.iter().enumerate() {
            let y = py + 5.0 + i as f32 * 34.0;
            let selected = i == self.member;
            let name_col = if selected {
                color::rgb(255, 240, 150)
            } else {
                color::WHITE
            };
            if selected {
                r.draw_text(">", px + 3.0, y, 1.0, color::rgb(255, 240, 150));
            }
            r.draw_text(&m.name, px + 12.0, y, 1.0, name_col);

            for (row, slot) in [EquipSlot::Weapon, EquipSlot::Armor]
                .into_iter()
                .enumerate()
            {
                let sy = y + 10.0 + row as f32 * 10.0;
                // Highlight the slot the cursor is on for the selected member.
                let active = selected && slot == self.slot;
                if active {
                    let hl = if self.focus == Focus::Bag {
                        color::rgba(70, 60, 30, 255)
                    } else {
                        color::rgba(40, 50, 90, 255)
                    };
                    r.draw_rect(px + 10.0, sy - 1.0, pw - 16.0, 9.0, hl);
                }
                let label = match slot {
                    EquipSlot::Weapon => "WPN",
                    EquipSlot::Armor => "ARM",
                };
                r.draw_text(label, px + 12.0, sy, 1.0, color::rgb(150, 190, 240));
                let item = party
                    .equipped(i, slot)
                    .and_then(|id| reg.equipment(id))
                    .map(|e| e.name.as_str())
                    .unwrap_or("(none)");
                let col = if party.equipped(i, slot).is_some() {
                    color::rgb(210, 210, 225)
                } else {
                    color::rgb(120, 120, 140)
                };
                r.draw_text(item, px + 38.0, sy, 1.0, col);
            }
        }
    }

    /// Right panel: in PARTY focus, the whole bag; in BAG focus, the slot-filtered
    /// chooser with UNEQUIP and a highlighted selection.
    fn draw_side(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        let (px, py, pw, ph) = (188.0, 20.0, 126.0, 132.0);
        r.draw_rect(px, py, pw, ph, color::rgba(10, 12, 24, 230));
        r.draw_rect_outline(px, py, pw, ph, 1.0, color::rgba(80, 90, 140, 255));

        match self.focus {
            Focus::Party => {
                r.draw_text("BAG", px + 4.0, py + 4.0, 1.0, color::rgb(170, 180, 210));
                if party.bag.is_empty() {
                    r.draw_text(
                        "(EMPTY)",
                        px + 6.0,
                        py + 16.0,
                        1.0,
                        color::rgb(120, 120, 140),
                    );
                } else {
                    for (row, id) in party.bag.iter().enumerate().take(11) {
                        let name = reg
                            .equipment(id)
                            .map(|e| e.name.as_str())
                            .unwrap_or(id.as_str());
                        r.draw_text(
                            name,
                            px + 6.0,
                            py + 16.0 + row as f32 * 10.0,
                            1.0,
                            color::rgb(200, 200, 215),
                        );
                    }
                }
            }
            Focus::Bag => {
                let slot_name = match self.slot {
                    EquipSlot::Weapon => "WEAPON",
                    EquipSlot::Armor => "ARMOR",
                };
                r.draw_text(
                    slot_name,
                    px + 4.0,
                    py + 4.0,
                    1.0,
                    color::rgb(255, 210, 120),
                );
                let choices = self.choices(party, reg);
                for (row, &choice) in choices.iter().enumerate() {
                    let y = py + 16.0 + row as f32 * 10.0;
                    if y > py + ph - 20.0 {
                        break;
                    }
                    let on = row == self.bag_cursor;
                    if on {
                        r.draw_rect(
                            px + 2.0,
                            y - 1.0,
                            pw - 4.0,
                            9.0,
                            color::rgba(60, 70, 120, 255),
                        );
                    }
                    let (text, col) = match choice {
                        UNEQUIP => ("(UNEQUIP)".to_string(), color::rgb(200, 160, 160)),
                        idx => {
                            let name = party
                                .bag
                                .get(idx)
                                .and_then(|id| reg.equipment(id))
                                .map(|e| e.name.clone())
                                .unwrap_or_default();
                            (name, color::WHITE)
                        }
                    };
                    r.draw_text(&text, px + 6.0, y, 1.0, col);
                }
                // Stat summary of the highlighted item.
                if let Some(&idx) = choices.get(self.bag_cursor) {
                    if idx != UNEQUIP {
                        if let Some(item) = party.bag.get(idx).and_then(|id| reg.equipment(id)) {
                            r.draw_text(
                                &summarize(item),
                                px + 4.0,
                                py + ph - 10.0,
                                1.0,
                                color::rgb(150, 220, 160),
                            );
                        }
                    }
                }
            }
        }
    }

    fn draw_hint(&self, r: &mut Renderer) {
        let hint = match self.focus {
            Focus::Party => "MOVE: PICK HERO/SLOT   CONFIRM: CHANGE   MENU/CANCEL: CLOSE",
            Focus::Bag => "MOVE: PICK ITEM   CONFIRM: EQUIP   CANCEL: BACK",
        };
        r.draw_text_centered(
            hint,
            VIRTUAL_W / 2.0,
            VIRTUAL_H - 12.0,
            1.0,
            color::rgb(150, 150, 170),
        );
    }
}

fn other_slot(slot: EquipSlot) -> EquipSlot {
    match slot {
        EquipSlot::Weapon => EquipSlot::Armor,
        EquipSlot::Armor => EquipSlot::Weapon,
    }
}
