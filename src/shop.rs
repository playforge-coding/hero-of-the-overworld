//! The shop: a small interior room you step into from the overworld to buy gear.
//!
//! A shop is pure data (see [`crate::data::ShopDef`]): a name, a greeting, the
//! wares for sale, and the wall the keeper faces. Entering builds a fixed
//! wood-floored, stone-walled room with the keeper standing at a counter; the
//! only doorway is the wall the keeper faces, so **you leave the way the keeper
//! is looking**. Walk up to the counter and press Confirm to open the buy menu,
//! pick an item and a party member, and Confirm again to purchase-and-equip
//! (gold is the only limiter — stock is unlimited).
//!
//! Adding a shop is a RON edit plus a [`crate::data::ShopSpawn`] placement in an
//! overworld screen — no engine change, the same "extensible content" contract
//! the rest of the game follows.

use glam::Vec2;

use crate::data::{EquipSlot, Registry, ShopDef, ShopFacing};
use crate::input::{Button, Input};
use crate::overworld::{leader_walk, walk_src, Facing, TILE};
use crate::party::Party;
use crate::renderer::{color, virtual_w, Renderer, TextureHandle, VIRTUAL_H};
use crate::util::TextureCache;

/// Interior size in tiles. 20x11 at 16px is 320x176 — the whole virtual screen,
/// so the room needs no scrolling camera.
const W: usize = 20;
const H: usize = 11;

const PLAYER_SPEED: f32 = 62.0;
/// Half-extents of the player's (feet) collision box against walls.
const HALF: Vec2 = Vec2::new(5.0, 4.0);
/// How close (px) the player must stand to the counter to open the menu.
const COUNTER_REACH: f32 = 30.0;

/// Horizontal offset that centres the fixed-width room in the (possibly wider)
/// virtual canvas, so on landscape phones the shop stays centred instead of
/// hugging the left edge. Zero at the classic 16:9 width. Purely presentational
/// — collision and the doorway stay in the room's local coordinates.
fn room_dx() -> f32 {
    ((virtual_w() - W as f32 * TILE) / 2.0).max(0.0)
}

/// What a [`Shop::update`] wants the game to do next.
pub enum ShopEvent {
    /// The player walked out the doorway — return to the overworld.
    Exit,
}

/// One purchasable line, resolved from the registry for display.
struct StockEntry {
    id: String,
    name: String,
    price: i32,
    slot: EquipSlot,
    /// Short "ATK+6 CRIT+6" style summary of what the item grants.
    summary: String,
    description: String,
}

pub struct Shop {
    name: String,
    greeting: String,
    stock: Vec<StockEntry>,

    // Interior.
    floor_tex: TextureHandle,
    wall_tex: TextureHandle,
    keeper_tex: TextureHandle,
    player_tex: TextureHandle,
    walls: Vec<bool>,
    /// Doorway tiles (carved from the wall); walking onto one leaves the shop.
    exit_tiles: Vec<(usize, usize)>,
    keeper_pos: Vec2,

    // Player actor.
    player: Vec2,
    player_facing: Facing,
    walk_t: f32,
    walk: crate::data::OverworldWalk,

    // Buy menu.
    menu_open: bool,
    item_cursor: usize,
    member_cursor: usize,
    /// Transient purchase feedback: (text, seconds remaining).
    message: Option<(String, f32)>,
    time: f32,
}

impl Shop {
    /// Build the runtime for `def`, resolving textures and stock up front.
    pub fn new(
        r: &mut Renderer,
        cache: &mut TextureCache,
        reg: &Registry,
        party: &Party,
        def: &ShopDef,
    ) -> Self {
        let stock = def
            .stock
            .iter()
            .filter_map(|s| {
                let item = reg.equipment(&s.item)?;
                Some(StockEntry {
                    id: item.id.clone(),
                    name: item.name.clone(),
                    price: s.price,
                    slot: item.slot,
                    summary: summarize(item),
                    description: item.description.clone(),
                })
            })
            .collect();

        let layout = Layout::build(def.facing);
        let walk = leader_walk(reg, party);
        let player_tex = cache.get(r, &walk.texture);

        Shop {
            name: def.name.clone(),
            greeting: def
                .greeting
                .clone()
                .unwrap_or_else(|| "WELCOME! TAKE A LOOK.".to_string()),
            stock,
            floor_tex: cache.get(r, "wood"),
            wall_tex: cache.get(r, "stone"),
            keeper_tex: cache.get(r, "shopkeeper"),
            player_tex,
            walls: layout.walls,
            exit_tiles: layout.exit_tiles,
            keeper_pos: layout.keeper_pos,
            player: layout.player_pos,
            player_facing: layout.player_facing,
            walk_t: 0.0,
            walk,
            menu_open: false,
            item_cursor: 0,
            member_cursor: 0,
            message: None,
            time: 0.0,
        }
    }

