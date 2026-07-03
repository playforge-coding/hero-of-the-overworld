//! macroquad-based 2D renderer.
//!
//! The whole game is drawn in a fixed *virtual* resolution (see
//! [`VIRTUAL_W`]/[`VIRTUAL_H`]). A `Camera2D` maps that virtual space into a
//! letterboxed viewport in the real window, so game logic never thinks about
//! real pixels or DPI. Sprites are drawn straight to the window through that
//! camera — a single nearest-neighbour resample from source art to final size —
//! which keeps pixel art crisp and undistorted (an intermediate low-res render
//! target would resample twice and warp scaled sprites).
//!
//! Text uses macroquad's built-in TrueType rasteriser (a `Font`), drawn on top
//! at the final on-screen size so glyphs stay sharp. Draw calls are queued
//! during a frame and replayed in order by [`render`](Renderer::render), so
//! painter-order layering and mid-frame
//! [`set_clear_color`](Renderer::set_clear_color) work as before.

use macroquad::prelude::{
    clear_background, draw_rectangle, draw_text_ex, draw_texture_ex, load_ttf_font_from_bytes,
    measure_text, screen_height, screen_width, set_camera, set_default_camera, vec2, Camera2D,
    Color as MqColor, DrawTextureParams, FilterMode, Font, Rect as MqRect, TextParams, Texture2D,
    BLACK,
};

/// Virtual canvas width in pixels. Game coordinates are in this space.
pub const VIRTUAL_W: f32 = 320.0;
/// Virtual canvas height in pixels.
pub const VIRTUAL_H: f32 = 180.0;

/// On-screen text height (px, virtual space) at `scale == 1.0`. Other scales
/// derive from this; tuned to match the old UI layout.
const BASE_FONT_PX: f32 = 12.0;

/// Handle to a texture registered with the renderer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureHandle(usize);

/// RGBA color, components in 0..1.
pub type Color = [f32; 4];

pub mod color {
    use super::Color;
    pub const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
    pub const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
    }
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ]
    }
}

fn mq(c: Color) -> MqColor {
    MqColor::new(c[0], c[1], c[2], c[3])
}

fn font_px(scale: f32) -> u16 {
    (scale * BASE_FONT_PX).round().max(1.0) as u16
}

/// A single queued draw, replayed in order by [`Renderer::render`].
enum Cmd {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: MqColor,
    },
    Sprite {
        tex: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        /// Source sub-rect in texture pixels; `None` means the whole texture.
        src: Option<MqRect>,
        flip_x: bool,
        color: MqColor,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        size: u16,
        color: MqColor,
    },
}

pub struct Renderer {
    textures: Vec<Texture2D>,
    font: Font,
    clear_color: MqColor,
    queue: Vec<Cmd>,
}

impl Renderer {
    /// Build the renderer. Must be called after macroquad's context exists (i.e.
    /// inside the async `main`), because it uploads the font atlas.
    pub fn new(font_ttf: &[u8]) -> Self {
        let mut font = load_ttf_font_from_bytes(font_ttf).expect("load UI font");
        // Nearest keeps the pixel font crisp when scaled up.
        font.set_filter(FilterMode::Nearest);

        Renderer {
            textures: Vec::new(),
            font,
            clear_color: MqColor::new(0.02, 0.02, 0.04, 1.0),
            queue: Vec::new(),
        }
    }

    // ---- Texture creation -----------------------------------------------------

    /// Register a PNG byte slice as a nearest-filtered texture.
    pub fn load_png(&mut self, bytes: &[u8]) -> TextureHandle {
        let tex = Texture2D::from_file_with_format(bytes, None);
        tex.set_filter(FilterMode::Nearest);
        self.textures.push(tex);
        TextureHandle(self.textures.len() - 1)
    }

    pub fn texture_size(&self, tex: TextureHandle) -> (u32, u32) {
        let t = &self.textures[tex.0];
        (t.width() as u32, t.height() as u32)
    }

    pub fn set_clear_color(&mut self, c: Color) {
        self.clear_color = mq(c);
    }

    // ---- Draw queue -----------------------------------------------------------

