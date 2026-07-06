//! The party menu / inventory screen.
//!
//! Opened with the **Menu** button while walking a level, this is the party's
//! out-of-battle hub. Pick a hero, then from their **action menu** you can:
//!
//! - **EQUIP** — swap the hero's weapon/armor with the shared **bag** of
//!   unequipped gear (see [`crate::party::Party::bag`]); nothing is bought or
//!   lost, gear only moves between heroes and the bag.
//! - **USE ITEM** — spend a restorative [item](crate::data::ItemDef) (a potion,
//!   an ether) on a chosen hero, right here on the map.
//! - **USE MOVE** — have the hero cast a healing [skill](crate::data::SkillDef)
//!   they know on a chosen ally, spending MP just like in battle.
//!
//! So healing — by item *or* move — works between fights, not only mid-battle.
//! Offensive items and buffs are deliberately battle-only (they need a foe / don't
//! persist), so only restorative effects show up here.

use crate::data::{EquipSlot, Registry};
use crate::input::{Button, Input};
use crate::party::{FieldUse, Party};
use crate::renderer::{color, virtual_w, Renderer, VIRTUAL_H};
use crate::shop::summarize;

/// What an [`Inventory::update`] wants the game to do next.
pub enum InventoryEvent {
    /// Close the screen and return to the level.
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    /// Choosing which hero (and equip slot) to act on.
    Party,
    /// The chosen hero's action menu: EQUIP / USE ITEM / USE MOVE.
    Action,
    /// Choosing a bag item (or UNEQUIP) to put in the chosen hero's slot.
    Bag,
    /// Choosing a restorative item to use.
    ItemList,
    /// Choosing one of the hero's healing moves to cast.
    MoveList,
    /// Choosing which ally the pending item/move applies to.
    Target,
}

/// The action queued up while the player picks a [`Focus::Target`].
enum Pending {
    /// Use the item at this stash index on the target.
    Item(usize),
    /// Have the selected hero cast this skill id on the target.
    Move(String),
}

/// Sentinel first entry of the bag chooser: "take the current item off".
const UNEQUIP: usize = usize::MAX;

/// The chosen hero's action menu.
const ACTIONS: [&str; 3] = ["EQUIP", "USE ITEM", "USE MOVE"];

/// A rectangular panel's geometry, bundled so the chooser draw helpers take one
/// argument instead of four loose floats.
#[derive(Clone, Copy)]
struct Panel {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

pub struct Inventory {
    focus: Focus,
    /// Selected party member — the hero being acted on (and the caster of moves).
    member: usize,
    /// Selected equipment slot for that member.
    slot: EquipSlot,
    /// Cursor into the slot-filtered bag chooser (index 0 is UNEQUIP).
    bag_cursor: usize,
    /// Cursor into the action menu.
    action_cursor: usize,
    /// Cursor into the item / move chooser lists.
    list_cursor: usize,
    /// Cursor into the target chooser (a party-member index).
    target_cursor: usize,
    /// The item/move awaiting a target while in [`Focus::Target`].
    pending: Option<Pending>,
    /// Transient feedback line: (text, seconds remaining).
    message: Option<(String, f32)>,
    time: f32,
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            focus: Focus::Party,
            member: 0,
            slot: EquipSlot::Weapon,
            bag_cursor: 0,
            action_cursor: 0,
            list_cursor: 0,
            target_cursor: 0,
            pending: None,
            message: None,
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
        if let Some((_, t)) = &mut self.message {
            *t -= dt;
            if *t <= 0.0 {
                self.message = None;
            }
        }
        let members = party.members.len().max(1);
        self.member = self.member.min(members - 1);