    pub fn update(&mut self, input: &Input, party: &mut Party, dt: f32) -> Option<ShopEvent> {
        self.time += dt;
        if let Some((_, t)) = &mut self.message {
            *t -= dt;
            if *t <= 0.0 {
                self.message = None;
            }
        }

        if self.menu_open {
            self.update_menu(input, party);
            return None;
        }

        // --- Walking the interior ------------------------------------------
        let mut mv = Vec2::ZERO;
        if input.held(Button::Left) {
            mv.x -= 1.0;
        }
        if input.held(Button::Right) {
            mv.x += 1.0;
        }
        if input.held(Button::Up) {
            mv.y -= 1.0;
        }
        if input.held(Button::Down) {
            mv.y += 1.0;
        }
        if mv != Vec2::ZERO {
            self.player_facing = facing_of(mv);
            let delta = mv.normalize() * PLAYER_SPEED * dt;
            self.player = self.slide(self.player, delta);
            self.walk_t += dt;
        } else {
            self.walk_t = 0.0;
        }

        // Stepping into the doorway leaves the shop.
        let (pc, pr) = self.tile_of(self.player);
        if self.exit_tiles.iter().any(|&(c, rr)| c == pc && rr == pr) {
            return Some(ShopEvent::Exit);
        }

        // Walk up to the counter and confirm to browse.
        if input.pressed(Button::Confirm)
            && (self.keeper_pos - self.player).length() < COUNTER_REACH
        {
            self.menu_open = true;
            self.member_cursor = self
                .member_cursor
                .min(party.members.len().saturating_sub(1));
        }

        None
    }

