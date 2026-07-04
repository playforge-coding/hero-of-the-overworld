//! A level: a set of connected *screens* (rooms) you walk between.
//!
//! Each screen is an ASCII tile grid from [`crate::data::ScreenDef`]. The player
//! walks freely in virtual pixels with per-axis tile collision (sliding along
//! walls), and a camera follows, clamped to the screen. Walking into a mid-edge
//! opening on a side that links to a neighbour flips to that screen — Zelda
//! style. Roaming enemies chase the player within an aggro radius; touching one
//! makes [`update`](Overworld::update) return [`Event::Battle`]. Pressing cancel
//! returns [`Event::ExitToMap`]. The player is faster than the enemies, so
//! encounters can be dodged.

use glam::Vec2;

use crate::data::{BattlerSprite, OverworldWalk, Registry};
use crate::input::{Button, Input};
use crate::party::Party;
use crate::renderer::{color, Color, Renderer, TextureHandle, VIRTUAL_H, VIRTUAL_W};
use crate::util::TextureCache;

/// Tile edge length in virtual pixels (matches the 16x16 tile art).
pub const TILE: f32 = 16.0;

const PLAYER_SPEED: f32 = 62.0;
const ENEMY_SPEED: f32 = 34.0;
/// Enemies within this many pixels start pursuing the player.
const AGGRO: f32 = 108.0;
/// Center-to-center distance at which an enemy triggers a battle.
const CONTACT: f32 = 11.0;
/// How close (px) the player must stand to a shop keeper to enter on Confirm.
const SHOP_REACH: f32 = 22.0;
/// Half-extents of the (feet) collision box used against solid tiles.
const HALF: Vec2 = Vec2::new(5.0, 4.0);
/// How close to an edge (px) counts as "in the opening" for a screen flip.
const EDGE: f32 = 1.0;

// ---- Tiles ------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
enum Tile {
    Grass,
    Water,
    Tree,
    Rock,
    Barricade,
}

impl Tile {
    fn from_char(c: char) -> Tile {
        match c {
            'T' => Tile::Tree,
            'R' => Tile::Rock,
            '~' => Tile::Water,
            '#' => Tile::Barricade,
            _ => Tile::Grass, // '.' / space / anything unknown = walkable grass
        }
    }

    /// Grass is the only walkable tile; everything else blocks movement.
    fn solid(self) -> bool {
        !matches!(self, Tile::Grass)
    }
}

// ---- Actors -----------------------------------------------------------------

#[derive(Copy, Clone)]
pub(crate) enum Facing {
    Down,
    Up,
    Left,
    Right,
}

#[derive(Copy, Clone)]
enum Dir {
    North,
    South,
    East,
    West,
}

struct Player {
    /// Ground-center position in world pixels (feet).
    pos: Vec2,
    facing: Facing,
    /// Seconds spent walking, for the walk-cycle frame; reset when idle.
    walk_t: f32,
    walk: OverworldWalk,
    tex: TextureHandle,
}

struct Enemy {
    pos: Vec2,
    home: Vec2,
    /// Encounter started when this enemy touches the player.
    encounter: String,
    walk: OverworldWalk,
    facing: Facing,
    tex: TextureHandle,
    /// Chase speed in px/s (from the enemy's `overworld_speed`, else the default).
    speed: f32,
    /// Recolour multiplied over the sprite so reskinned foes (green slimes, stony
    /// gargoyles) read differently on the map. White = no tint.
    tint: Color,
    /// Seconds spent chasing, for the walk-cycle frame; reset when idle.
    walk_t: f32,
    defeated: bool,
}

/// Identifies the enemy that started the current battle so it can be cleared.
pub struct Trigger {
    pub screen: usize,
    pub enemy: usize,
    pub encounter: String,
}

/// A shop doorway on a screen: a shopkeeper standing at `pos` who ushers the
/// player into the referenced shop when they walk up and press Confirm.
struct ShopEntrance {
    pos: Vec2,
    shop: String,
}

/// What an [`Overworld::update`] wants the game to do next.
pub enum Event {
    /// Start a battle (an enemy touched the player).
    Battle(Trigger),
    /// Enter the shop with this id (player walked up to a keeper and confirmed).
    EnterShop(String),
    /// Leave the level and go back to the map screen.
    ExitToMap,
}

// ---- Screens ----------------------------------------------------------------