        // Menu always closes the whole screen for a quick exit; Cancel steps back
        // one level (and closes from the top-level PARTY focus).
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
                    self.focus = Focus::Action;
                    self.action_cursor = 0;
                }
            }
            Focus::Action => {
                if input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                }
                let n = ACTIONS.len();
                if input.pressed(Button::Cancel) {
                    self.focus = Focus::Party;
                } else if input.pressed(Button::Up) {
                    self.action_cursor = (self.action_cursor + n - 1) % n;
                } else if input.pressed(Button::Down) {
                    self.action_cursor = (self.action_cursor + 1) % n;
                } else if input.pressed(Button::Confirm) {
                    self.list_cursor = 0;
                    self.focus = match self.action_cursor {
                        0 => {
                            self.bag_cursor = 0;
                            Focus::Bag
                        }
                        1 => Focus::ItemList,
                        _ => Focus::MoveList,
                    };
                }
            }
            Focus::Bag => {
                let choices = self.choices(party, reg);
                let n = choices.len();
                self.bag_cursor = self.bag_cursor.min(n - 1);
                if input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                } else if input.pressed(Button::Cancel) {
                    self.focus = Focus::Action;
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
                    self.focus = Focus::Action;
                }
            }
            Focus::ItemList => {
                let items = self.field_items(party, reg);
                if input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                } else if input.pressed(Button::Cancel) {
                    self.focus = Focus::Action;
                } else if !items.is_empty() {
                    let n = items.len();
                    self.list_cursor = self.list_cursor.min(n - 1);
                    if input.pressed(Button::Up) {
                        self.list_cursor = (self.list_cursor + n - 1) % n;
                    } else if input.pressed(Button::Down) {
                        self.list_cursor = (self.list_cursor + 1) % n;
                    } else if input.pressed(Button::Confirm) {
                        self.pending = Some(Pending::Item(items[self.list_cursor]));
                        self.target_cursor = self.member;
                        self.focus = Focus::Target;
                    }
                }
            }
            Focus::MoveList => {
                let moves = party.field_heal_skills(reg, self.member);
                if input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                } else if input.pressed(Button::Cancel) {
                    self.focus = Focus::Action;
                } else if !moves.is_empty() {
                    let n = moves.len();
                    self.list_cursor = self.list_cursor.min(n - 1);
                    if input.pressed(Button::Up) {
                        self.list_cursor = (self.list_cursor + n - 1) % n;
                    } else if input.pressed(Button::Down) {
                        self.list_cursor = (self.list_cursor + 1) % n;
                    } else if input.pressed(Button::Confirm) {
                        self.pending = Some(Pending::Move(moves[self.list_cursor].clone()));
                        self.target_cursor = self.member;
                        self.focus = Focus::Target;
                    }
                }
            }
            Focus::Target => {
                if input.pressed(Button::Menu) {
                    return Some(InventoryEvent::Close);
                } else if input.pressed(Button::Cancel) {
                    // Back to whichever chooser queued this action.
                    self.focus = match self.pending.take() {
                        Some(Pending::Move(_)) => Focus::MoveList,
                        _ => Focus::ItemList,
                    };
                } else if input.pressed(Button::Up) {
                    self.target_cursor = (self.target_cursor + members - 1) % members;
                } else if input.pressed(Button::Down) {
                    self.target_cursor = (self.target_cursor + 1) % members;
                } else if input.pressed(Button::Confirm) {
                    self.apply_pending(party, reg);
                }
            }
        }
        None
    }

    /// Resolve the queued item/move on the chosen target, set the feedback line,
    /// and return to the chooser it came from.
    fn apply_pending(&mut self, party: &mut Party, reg: &Registry) {
        let target = self.target_cursor;
        let (result, back) = match self.pending.take() {
            Some(Pending::Item(idx)) => {
                (party.use_item_in_field(reg, idx, target), Focus::ItemList)
            }
            Some(Pending::Move(id)) => (
                party.use_heal_skill_in_field(reg, self.member, &id, target),
                Focus::MoveList,
            ),
            None => return,
        };
        let name = party
            .members
            .get(target)
            .map(|m| m.name.clone())
            .unwrap_or_default();
        self.message = Some((field_use_message(&result, &name), 1.6));
        self.list_cursor = 0;
        self.focus = back;
    }

    /// The bag chooser for the selected slot: a leading [`UNEQUIP`] sentinel, then
    /// the bag indices of items that fit the slot.
    fn choices(&self, party: &Party, reg: &Registry) -> Vec<usize> {
        let mut v = vec![UNEQUIP];
        v.extend(party.bag_indices_for(reg, self.slot));
        v
    }

    /// Stash indices of the items usable from the field (restorative ones).
    fn field_items(&self, party: &Party, reg: &Registry) -> Vec<usize> {
        party
            .items
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                reg.item(&s.id)
                    .map(|it| it.effect.usable_in_field())
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn draw(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgb(14, 14, 26));

        // Header.
        r.draw_text("PARTY MENU", 8.0, 6.0, 1.2, color::rgb(255, 226, 120));
        let gold = format!("GOLD {}", party.gold);
        let gw = r.text_width(&gold, 1.0);
        r.draw_text(
            &gold,
            virtual_w() - gw - 8.0,
            7.0,
            1.0,
            color::rgb(240, 220, 130),
        );

        self.draw_members(r, party, reg);
        self.draw_side(r, party, reg);
        if self.focus == Focus::Action {
            self.draw_action_menu(r);
        }
        self.draw_message(r);
        self.draw_hint(r);
    }

    /// Left panel: each member with HP/MP and their weapon and armor.
    fn draw_members(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        let (px, py, pw) = (6.0, 20.0, 176.0);
        let rows = party.members.len().max(1);
        let ph = rows as f32 * 34.0 + 6.0;
        r.draw_rect(px, py, pw, ph, color::rgba(10, 12, 24, 230));
        r.draw_rect_outline(px, py, pw, ph, 1.0, color::rgba(80, 90, 140, 255));

        let picking_target = self.focus == Focus::Target;

        for (i, m) in party.members.iter().enumerate() {
            let y = py + 5.0 + i as f32 * 34.0;
            let selected = i == self.member;
            // While picking a target, the green cursor tracks the recipient row.
            let targeted = picking_target && i == self.target_cursor;
            if targeted {
                r.draw_rect(
                    px + 1.0,
                    y - 1.0,
                    pw - 2.0,
                    9.0,
                    color::rgba(30, 70, 40, 255),
                );
            }
            let name_col = if targeted {
                color::rgb(150, 240, 160)
            } else if selected {
                color::rgb(255, 240, 150)
            } else {
                color::WHITE
            };
            if targeted {
                r.draw_text("+", px + 3.0, y, 1.0, color::rgb(150, 240, 160));
            } else if selected {
                r.draw_text(">", px + 3.0, y, 1.0, color::rgb(255, 240, 150));
            }
            r.draw_text(&m.name, px + 12.0, y, 1.0, name_col);

            // Live HP / MP, so the effect of a heal is visible at a glance.
            let vitals = format!(
                "HP {}/{}  MP {}/{}",
                m.hp, m.stats.max_hp, m.mp, m.stats.max_mp
            );
            let vw = r.text_width(&vitals, 1.0);
            r.draw_text(
                &vitals,
                px + pw - vw - 4.0,
                y,
                1.0,
                color::rgb(150, 200, 160),
            );

            for (row, slot) in [EquipSlot::Weapon, EquipSlot::Armor]
                .into_iter()
                .enumerate()
            {
                let sy = y + 10.0 + row as f32 * 10.0;
                // Highlight the slot the cursor is on for the selected member while
                // equipping (PARTY / ACTION / BAG focuses).
                let equipping = matches!(self.focus, Focus::Party | Focus::Action | Focus::Bag);
                let active = selected && equipping && slot == self.slot;
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

    /// Right panel: its contents follow the current focus — a bag+stash overview,
    /// the equip chooser, or the item / move chooser.
    fn draw_side(&self, r: &mut Renderer, party: &Party, reg: &Registry) {
        let p = Panel {
            x: 188.0,
            y: 20.0,
            w: 126.0,
            h: 132.0,
        };
        r.draw_rect(p.x, p.y, p.w, p.h, color::rgba(10, 12, 24, 230));
        r.draw_rect_outline(p.x, p.y, p.w, p.h, 1.0, color::rgba(80, 90, 140, 255));

        match self.focus {
            Focus::Party | Focus::Action | Focus::Target => self.draw_overview(r, party, reg, p),
            Focus::Bag => self.draw_bag_chooser(r, party, reg, p),
            Focus::ItemList => self.draw_item_chooser(r, party, reg, p),
            Focus::MoveList => self.draw_move_chooser(r, party, reg, p),
        }
    }

    /// The bag + item-stash overview shown while browsing heroes.
    fn draw_overview(&self, r: &mut Renderer, party: &Party, reg: &Registry, p: Panel) {
        let (px, py, pw) = (p.x, p.y, p.w);
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
            for (row, id) in party.bag.iter().enumerate().take(5) {
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

        let iy = py + 74.0;
        r.draw_rect(
            px + 3.0,
            iy - 2.0,
            pw - 6.0,
            1.0,
            color::rgba(80, 90, 140, 255),
        );
        r.draw_text("ITEMS", px + 4.0, iy, 1.0, color::rgb(200, 190, 150));
        if party.items.is_empty() {
            r.draw_text(
                "(NONE)",
                px + 6.0,
                iy + 12.0,
                1.0,
                color::rgb(120, 120, 140),
            );
        } else {
            for (row, stack) in party.items.iter().enumerate().take(5) {
                let name = reg
                    .item(&stack.id)
                    .map(|it| it.name.as_str())
                    .unwrap_or(stack.id.as_str());
                r.draw_text(
                    &format!("{name}  x{}", stack.count),
                    px + 6.0,
                    iy + 12.0 + row as f32 * 10.0,
                    1.0,
                    color::rgb(210, 205, 190),
                );
            }
        }
    }

    /// The slot-filtered equip chooser (UNEQUIP + fitting bag gear).
    fn draw_bag_chooser(&self, r: &mut Renderer, party: &Party, reg: &Registry, p: Panel) {
        let (px, py, pw, ph) = (p.x, p.y, p.w, p.h);
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
            if row == self.bag_cursor {
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

    /// The restorative-item chooser.
    fn draw_item_chooser(&self, r: &mut Renderer, party: &Party, reg: &Registry, p: Panel) {
        let (px, py, pw, ph) = (p.x, p.y, p.w, p.h);
        r.draw_text(
            "USE ITEM",
            px + 4.0,
            py + 4.0,
            1.0,
            color::rgb(255, 210, 120),
        );
        let items = self.field_items(party, reg);
        if items.is_empty() {
            r.draw_text(
                "(NO USABLE ITEMS)",
                px + 6.0,
                py + 18.0,
                1.0,
                color::rgb(120, 120, 140),
            );
            return;
        }
        for (row, &idx) in items.iter().enumerate() {
            let y = py + 16.0 + row as f32 * 10.0;
            if y > py + ph - 20.0 {
                break;
            }
            if row == self.list_cursor {
                r.draw_rect(
                    px + 2.0,
                    y - 1.0,
                    pw - 4.0,
                    9.0,
                    color::rgba(60, 70, 120, 255),
                );
            }
            if let Some(stack) = party.items.get(idx) {
                let name = reg
                    .item(&stack.id)
                    .map(|it| it.name.clone())
                    .unwrap_or_else(|| stack.id.clone());
                r.draw_text(
                    &format!("{name}  x{}", stack.count),
                    px + 6.0,
                    y,
                    1.0,
                    color::WHITE,
                );
            }
        }
        // Effect summary of the highlighted item.
        if let Some(stack) = items
            .get(self.list_cursor)
            .and_then(|&i| party.items.get(i))
        {
            if let Some(it) = reg.item(&stack.id) {
                r.draw_text(
                    &crate::battle::item_effect_summary(reg, it),
                    px + 4.0,
                    py + ph - 10.0,
                    1.0,
                    color::rgb(150, 220, 160),
                );
            }
        }
    }

    /// The healing-move chooser for the selected hero.
    fn draw_move_chooser(&self, r: &mut Renderer, party: &Party, reg: &Registry, p: Panel) {
        let (px, py, pw, ph) = (p.x, p.y, p.w, p.h);
        let caster_mp = party.members.get(self.member).map(|m| m.mp).unwrap_or(0);
        r.draw_text(
            "USE MOVE",
            px + 4.0,
            py + 4.0,
            1.0,
            color::rgb(255, 210, 120),
        );
        let moves = party.field_heal_skills(reg, self.member);
        if moves.is_empty() {
            r.draw_text(
                "(NO HEALING MOVES)",
                px + 6.0,
                py + 18.0,
                1.0,
                color::rgb(120, 120, 140),
            );
            return;
        }
        for (row, id) in moves.iter().enumerate() {
            let y = py + 16.0 + row as f32 * 10.0;
            if y > py + ph - 20.0 {
                break;
            }
            if row == self.list_cursor {
                r.draw_rect(
                    px + 2.0,
                    y - 1.0,
                    pw - 4.0,
                    9.0,
                    color::rgba(60, 70, 120, 255),
                );
            }
            let Some(def) = reg.skill(id) else { continue };
            // Grey out moves the caster can't currently afford.
            let col = if caster_mp >= def.mp_cost {
                color::WHITE
            } else {
                color::rgb(120, 120, 130)
            };
            r.draw_text(&def.name, px + 6.0, y, 1.0, col);
            let cost = format!("{}MP", def.mp_cost);
            let cw = r.text_width(&cost, 1.0);
            r.draw_text(&cost, px + pw - cw - 6.0, y, 1.0, color::rgb(150, 180, 240));
        }
        if let Some(def) = moves.get(self.list_cursor).and_then(|id| reg.skill(id)) {
            r.draw_text(
                &def.description,
                px + 4.0,
                py + ph - 10.0,
                1.0,
                color::rgb(150, 220, 160),
            );
        }
    }

    /// The chosen hero's action menu, drawn as a small popup.
    fn draw_action_menu(&self, r: &mut Renderer) {
        let (mx, my, mw) = (66.0, 40.0, 84.0);
        let mh = ACTIONS.len() as f32 * 12.0 + 8.0;
        r.draw_rect(mx, my, mw, mh, color::rgba(18, 20, 40, 244));
        r.draw_rect_outline(mx, my, mw, mh, 1.0, color::rgba(120, 130, 190, 255));
        for (i, label) in ACTIONS.iter().enumerate() {
            let y = my + 5.0 + i as f32 * 12.0;
            if i == self.action_cursor {
                r.draw_rect(
                    mx + 2.0,
                    y - 1.0,
                    mw - 4.0,
                    11.0,
                    color::rgba(60, 70, 120, 255),
                );
                r.draw_text(">", mx + 4.0, y, 1.0, color::rgb(255, 240, 170));
            }
            r.draw_text(label, mx + 12.0, y, 1.0, color::WHITE);
        }
    }

    fn draw_message(&self, r: &mut Renderer) {
        if let Some((text, _)) = &self.message {
            r.draw_rect(
                0.0,
                VIRTUAL_H - 24.0,
                virtual_w(),
                11.0,
                color::rgba(10, 8, 16, 210),
            );
            r.draw_text_centered(
                text,
                virtual_w() / 2.0,
                VIRTUAL_H - 23.0,
                1.0,
                color::rgb(255, 230, 140),
            );
        }
    }

    fn draw_hint(&self, r: &mut Renderer) {
        let hint = match self.focus {
            Focus::Party => "MOVE: PICK HERO/SLOT   CONFIRM: OPEN   MENU/CANCEL: CLOSE",
            Focus::Action => "UP/DOWN: ACTION   CONFIRM: CHOOSE   CANCEL: BACK",
            Focus::Bag => "MOVE: PICK GEAR   CONFIRM: EQUIP   CANCEL: BACK",
            Focus::ItemList => "MOVE: PICK ITEM   CONFIRM: USE   CANCEL: BACK",
            Focus::MoveList => "MOVE: PICK MOVE   CONFIRM: CAST   CANCEL: BACK",
            Focus::Target => "UP/DOWN: PICK TARGET   CONFIRM: USE   CANCEL: BACK",
        };
        r.draw_text_centered(
            hint,
            virtual_w() / 2.0,
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

/// Turn a field item/move result into a one-line feedback message.
fn field_use_message(result: &FieldUse, target: &str) -> String {
    match result {
        FieldUse::Restored { hp, mp } => {
            let mut parts = Vec::new();
            if *hp > 0 {
                parts.push(format!("+{hp} HP"));
            }
            if *mp > 0 {
                parts.push(format!("+{mp} MP"));
            }
            if parts.is_empty() {
                format!("{target}: NO EFFECT")
            } else {
                format!("{target}  {}", parts.join("  "))
            }
        }
        FieldUse::NoEffect => "NO EFFECT".to_string(),
        FieldUse::NotEnoughMp => "NOT ENOUGH MP".to_string(),
    }
}