    /// Buy-menu controls: pick an item and a member, purchase-and-equip.
    fn update_menu(&mut self, input: &Input, party: &mut Party) {
        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            self.menu_open = false;
            return;
        }
        if self.stock.is_empty() {
            return;
        }
        let n = self.stock.len();
        if input.pressed(Button::Up) {
            self.item_cursor = (self.item_cursor + n - 1) % n;
        }
        if input.pressed(Button::Down) {
            self.item_cursor = (self.item_cursor + 1) % n;
        }
        let members = party.members.len().max(1);
        if input.pressed(Button::Left) {
            self.member_cursor = (self.member_cursor + members - 1) % members;
        }
        if input.pressed(Button::Right) {
            self.member_cursor = (self.member_cursor + 1) % members;
        }
        if input.pressed(Button::Confirm) {
            self.buy(party);
        }
    }

    /// Attempt to purchase the selected item for the selected member, setting
    /// the transient feedback line either way.
    fn buy(&mut self, party: &mut Party) {
        let entry = &self.stock[self.item_cursor];
        self.message = Some(match apply_purchase(party, self.member_cursor, entry) {
            PurchaseResult::Bought => {
                let who = party
                    .members
                    .get(self.member_cursor)
                    .map(|m| m.name.as_str())
                    .unwrap_or("");
                (format!("{who} EQUIPPED {}", entry.name), 1.8)
            }
            PurchaseResult::TooPoor => ("NOT ENOUGH GOLD".to_string(), 1.6),
            PurchaseResult::NoMember => return,
        });
    }

    // ---- Collision ----------------------------------------------------------

    fn solid_tile(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col as usize >= W || row as usize >= H {
            return true; // out of bounds is a wall
        }
        self.walls[row as usize * W + col as usize]
    }

    fn blocked(&self, c: Vec2) -> bool {
        [
            Vec2::new(c.x - HALF.x, c.y - HALF.y),
            Vec2::new(c.x + HALF.x, c.y - HALF.y),
            Vec2::new(c.x - HALF.x, c.y + HALF.y),
            Vec2::new(c.x + HALF.x, c.y + HALF.y),
        ]
        .iter()
        .any(|p| self.solid_tile((p.x / TILE).floor() as i32, (p.y / TILE).floor() as i32))
    }

    /// Move `pos` by `delta`, resolving each axis so the player slides on walls.
    fn slide(&self, pos: Vec2, delta: Vec2) -> Vec2 {
        let mut p = pos;
        let nx = Vec2::new(p.x + delta.x, p.y);
        if !self.blocked(nx) {
            p.x = nx.x;
        }
        let ny = Vec2::new(p.x, p.y + delta.y);
        if !self.blocked(ny) {
            p.y = ny.y;
        }
        p
    }

    fn tile_of(&self, p: Vec2) -> (usize, usize) {
        (
            (p.x / TILE).floor().clamp(0.0, (W - 1) as f32) as usize,
            (p.y / TILE).floor().clamp(0.0, (H - 1) as f32) as usize,
        )
    }

    // ---- Rendering ----------------------------------------------------------

    pub fn draw(&self, r: &mut Renderer, reg: &Registry, party: &Party) {
        r.set_clear_color(color::rgb(20, 14, 10));
        self.draw_room(r);
        self.draw_actors(r);
        if self.menu_open {
            self.draw_menu(r, reg, party);
        } else {
            self.draw_hud(r, party.gold);
        }
        if let Some((text, _)) = &self.message {
            let y = if self.menu_open {
                VIRTUAL_H - 22.0
            } else {
                14.0
            };
            r.draw_rect(0.0, y - 2.0, virtual_w(), 12.0, color::rgba(10, 8, 16, 200));
            r.draw_text_centered(text, virtual_w() / 2.0, y, 1.0, color::rgb(255, 230, 140));
        }
    }

    fn draw_room(&self, r: &mut Renderer) {
        let dx = room_dx();
        // On a wide screen the fixed-width room is centred, leaving side margins.
        // Tile wall across the whole canvas underneath first, so those margins
        // read as more stone wall (matching the room's own wall border) rather
        // than an empty gap. A no-op at the classic 16:9 width.
        if dx > 0.0 {
            let cols = (virtual_w() / TILE).ceil() as usize;
            let rows = (VIRTUAL_H / TILE).ceil() as usize;
            for row in 0..rows {
                for col in 0..cols {
                    let x = col as f32 * TILE;
                    let y = row as f32 * TILE;
                    r.draw_texture(self.wall_tex, x, y, TILE, TILE, color::WHITE);
                }
            }
        }
        for row in 0..H {
            for col in 0..W {
                let x = col as f32 * TILE + dx;
                let y = row as f32 * TILE;
                if self.walls[row * W + col] {
                    r.draw_texture(self.wall_tex, x, y, TILE, TILE, color::WHITE);
                } else {
                    r.draw_texture(self.floor_tex, x, y, TILE, TILE, color::WHITE);
                }
            }
        }
    }

    fn draw_actors(&self, r: &mut Renderer) {
        // Painter's order by feet-Y so whoever is lower draws on top.
        if self.keeper_pos.y <= self.player.y {
            self.draw_keeper(r);
            self.draw_player(r);
        } else {
            self.draw_player(r);
            self.draw_keeper(r);
        }
    }

    fn draw_keeper(&self, r: &mut Renderer) {
        let (tw, th) = r.texture_size(self.keeper_tex);
        let dw = tw as f32 * 2.0;
        let dh = th as f32 * 2.0;
        let dx = room_dx();
        // A wooden counter slab in front of the keeper.
        r.draw_rect(
            self.keeper_pos.x + dx - 18.0,
            self.keeper_pos.y - 1.0,
            36.0,
            6.0,
            color::rgb(96, 62, 34),
        );
        r.draw_rect(
            self.keeper_pos.x + dx - 20.0,
            self.keeper_pos.y - 2.0,
            36.0,
            3.0,
            color::rgba(0, 0, 0, 70),
        );
        r.draw_texture(
            self.keeper_tex,
            self.keeper_pos.x + dx - dw / 2.0,
            self.keeper_pos.y + HALF.y - dh - 4.0,
            dw,
            dh,
            color::WHITE,
        );
        // A hint above the keeper when the player is in reach and not browsing.
        if !self.menu_open
            && (self.keeper_pos - self.player).length() < COUNTER_REACH
            && (self.time * 2.0) as i32 % 2 == 0
        {
            r.draw_text_centered(
                "PRESS Z",
                self.keeper_pos.x + dx,
                self.keeper_pos.y - dh - 8.0,
                1.0,
                color::rgb(220, 240, 210),
            );
        }
    }

    fn draw_player(&self, r: &mut Renderer) {
        let src = walk_src(&self.walk, self.player_facing, self.walk_t);
        let dw = self.walk.draw_w;
        let dh = self.walk.draw_h;
        let dx = room_dx();
        r.draw_rect(
            self.player.x + dx - dw * 0.28,
            self.player.y - 1.0,
            dw * 0.56,
            4.0,
            color::rgba(0, 0, 0, 80),
        );
        let dest = [
            self.player.x + dx - dw / 2.0,
            self.player.y + HALF.y - dh,
            dw,
            dh,
        ];
        r.draw_sprite(self.player_tex, dest, src, false, color::WHITE);
    }

    fn draw_hud(&self, r: &mut Renderer, gold: i32) {
        r.draw_rect(0.0, 0.0, virtual_w(), 12.0, color::rgba(10, 8, 16, 200));
        r.draw_text(&self.name, 5.0, 2.0, 1.0, color::rgb(255, 226, 160));
        let g = format!("GOLD {gold}");
        let gw = r.text_width(&g, 1.0);
        r.draw_text(
            &g,
            virtual_w() - gw - 5.0,
            2.0,
            1.0,
            color::rgb(255, 220, 120),
        );
        if (self.time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered(
                "WALK TO THE COUNTER - OR OUT THE DOOR TO LEAVE",
                virtual_w() / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgba(210, 200, 180, 220),
            );
        }
    }

    /// The buy menu overlay: wares on the left, the selected item's detail and
    /// the member being outfitted on the right.
    fn draw_menu(&self, r: &mut Renderer, reg: &Registry, party: &Party) {
        // Dim the room and frame a panel.
        r.draw_rect(0.0, 0.0, virtual_w(), VIRTUAL_H, color::rgba(6, 5, 12, 210));
        let (px, py, pw, ph) = (8.0, 8.0, virtual_w() - 16.0, VIRTUAL_H - 16.0);
        r.draw_rect(px, py, pw, ph, color::rgba(18, 16, 30, 244));
        r.draw_rect_outline(px, py, pw, ph, 1.0, color::rgba(150, 120, 70, 255));

        // Header: shop name + gold on hand.
        r.draw_text(
            &self.name,
            px + 6.0,
            py + 5.0,
            1.2,
            color::rgb(255, 226, 160),
        );
        let g = format!("GOLD {}", party.gold);
        let gw = r.text_width(&g, 1.0);
        r.draw_text(
            &g,
            px + pw - gw - 6.0,
            py + 6.0,
            1.0,
            color::rgb(255, 220, 120),
        );
        r.draw_text(
            &self.greeting,
            px + 6.0,
            py + 20.0,
            1.0,
            color::rgba(190, 200, 210, 255),
        );

        // Left column: the wares.
        let list_x = px + 8.0;
        let mut y = py + 36.0;
        let split = px + pw * 0.52;
        for (i, e) in self.stock.iter().enumerate() {
            let selected = i == self.item_cursor;
            if selected {
                r.draw_rect(
                    px + 2.0,
                    y - 1.0,
                    split - px - 4.0,
                    11.0,
                    color::rgba(70, 60, 40, 255),
                );
            }
            let afford = party.gold >= e.price;
            let name_col = if !afford {
                color::rgb(120, 120, 130)
            } else if selected {
                color::rgb(255, 240, 170)
            } else {
                color::rgb(220, 220, 225)
            };
            r.draw_text(&e.name, list_x + 8.0, y, 1.0, name_col);
            if selected {
                r.draw_text(">", list_x, y, 1.0, color::rgb(255, 240, 170));
            }
            let price = format!("{}G", e.price);
            let prw = r.text_width(&price, 1.0);
            let price_col = if afford {
                color::rgb(255, 220, 120)
            } else {
                color::rgb(170, 110, 110)
            };
            r.draw_text(&price, split - prw - 6.0, y, 1.0, price_col);
            y += 12.0;
        }

        // Right column: detail of the selected item.
        if let Some(e) = self.stock.get(self.item_cursor) {
            let dx = split + 6.0;
            let mut dy = py + 36.0;
            let slot = match e.slot {
                EquipSlot::Weapon => "WEAPON",
                EquipSlot::Armor => "ARMOR",
            };
            r.draw_text(slot, dx, dy, 1.0, color::rgb(160, 200, 255));
            dy += 12.0;
            r.draw_text(&e.summary, dx, dy, 1.0, color::rgb(200, 230, 200));
            dy += 12.0;
            for line in wrap(&e.description, ((px + pw - 4.0 - dx) / 5.0) as usize) {
                r.draw_text(&line, dx, dy, 1.0, color::rgba(200, 200, 210, 255));
                dy += 10.0;
            }
        }

        // The party member being outfitted, with their current gear in this slot.
        let selected_slot = self.stock.get(self.item_cursor).map(|e| e.slot);
        let row_y = py + ph - 38.0;
        r.draw_rect(
            px + 2.0,
            row_y - 2.0,
            pw - 4.0,
            26.0,
            color::rgba(10, 8, 18, 255),
        );
        if let Some(m) = party.members.get(self.member_cursor) {
            let who = format!("< OUTFIT {} >", m.name);
            r.draw_text_centered(
                &who,
                virtual_w() / 2.0,
                row_y,
                1.0,
                color::rgb(255, 236, 180),
            );
            let cur_id = match selected_slot {
                Some(EquipSlot::Weapon) | None => m.weapon.as_deref(),
                Some(EquipSlot::Armor) => m.armor.as_deref(),
            };
            let cur = cur_id
                .and_then(|id| reg.equipment(id))
                .map(|it| it.name.clone())
                .unwrap_or_else(|| "(NONE)".to_string());
            r.draw_text_centered(
                &format!("NOW: {cur}"),
                virtual_w() / 2.0,
                row_y + 11.0,
                1.0,
                color::rgba(190, 200, 210, 255),
            );
        }

        r.draw_text_centered(
            "UP/DOWN ITEM   LEFT/RIGHT WHO   Z BUY   X BACK",
            virtual_w() / 2.0,
            py + ph - 10.0,
            1.0,
            color::rgba(180, 185, 205, 235),
        );
    }
}