struct Screen {
    tiles: Vec<Tile>,
    w: usize,
    h: usize,
    enemies: Vec<Enemy>,
    shops: Vec<ShopEntrance>,
    north: Option<usize>,
    south: Option<usize>,
    east: Option<usize>,
    west: Option<usize>,
}

impl Screen {
    fn neighbor(&self, dir: Dir) -> Option<usize> {
        match dir {
            Dir::North => self.north,
            Dir::South => self.south,
            Dir::East => self.east,
            Dir::West => self.west,
        }
    }

    fn open_at(&self, col: usize, row: usize) -> bool {
        !self.tiles[row * self.w + col].solid()
    }

    /// Row of the walkable opening in column `col` whose center is closest to
    /// `target_y` — used to line an arriving player up with the doorway they
    /// walked toward, rather than dumping them at the edge's midpoint.
    fn nearest_open_row(&self, col: usize, target_y: f32) -> usize {
        (0..self.h)
            .filter(|&r| self.open_at(col, r))
            .min_by(|&a, &b| {
                let da = ((a as f32 + 0.5) * TILE - target_y).abs();
                let db = ((b as f32 + 0.5) * TILE - target_y).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(self.h / 2)
    }

    /// Column of the walkable opening in row `row` closest to `target_x`.
    fn nearest_open_col(&self, row: usize, target_x: f32) -> usize {
        (0..self.w)
            .filter(|&c| self.open_at(c, row))
            .min_by(|&a, &b| {
                let da = ((a as f32 + 0.5) * TILE - target_x).abs();
                let db = ((b as f32 + 0.5) * TILE - target_x).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(self.w / 2)
    }
}

struct TileTex {
    /// Walkable base ground, drawn under every tile (per-level: grass/stone/…).
    ground: TextureHandle,
    /// Texture for solid `#` wall tiles (per-level: barricade/dark_wall).
    wall: TextureHandle,
    water: TextureHandle,
    tree: TextureHandle,
    rock: TextureHandle,
}

pub struct Overworld {
    name: String,
    screens: Vec<Screen>,
    current: usize,
    tex: TileTex,
    /// Sprite drawn for a shop keeper standing at an entrance.
    shop_tex: TextureHandle,
    player: Player,
    cam: Vec2,
    /// Brief window during which contact can't trigger a fight (post-battle or
    /// just after a screen transition).
    grace: f32,
    time: f32,
}

impl Overworld {
    /// Build the runtime for `reg.data.levels[level_idx]`.
    ///
    /// `defeated` restores saved progress: `defeated[screen][enemy]` marks an
    /// enemy already beaten this run (from a previous session or a re-entry), so
    /// clearing a level survives quitting mid-way. Pass an empty slice for a
    /// fresh, untouched level.
    pub fn new(
        r: &mut Renderer,
        cache: &mut TextureCache,
        reg: &Registry,
        party: &Party,
        level_idx: usize,
        defeated: &[Vec<bool>],
    ) -> Self {
        let level = &reg.data.levels[level_idx];

        // Ground/wall are per-level so each region reads as its own place
        // (grassy GREENWOOD, stony STONE PASS, dark DEMON FORTRESS).
        let tex = TileTex {
            ground: cache.get(r, level.ground.as_deref().unwrap_or("grass")),
            wall: cache.get(r, level.wall.as_deref().unwrap_or("barricade")),
            water: cache.get(r, "water"),
            tree: cache.get(r, "tree"),
            rock: cache.get(r, "rock"),
        };

        // The party leader walks the map. Use its overworld sprite, or synthesize
        // one from the battle sprite so any character can lead in a pinch.
        let walk = leader_walk(reg, party);
        let player_tex = cache.get(r, &walk.texture);

        // Shopkeepers standing at shop entrances share one small NPC sprite.
        let shop_tex = cache.get(r, "shopkeeper");

        let mut screens = Vec::new();
        for (screen_idx, sd) in level.screens.iter().enumerate() {
            let w = sd
                .map
                .iter()
                .map(|s| s.chars().count())
                .max()
                .unwrap_or(1)
                .max(1);
            let h = sd.map.len().max(1);
            let mut tiles = vec![Tile::Grass; w * h];
            for (ri, line) in sd.map.iter().enumerate() {
                for (ci, ch) in line.chars().enumerate() {
                    tiles[ri * w + ci] = Tile::from_char(ch);
                }
            }

            // Which enemies on this screen are already beaten (restored save).
            let screen_defeated = defeated.get(screen_idx);

            let mut enemies = Vec::new();
            for sp in &sd.spawns {
                let Some(enc) = reg.encounter(&sp.encounter) else {
                    log::warn!("spawn references unknown encounter '{}'", sp.encounter);
                    continue;
                };
                let Some(def) = enc.enemies.first().and_then(|id| reg.enemy(id)) else {
                    continue;
                };
                let ewalk = def
                    .overworld
                    .clone()
                    .unwrap_or_else(|| fallback_walk(&def.sprite));
                let etex = cache.get(r, &ewalk.texture);
                let pos = tile_center(sp.col, sp.row);
                let tint = def
                    .sprite
                    .tint
                    .map(|(cr, cg, cb)| color::rgb(cr, cg, cb))
                    .unwrap_or(color::WHITE);
                let was_defeated = screen_defeated
                    .and_then(|s| s.get(enemies.len()))
                    .copied()
                    .unwrap_or(false);
                enemies.push(Enemy {
                    pos,
                    home: pos,
                    encounter: sp.encounter.clone(),
                    walk: ewalk,
                    facing: Facing::Down,
                    tex: etex,
                    speed: def.overworld_speed.unwrap_or(ENEMY_SPEED),
                    tint,
                    walk_t: 0.0,
                    defeated: was_defeated,
                });
            }

            // Shop doorways: a keeper standing at each placed tile.
            let shops = sd
                .shops
                .iter()
                .map(|sp| ShopEntrance {
                    pos: tile_center(sp.col, sp.row),
                    shop: sp.shop.clone(),
                })
                .collect();

            screens.push(Screen {
                tiles,
                w,
                h,
                enemies,
                shops,
                north: sd.north,
                south: sd.south,
                east: sd.east,
                west: sd.west,
            });
        }

        let current = level.start_screen.min(screens.len().saturating_sub(1));
        let player = Player {
            pos: tile_center(level.start.0, level.start.1),
            facing: Facing::Down,
            walk_t: 0.0,
            walk,
            tex: player_tex,
        };

        let mut ow = Overworld {
            name: level.name.clone(),
            screens,
            current,
            tex,
            shop_tex,
            player,
            cam: Vec2::ZERO,
            grace: 0.0,
            time: 0.0,
        };
        ow.center_camera();
        ow
    }

    fn cur(&self) -> &Screen {
        &self.screens[self.current]
    }

    /// True once every enemy in every screen has been defeated.
    pub fn all_cleared(&self) -> bool {
        self.screens
            .iter()
            .all(|s| s.enemies.iter().all(|e| e.defeated))
    }

    /// Snapshot of which enemies are beaten, as `screens[s][e]`, for saving. The
    /// shape mirrors the `defeated` argument to [`Overworld::new`], so a
    /// save→load round-trip restores the exact same field state.
    pub fn defeated_state(&self) -> Vec<Vec<bool>> {
        self.screens
            .iter()
            .map(|s| s.enemies.iter().map(|e| e.defeated).collect())
            .collect()
    }

    fn tile(&self, col: i32, row: i32) -> Tile {
        let s = self.cur();
        if col < 0 || row < 0 || col as usize >= s.w || row as usize >= s.h {
            return Tile::Tree; // out of bounds reads as solid
        }
        s.tiles[row as usize * s.w + col as usize]
    }

    /// True if the collision box centered at `c` overlaps any solid tile.
    fn blocked(&self, c: Vec2) -> bool {
        let corners = [
            Vec2::new(c.x - HALF.x, c.y - HALF.y),
            Vec2::new(c.x + HALF.x, c.y - HALF.y),
            Vec2::new(c.x - HALF.x, c.y + HALF.y),
            Vec2::new(c.x + HALF.x, c.y + HALF.y),
        ];
        corners.iter().any(|p| {
            self.tile((p.x / TILE).floor() as i32, (p.y / TILE).floor() as i32)
                .solid()
        })
    }

    /// Move `pos` by `delta`, resolving each axis independently so the mover
    /// slides along walls instead of sticking.
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

    /// Called by the game when a battle ends. On victory the enemy is cleared;
    /// otherwise every enemy in its screen retreats home so it can't insta-fight.
    pub fn resolve_battle(&mut self, trigger: &Trigger, won: bool) {
        if let Some(screen) = self.screens.get_mut(trigger.screen) {
            if won {
                if let Some(e) = screen.enemies.get_mut(trigger.enemy) {
                    e.defeated = true;
                }
            } else {
                for e in &mut screen.enemies {
                    e.pos = e.home;
                }
            }
        }
        self.grace = 0.7;
    }

    pub fn update(&mut self, input: &Input, dt: f32) -> Option<Event> {
        self.time += dt;
        if self.grace > 0.0 {
            self.grace = (self.grace - dt).max(0.0);
        }

        if input.pressed(Button::Cancel) || input.pressed(Button::Menu) {
            return Some(Event::ExitToMap);
        }

        // --- Player ---------------------------------------------------------
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
            self.player.facing = facing_of(mv);
            let delta = mv.normalize() * PLAYER_SPEED * dt;
            self.player.pos = self.slide(self.player.pos, delta);
            self.player.walk_t += dt;
        } else {
            self.player.walk_t = 0.0;
        }

        // A screen flip may relocate the player and change the current screen.
        self.check_transition();

        // --- Shop entrance: walk up to a keeper and confirm to go inside ----
        if input.pressed(Button::Confirm) {
            let p = self.player.pos;
            if let Some(sh) = self
                .cur()
                .shops
                .iter()
                .find(|s| (s.pos - p).length() < SHOP_REACH)
            {
                return Some(Event::EnterShop(sh.shop.clone()));
            }
        }

        // --- Enemies chase (current screen only) ----------------------------
        let target = self.player.pos;
        let Screen {
            tiles,
            enemies,
            w,
            h,
            ..
        } = &mut self.screens[self.current];
        let (sw, sh) = (*w, *h);
        for e in enemies.iter_mut() {
            if e.defeated {
                continue;
            }
            let to = target - e.pos;
            let dist = to.length();
            if dist > 0.5 && dist < AGGRO {
                let delta = to / dist * e.speed * dt;
                e.facing = facing_of(delta);
                e.pos = slide_on(tiles, sw, sh, e.pos, delta);
                e.walk_t += dt;
            } else {
                e.walk_t = 0.0;
            }
        }

        self.update_camera();

        // --- Contact --------------------------------------------------------
        if self.grace <= 0.0 {
            let idx = self.current;
            for (i, e) in self.screens[idx].enemies.iter().enumerate() {
                if !e.defeated && (e.pos - self.player.pos).length() < CONTACT {
                    return Some(Event::Battle(Trigger {
                        screen: idx,
                        enemy: i,
                        encounter: e.encounter.clone(),
                    }));
                }
            }
        }
        None
    }

    /// If the player has walked into an edge opening that links elsewhere, flip
    /// to the neighbouring screen and place them at the opposite edge.
    fn check_transition(&mut self) {
        let s = self.cur();
        let map = Vec2::new(s.w as f32 * TILE, s.h as f32 * TILE);
        let p = self.player.pos;
        let exit = if p.x <= HALF.x + EDGE && s.west.is_some() {
            Some(Dir::West)
        } else if p.x >= map.x - HALF.x - EDGE && s.east.is_some() {
            Some(Dir::East)
        } else if p.y <= HALF.y + EDGE && s.north.is_some() {
            Some(Dir::North)
        } else if p.y >= map.y - HALF.y - EDGE && s.south.is_some() {
            Some(Dir::South)
        } else {
            None
        };

        let Some(dir) = exit else { return };
        let Some(target) = s.neighbor(dir) else {
            return;
        };
        if target >= self.screens.len() {
            return;
        }

        // Remember where along the shared edge the player left, so the new
        // screen can line them up with the doorway nearest that spot.
        let exit_along = p;

        self.current = target;
        let ns = self.cur();
        self.player.pos = entry_pos(ns, dir, exit_along);
        self.player.facing = match dir {
            Dir::East => Facing::Right,
            Dir::West => Facing::Left,
            Dir::North => Facing::Up,
            Dir::South => Facing::Down,
        };
        // Don't insta-battle on arrival, and snap the camera to the new screen.
        self.grace = self.grace.max(0.35);
        self.center_camera();
    }

    fn center_camera(&mut self) {
        self.cam = self.clamp_cam(self.player.pos - Vec2::new(VIRTUAL_W, VIRTUAL_H) * 0.5);
    }

    fn update_camera(&mut self) {
        // Ease toward the player-centered target for a soft follow.
        let target = self.clamp_cam(self.player.pos - Vec2::new(VIRTUAL_W, VIRTUAL_H) * 0.5);
        self.cam += (target - self.cam) * 0.18;
    }

    fn clamp_cam(&self, c: Vec2) -> Vec2 {
        let s = self.cur();
        let map = Vec2::new(s.w as f32 * TILE, s.h as f32 * TILE);
        let clamp_axis = |v: f32, screen: f32, world: f32| {
            if world <= screen {
                (world - screen) * 0.5 // center a small map
            } else {
                v.clamp(0.0, world - screen)
            }
        };
        Vec2::new(
            clamp_axis(c.x, VIRTUAL_W, map.x),
            clamp_axis(c.y, VIRTUAL_H, map.y),
        )
    }

    // ---- Rendering ----------------------------------------------------------

    pub fn draw(&self, r: &mut Renderer) {
        r.set_clear_color(color::rgb(24, 30, 22));
        self.draw_tiles(r);
        self.draw_actors(r);
        self.draw_hud(r);
    }

    fn draw_tiles(&self, r: &mut Renderer) {
        let s = self.cur();
        let cam = self.cam;
        // Only iterate the tiles overlapping the viewport.
        let c0 = (cam.x / TILE).floor().max(0.0) as usize;
        let r0 = (cam.y / TILE).floor().max(0.0) as usize;
        let c1 = (((cam.x + VIRTUAL_W) / TILE).ceil() as usize).min(s.w);
        let r1 = (((cam.y + VIRTUAL_H) / TILE).ceil() as usize).min(s.h);

        for row in r0..r1 {
            for col in c0..c1 {
                let x = col as f32 * TILE - cam.x;
                let y = row as f32 * TILE - cam.y;
                let t = s.tiles[row * s.w + col];
                // The level's ground texture is the base under everything.
                r.draw_texture(self.tex.ground, x, y, TILE, TILE, color::WHITE);
                match t {
                    Tile::Grass => {}
                    Tile::Water => r.draw_texture(self.tex.water, x, y, TILE, TILE, color::WHITE),
                    Tile::Barricade => {
                        r.draw_texture(self.tex.wall, x, y, TILE, TILE, color::WHITE)
                    }
                    Tile::Tree => blit_object(r, self.tex.tree, x, y),
                    Tile::Rock => blit_object(r, self.tex.rock, x, y),
                }
            }
        }
    }

    fn draw_actors(&self, r: &mut Renderer) {
        // Painter's order: sort everything on the field by feet-Y.
        enum Item {
            Player,
            Enemy(usize),
            Shop(usize),
        }
        let mut items: Vec<(f32, Item)> = vec![(self.player.pos.y, Item::Player)];
        for (i, e) in self.cur().enemies.iter().enumerate() {
            if !e.defeated {
                items.push((e.pos.y, Item::Enemy(i)));
            }
        }
        for (i, s) in self.cur().shops.iter().enumerate() {
            items.push((s.pos.y, Item::Shop(i)));
        }
        items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, item) in &items {
            match item {
                Item::Player => self.draw_player(r),
                Item::Enemy(i) => self.draw_enemy(r, &self.cur().enemies[*i]),
                Item::Shop(i) => self.draw_shop_entrance(r, &self.cur().shops[*i]),
            }
        }
    }

    /// A shop keeper standing at a doorway, with a "SHOP" banner and a "PRESS Z"
    /// prompt once the player is close enough to enter.
    fn draw_shop_entrance(&self, r: &mut Renderer, s: &ShopEntrance) {
        let (tw, th) = r.texture_size(self.shop_tex);
        // Draw the small keeper sprite at roughly double size so it reads on the
        // field next to the ~20px hero/enemy sprites.
        let dw = tw as f32 * 1.6;
        let dh = th as f32 * 1.6;
        let sx = s.pos.x - self.cam.x;
        let sy = s.pos.y - self.cam.y;
        r.draw_rect(
            sx - dw * 0.32,
            sy - 1.0,
            dw * 0.64,
            3.0,
            color::rgba(0, 0, 0, 80),
        );
        r.draw_texture(
            self.shop_tex,
            sx - dw / 2.0,
            sy + HALF.y - dh,
            dw,
            dh,
            color::WHITE,
        );
        r.draw_text_centered("SHOP", sx, sy - dh - 6.0, 1.0, color::rgb(255, 226, 120));
        if (s.pos - self.player.pos).length() < SHOP_REACH && (self.time * 2.0) as i32 % 2 == 0 {
            r.draw_text_centered("PRESS Z", sx, sy + 4.0, 1.0, color::rgb(200, 240, 200));
        }
    }

    fn draw_player(&self, r: &mut Renderer) {
        let w = &self.player.walk;
        let src = walk_src(w, self.player.facing, self.player.walk_t);
        self.blit_sprite(
            r,
            self.player.tex,
            self.player.pos,
            w.draw_w,
            w.draw_h,
            src,
            false,
            color::WHITE,
        );
    }

    fn draw_enemy(&self, r: &mut Renderer, e: &Enemy) {
        let src = walk_src(&e.walk, e.facing, e.walk_t);
        self.blit_sprite(
            r,
            e.tex,
            e.pos,
            e.walk.draw_w,
            e.walk.draw_h,
            src,
            false,
            e.tint,
        );
    }

    /// Draw a battler sprite standing on `pos` (feet), with a soft shadow.
    #[allow(clippy::too_many_arguments)]
    fn blit_sprite(
        &self,
        r: &mut Renderer,
        tex: TextureHandle,
        pos: Vec2,
        dw: f32,
        dh: f32,
        src: [f32; 4],
        flip: bool,
        tint: Color,
    ) {
        let sx = pos.x - self.cam.x;
        let sy = pos.y - self.cam.y;
        r.draw_rect(
            sx - dw * 0.28,
            sy - 1.0,
            dw * 0.56,
            4.0,
            color::rgba(0, 0, 0, 80),
        );
        let dest = [sx - dw / 2.0, sy + HALF.y - dh, dw, dh];
        r.draw_sprite(tex, dest, src, flip, tint);
    }

    fn draw_hud(&self, r: &mut Renderer) {
        let remaining = self.cur().enemies.iter().filter(|e| !e.defeated).count();
        // Top bar: level name + which area of the level we're in.
        r.draw_rect(0.0, 0.0, VIRTUAL_W, 12.0, color::rgba(10, 12, 22, 200));
        r.draw_text(&self.name, 5.0, 2.0, 1.0, color::rgb(200, 220, 200));
        let area = format!("AREA {}/{}", self.current + 1, self.screens.len());
        let aw = r.text_width(&area, 1.0);
        r.draw_text(
            &area,
            VIRTUAL_W - aw - 5.0,
            2.0,
            1.0,
            color::rgb(180, 200, 240),
        );

        if self.all_cleared() {
            r.draw_text_centered(
                "LEVEL CLEARED!",
                VIRTUAL_W / 2.0,
                VIRTUAL_H / 2.0 - 4.0,
                1.4,
                color::rgb(255, 230, 120),
            );
            r.draw_text_centered(
                "PRESS ESC TO RETURN TO THE MAP",
                VIRTUAL_W / 2.0,
                VIRTUAL_H / 2.0 + 12.0,
                1.0,
                color::rgb(210, 210, 180),
            );
        } else if (self.time * 2.0) as i32 % 2 == 0 {
            let hint = format!("ARROWS: MOVE   ESC: MAP   FOES HERE: {remaining}");
            r.draw_text_centered(
                &hint,
                VIRTUAL_W / 2.0,
                VIRTUAL_H - 10.0,
                1.0,
                color::rgba(180, 190, 170, 220),
            );
        }
    }
}

// ---- Free helpers -----------------------------------------------------------

fn tile_center(col: u32, row: u32) -> Vec2 {
    Vec2::new((col as f32 + 0.5) * TILE, (row as f32 + 0.5) * TILE)
}

/// Where the player appears after flipping screens travelling `dir`. They enter
/// through the opposite edge of the new screen `ns`, snapped to the walkable
/// opening on that edge nearest to where they left the old one (`exit`), then
/// nudged just inside so they don't immediately flip back. This lets a screen's
/// doorways sit anywhere along an edge — not only its midpoint — so mazes can
/// wind their exits wherever the layout wants.
fn entry_pos(ns: &Screen, dir: Dir, exit: Vec2) -> Vec2 {
    let map = Vec2::new(ns.w as f32 * TILE, ns.h as f32 * TILE);
    match dir {
        Dir::East => {
            let row = ns.nearest_open_row(0, exit.y);
            Vec2::new(HALF.x + 2.0, (row as f32 + 0.5) * TILE)
        }
        Dir::West => {
            let row = ns.nearest_open_row(ns.w - 1, exit.y);
            Vec2::new(map.x - HALF.x - 2.0, (row as f32 + 0.5) * TILE)
        }
        Dir::South => {
            let col = ns.nearest_open_col(0, exit.x);
            Vec2::new((col as f32 + 0.5) * TILE, HALF.y + 2.0)
        }
        Dir::North => {
            let col = ns.nearest_open_col(ns.h - 1, exit.x);
            Vec2::new((col as f32 + 0.5) * TILE, map.y - HALF.y - 2.0)
        }
    }
}

/// Pick the facing that best matches a movement vector (dominant axis).
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

/// Source rect for a directional walk sprite: the row for `facing`, cycling all
/// `frames` while moving (`walk_t > 0`) and resting on frame 0 when idle.
pub(crate) fn walk_src(w: &OverworldWalk, facing: Facing, walk_t: f32) -> [f32; 4] {
    let row = match facing {
        Facing::Down => w.row_down,
        Facing::Up => w.row_up,
        Facing::Left => w.row_left,
        Facing::Right => w.row_right,
    };
    let frames = w.frames.max(1);
    let frame = if walk_t > 0.0 {
        (walk_t * w.fps) as u32 % frames
    } else {
        0
    };
    [
        (frame * w.frame_w) as f32,
        (row * w.frame_h) as f32,
        w.frame_w as f32,
        w.frame_h as f32,
    ]
}

/// Object tiles (tree/rock) are smaller than a cell; draw them bottom-centered
/// over the grass so they read as props sitting on the ground.
fn blit_object(r: &mut Renderer, tex: TextureHandle, x: f32, y: f32) {
    let (tw, th) = r.texture_size(tex);
    let (tw, th) = (tw as f32, th as f32);
    let ox = x + (TILE - tw) / 2.0;
    let oy = y + (TILE - th);
    r.draw_texture(tex, ox, oy, tw, th, color::WHITE);
}

/// Standalone collision-slide against a tile grid (used for enemies, whose loop
/// holds a mutable borrow that rules out calling `&self` methods).
fn slide_on(tiles: &[Tile], w: usize, h: usize, pos: Vec2, delta: Vec2) -> Vec2 {
    let solid_at = |x: f32, y: f32| -> bool {
        let col = (x / TILE).floor();
        let row = (y / TILE).floor();
        if col < 0.0 || row < 0.0 || col as usize >= w || row as usize >= h {
            return true;
        }
        tiles[row as usize * w + col as usize].solid()
    };
    let blocked = |c: Vec2| -> bool {
        solid_at(c.x - HALF.x, c.y - HALF.y)
            || solid_at(c.x + HALF.x, c.y - HALF.y)
            || solid_at(c.x - HALF.x, c.y + HALF.y)
            || solid_at(c.x + HALF.x, c.y + HALF.y)
    };
    let mut p = pos;
    let nx = Vec2::new(p.x + delta.x, p.y);
    if !blocked(nx) {
        p.x = nx.x;
    }
    let ny = Vec2::new(p.x, p.y + delta.y);
    if !blocked(ny) {
        p.y = ny.y;
    }
    p
}

/// The party leader's overworld walk sprite: its dedicated art, or one
/// synthesized from its battle sprite so any character can lead. Shared by the
/// overworld and the shop interior so the walking hero looks the same in both.
pub(crate) fn leader_walk(reg: &Registry, party: &Party) -> OverworldWalk {
    let leader = &party.members[0];
    reg.character(&leader.def_id)
        .and_then(|c| c.overworld.clone())
        .unwrap_or_else(|| fallback_walk(&leader.sprite))
}

/// Build a serviceable overworld walk sprite from a battle sprite when a
/// character has no dedicated `overworld` art (uses the idle row for all
/// directions, so it at least renders and animates).
fn fallback_walk(sprite: &BattlerSprite) -> OverworldWalk {
    OverworldWalk {
        texture: sprite.texture.clone(),
        frame_w: sprite.frame_w,
        frame_h: sprite.frame_h,
        draw_w: 20.0,
        draw_h: 20.0,
        row_down: sprite.idle.row,
        row_up: sprite.idle.row,
        row_left: sprite.idle.row,
        row_right: sprite.idle.row,
        frames: sprite.idle.frames.max(1),
        fps: sprite.idle.fps,
    }
}