    /// Draw a solid-color rectangle.
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.queue.push(Cmd::Rect {
            x,
            y,
            w,
            h,
            color: mq(color),
        });
    }

    /// Draw a `t`-thick outline rectangle.
    pub fn draw_rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, t: f32, color: Color) {
        self.draw_rect(x, y, w, t, color);
        self.draw_rect(x, y + h - t, w, t, color);
        self.draw_rect(x, y, t, h, color);
        self.draw_rect(x + w - t, y, t, h, color);
    }

    /// Draw a sub-rectangle of a texture (source in pixels) into a destination
    /// rectangle (virtual pixels), optionally horizontally flipped.
    pub fn draw_sprite(
        &mut self,
        tex: TextureHandle,
        dest: [f32; 4],
        src: [f32; 4],
        flip_x: bool,
        tint: Color,
    ) {
        self.queue.push(Cmd::Sprite {
            tex: tex.0,
            x: dest[0],
            y: dest[1],
            w: dest[2],
            h: dest[3],
            src: Some(MqRect::new(src[0], src[1], src[2], src[3])),
            flip_x,
            color: mq(tint),
        });
    }

    /// Draw a whole texture at a destination rectangle.
    pub fn draw_texture(
        &mut self,
        tex: TextureHandle,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tint: Color,
    ) {
        self.queue.push(Cmd::Sprite {
            tex: tex.0,
            x,
            y,
            w,
            h,
            src: None,
            flip_x: false,
            color: mq(tint),
        });
    }

    /// Width in virtual pixels a string will occupy at the given scale.
    pub fn text_width(&self, text: &str, scale: f32) -> f32 {
        measure_text(text, Some(&self.font), font_px(scale), 1.0).width
    }

    /// Queue a string using the built-in font. `x,y` is the top-left. Returns the
    /// end X, so callers can chain text.
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: Color) -> f32 {
        let size = font_px(scale);
        let w = measure_text(text, Some(&self.font), size, 1.0).width;
        self.queue.push(Cmd::Text {
            text: text.to_string(),
            x,
            y,
            size,
            color: mq(color),
        });
        x + w
    }

    /// Draw text centered horizontally on `cx`.
    pub fn draw_text_centered(&mut self, text: &str, cx: f32, y: f32, scale: f32, color: Color) {
        let w = self.text_width(text, scale);
        self.draw_text(text, cx - w / 2.0, y, scale, color);
    }

    // ---- Presentation ---------------------------------------------------------

    /// Letterboxed viewport (x, y, w, h) in real pixels that preserves the
    /// virtual aspect ratio, centered in the window.
    fn viewport(&self) -> (f32, f32, f32, f32) {
        let sw = screen_width();
        let sh = screen_height();
        let aspect = VIRTUAL_W / VIRTUAL_H;
        let (vw, vh) = if sw / sh > aspect {
            (sh * aspect, sh)
        } else {
            (sw, sw / aspect)
        };
        ((sw - vw) / 2.0, (sh - vh) / 2.0, vw, vh)
    }

    /// Replay every queued draw. Shapes and sprites go through a camera that maps
    /// the virtual canvas into the letterboxed viewport (drawn directly to the
    /// window, one resample); text is drawn afterwards at the final on-screen
    /// size so it stays sharp. Clears the queue; the caller presents via
    /// `next_frame().await`.
    pub fn render(&mut self) {
        // Whole-window black first, for the letterbox bars.
        set_default_camera();
        clear_background(BLACK);

        let (vx, vy, vw, vh) = self.viewport();
        // Camera: virtual (0,0)-(VW,VH) -> the viewport rect, top-left origin,
        // y-down (matching game coordinates).
        let cam = Camera2D {
            target: vec2(VIRTUAL_W / 2.0, VIRTUAL_H / 2.0),
            zoom: vec2(2.0 / VIRTUAL_W, 2.0 / VIRTUAL_H),
            viewport: Some((vx as i32, vy as i32, vw as i32, vh as i32)),
            ..Default::default()
        };
        set_camera(&cam);

        // Scene background fills only the viewport (the camera clips to it).
        draw_rectangle(0.0, 0.0, VIRTUAL_W, VIRTUAL_H, self.clear_color);

        for cmd in &self.queue {
            match cmd {
                Cmd::Rect { x, y, w, h, color } => {
                    draw_rectangle(*x, *y, *w, *h, *color);
                }
                Cmd::Sprite {
                    tex,
                    x,
                    y,
                    w,
                    h,
                    src,
                    flip_x,
                    color,
                } => {
                    draw_texture_ex(
                        &self.textures[*tex],
                        *x,
                        *y,
                        *color,
                        DrawTextureParams {
                            dest_size: Some(vec2(*w, *h)),
                            source: *src,
                            flip_x: *flip_x,
                            ..Default::default()
                        },
                    );
                }
                Cmd::Text { .. } => {} // drawn on top at screen resolution below
            }
        }

        // Text on top, rasterised at the real on-screen size (crisp).
        set_default_camera();
        let s = vw / VIRTUAL_W; // virtual -> screen scale
        for cmd in &self.queue {
            if let Cmd::Text {
                text,
                x,
                y,
                size,
                color,
            } = cmd
            {
                let fs = ((*size as f32) * s).round().max(1.0) as u16;
                let dims = measure_text(text, Some(&self.font), fs, 1.0);
                draw_text_ex(
                    text,
                    vx + x * s,
                    vy + y * s + dims.offset_y,
                    TextParams {
                        font: Some(&self.font),
                        font_size: fs,
                        color: *color,
                        ..Default::default()
                    },
                );
            }
        }

        self.queue.clear();
    }
}