// ---- Interior layout --------------------------------------------------------

/// The generated interior for one keeper facing: which tiles are walls, where
/// the doorway is, and where the keeper and player stand.
struct Layout {
    walls: Vec<bool>,
    exit_tiles: Vec<(usize, usize)>,
    keeper_pos: Vec2,
    player_pos: Vec2,
    player_facing: Facing,
}

impl Layout {
    fn build(facing: ShopFacing) -> Layout {
        let mut walls = vec![false; W * H];
        // Border ring of stone walls.
        for c in 0..W {
            walls[c] = true;
            walls[(H - 1) * W + c] = true;
        }
        for rr in 0..H {
            walls[rr * W] = true;
            walls[rr * W + (W - 1)] = true;
        }

        // Centered 2-wide doorway on the keeper's wall; the keeper stands one
        // tile in from the opposite wall, facing the door; the player enters
        // just inside the doorway, facing inward.
        let (cc0, cc1) = (W / 2 - 1, W / 2); // 9, 10 (exact horizontal center)
        let (cr0, cr1) = (H / 2 - 1, H / 2); // 4, 5
        let mid_x = (W / 2) as f32 * TILE; // 160 — between cc0 and cc1
        let mid_y = (H / 2) as f32 * TILE;

        let (exit_tiles, keeper_pos, player_pos, player_facing) = match facing {
            ShopFacing::Down => (
                vec![(cc0, H - 1), (cc1, H - 1)],
                Vec2::new(mid_x, tile_mid(1)),
                Vec2::new(mid_x, tile_mid(H - 3)),
                Facing::Up,
            ),
            ShopFacing::Up => (
                vec![(cc0, 0), (cc1, 0)],
                Vec2::new(mid_x, tile_mid(H - 2)),
                Vec2::new(mid_x, tile_mid(2)),
                Facing::Down,
            ),
            ShopFacing::Left => (
                vec![(0, cr0), (0, cr1)],
                Vec2::new(tile_mid(W - 2), mid_y),
                Vec2::new(tile_mid(2), mid_y),
                Facing::Right,
            ),
            ShopFacing::Right => (
                vec![(W - 1, cr0), (W - 1, cr1)],
                Vec2::new(tile_mid(1), mid_y),
                Vec2::new(tile_mid(W - 3), mid_y),
                Facing::Left,
            ),
        };

        // Carve the doorway out of the wall so the player can pass through.
        for &(c, rr) in &exit_tiles {
            walls[rr * W + c] = false;
        }

        Layout {
            walls,
            exit_tiles,
            keeper_pos,
            player_pos,
            player_facing,
        }
    }
}

// ---- Free helpers -----------------------------------------------------------

/// Outcome of [`apply_purchase`], used to pick the shop's feedback line.
#[derive(Debug, PartialEq, Eq)]
enum PurchaseResult {
    Bought,
    TooPoor,
    NoMember,
}

/// The pure buy-and-equip transaction: if `party` can afford `entry`, deduct the
/// gold and equip the item to member `member_idx` in its slot. No rendering or
/// UI state, so it is unit-testable without a live window.
fn apply_purchase(party: &mut Party, member_idx: usize, entry: &StockEntry) -> PurchaseResult {
    if party.members.get(member_idx).is_none() {
        return PurchaseResult::NoMember;
    }
    if party.gold < entry.price {
        return PurchaseResult::TooPoor;
    }
    party.gold -= entry.price;
    // Equip the newly-bought item and keep whatever it displaces in the party's
    // bag rather than discarding it (the inventory screen can re-equip it later).
    let member = &mut party.members[member_idx];
    let displaced = match entry.slot {
        EquipSlot::Weapon => member.weapon.replace(entry.id.clone()),
        EquipSlot::Armor => member.armor.replace(entry.id.clone()),
    };
    if let Some(old) = displaced {
        party.bag.push(old);
    }
    PurchaseResult::Bought
}

/// Pixel center of a tile index along one axis.
fn tile_mid(i: usize) -> f32 {
    (i as f32 + 0.5) * TILE
}

/// Dominant-axis facing for a movement vector (mirrors the overworld's).
fn facing_of(v: Vec2) -> Facing {
    if v.x.abs() > v.y.abs() {
        if v.x < 0.0 {
            Facing::Left
        } else {
            Facing::Right
        }
    } else if v.y < 0.0 {
        Facing::Up
    } else {
        Facing::Down
    }
}

/// Greedy word wrap to at most `max` characters per line (the font is
/// fixed-width, so character count maps directly to width).
fn wrap(text: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= max {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// A short "ATK+6 CRIT+6 ACC+4" summary of what a piece of equipment grants.
pub(crate) fn summarize(item: &crate::data::EquipmentDef) -> String {
    let mut parts: Vec<String> = Vec::new();
    let m = &item.mods;
    for (label, v) in [
        ("ATK", m.attack),
        ("DEF", m.defense),
        ("MAG", m.magic),
        ("SPD", m.speed),
    ] {
        if v != 0 {
            parts.push(format!("{label}{v:+}"));
        }
    }
    for (label, v) in [
        ("CRIT", item.crit),
        ("ACC", item.accuracy),
        ("EVA", item.evasion),
    ] {
        if v != 0 {
            parts.push(format!("{label}{v:+}"));
        }
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{EquipSlot, Registry, ShopFacing};
    use crate::party::Party;

    fn stock(id: &str, price: i32, slot: EquipSlot) -> StockEntry {
        StockEntry {
            id: id.to_string(),
            name: id.to_uppercase(),
            price,
            slot,
            summary: "-".to_string(),
            description: String::new(),
        }
    }

    /// Every keeper facing yields a sealed room: a solid border with exactly one
    /// 2-tile doorway carved from the wall on the keeper's side, and both the
    /// keeper and the entering player standing on open floor inside.
    #[test]
    fn interior_layout_is_sealed_with_one_door() {
        for facing in [
            ShopFacing::Down,
            ShopFacing::Up,
            ShopFacing::Left,
            ShopFacing::Right,
        ] {
            let l = Layout::build(facing);
            assert_eq!(l.walls.len(), W * H);
            assert_eq!(l.exit_tiles.len(), 2, "{facing:?}: door should be 2 tiles");

            // The doorway tiles are walkable; they sit on the outer wall ring.
            for &(c, r) in &l.exit_tiles {
                assert!(!l.walls[r * W + c], "{facing:?}: doorway not carved open");
                let on_border = c == 0 || c == W - 1 || r == 0 || r == H - 1;
                assert!(on_border, "{facing:?}: doorway must be on the outer wall");
            }

            // Keeper and player stand on interior floor, not in a wall.
            for pos in [l.keeper_pos, l.player_pos] {
                let c = (pos.x / TILE).floor() as usize;
                let r = (pos.y / TILE).floor() as usize;
                assert!(c < W && r < H, "{facing:?}: actor out of the room");
                assert!(!l.walls[r * W + c], "{facing:?}: actor stands in a wall");
            }

            // The player enters near the door, the keeper stands away from it.
            let door = l.exit_tiles[0];
            let door_px = Vec2::new(tile_mid(door.0), tile_mid(door.1));
            assert!(
                (l.player_pos - door_px).length() < (l.keeper_pos - door_px).length(),
                "{facing:?}: player should enter nearer the door than the keeper",
            );
        }
    }

    /// The exit doorway is on the wall the keeper faces (the requested mechanic:
    /// "leave the way the keeper is looking").
    #[test]
    fn door_is_on_the_keeper_facing_wall() {
        let cases = [
            (ShopFacing::Down, H - 1, true),
            (ShopFacing::Up, 0, true),
            (ShopFacing::Left, 0, false),
            (ShopFacing::Right, W - 1, false),
        ];
        for (facing, edge, is_row) in cases {
            let l = Layout::build(facing);
            for &(c, r) in &l.exit_tiles {
                let coord = if is_row { r } else { c };
                assert_eq!(coord, edge, "{facing:?}: door on the wrong wall");
            }
        }
    }

    /// Buying deducts gold and equips the item; a broke party is refused and its
    /// gold and gear are left untouched.
    #[test]
    fn purchase_equips_and_charges_only_when_affordable() {
        let reg = Registry::load();
        let mut party = Party::from_registry(&reg);
        party.gold = 100;
        let leader = 0;

        // Affordable weapon: gold drops, weapon swaps to the bought one.
        let sword = stock("stone_fists", 60, EquipSlot::Weapon);
        assert_eq!(
            apply_purchase(&mut party, leader, &sword),
            PurchaseResult::Bought
        );
        assert_eq!(party.gold, 40);
        assert_eq!(party.members[leader].weapon.as_deref(), Some("stone_fists"));

        // Too expensive: refused, and nothing changes.
        let robe = stock("travelers_robe", 999, EquipSlot::Armor);
        let armor_before = party.members[leader].armor.clone();
        assert_eq!(
            apply_purchase(&mut party, leader, &robe),
            PurchaseResult::TooPoor
        );
        assert_eq!(party.gold, 40);
        assert_eq!(party.members[leader].armor, armor_before);

        // Out-of-range member index is a no-op, not a panic.
        assert_eq!(
            apply_purchase(&mut party, 99, &sword),
            PurchaseResult::NoMember
        );
        assert_eq!(party.gold, 40);
    }
}
