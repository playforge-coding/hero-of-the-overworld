//! wgpu-based 2D sprite renderer.
//!
//! The whole game is drawn as tinted textured quads in a fixed *virtual*
//! resolution (see [`VIRTUAL_W`]/[`VIRTUAL_H`]). The renderer letterboxes that
//! virtual canvas into whatever window/canvas size winit gives us, so the game
//! logic never has to think about real pixels or DPI.
//!
//! Draw calls are immediate-mode: each frame the game calls `draw_*` to queue
//! quads, then `render()` batches consecutive quads that share a texture and
//! submits them. Submission order is preserved, so drawing back-to-front just
//! works (painter's algorithm).

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::window::Window;

/// Virtual canvas width in pixels. Game coordinates are in this space.
pub const VIRTUAL_W: f32 = 320.0;
/// Virtual canvas height in pixels.
pub const VIRTUAL_H: f32 = 180.0;

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
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Instance {
    dest: [f32; 4], // x, y, w, h (top-left origin, virtual pixels)
    uv: [f32; 4],   // u0, v0, u1, v1
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
}

struct TextureEntry {
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

/// Metrics for the built-in monospace bitmap font atlas.
pub struct FontInfo {
    pub texture: TextureHandle,
    pub cell_w: u32,
    pub cell_h: u32,
    pub cols: u32,
    pub first_char: u8,
    pub last_char: u8,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,

    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    tex_bind_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    quad_vbuf: wgpu::Buffer,
    quad_ibuf: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,

    textures: Vec<TextureEntry>,
    white: TextureHandle,
    font: FontInfo,

    // Per-frame queued draws.
    instances: Vec<Instance>,
    runs: Vec<(usize, u32)>, // (texture index, instance count)

    clear_color: wgpu::Color,
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, font_png: &[u8]) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("no suitable GPU adapter");

        // WebGL2 (the wasm fallback when WebGPU is unavailable) has tighter
        // limits, so cap our required limits to what the adapter reports.
        let required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        }
        .using_resolution(adapter.limits());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("request device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // ---- Shader & pipeline -------------------------------------------------
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sprite.wgsl").into()),
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let tex_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[Some(&globals_layout), Some(&tex_bind_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
        };
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: (2 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Unit quad (two triangles via index buffer).
        let quad: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let quad_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertices"),
            contents: bytemuck::cast_slice(&quad),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals bind group"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let instance_capacity = 1024;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: (instance_capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            globals_buffer,
            globals_bind_group,
            tex_bind_layout,
            sampler,
            quad_vbuf,
            quad_ibuf,
            instance_buffer,
            instance_capacity,
            textures: Vec::new(),
            white: TextureHandle(0),
            font: FontInfo {
                texture: TextureHandle(0),
                cell_w: 8,
                cell_h: 13,
                cols: 16,
                first_char: 32,
                last_char: 126,
            },
            instances: Vec::new(),
            runs: Vec::new(),
            clear_color: wgpu::Color {
                r: 0.02,
                g: 0.02,
                b: 0.04,
                a: 1.0,
            },
            window,
        };

        // 1x1 white pixel used for solid-color rectangles.
        renderer.white = renderer.create_texture_rgba(1, 1, &[255, 255, 255, 255]);

        // Built-in font atlas (8x13 cells, 16 columns, ASCII 32..126).
        let font_tex = renderer.load_png(font_png);
        renderer.font = FontInfo {
            texture: font_tex,
            cell_w: 8,
            cell_h: 13,
            cols: 16,
            first_char: 32,
            last_char: 126,
        };

        renderer
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn font(&self) -> &FontInfo {
        &self.font
    }

    pub fn white(&self) -> TextureHandle {
        self.white
    }

    pub fn texture_size(&self, tex: TextureHandle) -> (u32, u32) {
        let e = &self.textures[tex.0];
        (e.width, e.height)
    }

    pub fn set_clear_color(&mut self, c: Color) {
        self.clear_color = wgpu::Color {
            r: c[0] as f64,
            g: c[1] as f64,
            b: c[2] as f64,
            a: c[3] as f64,
        };
    }

    // ---- Texture creation -----------------------------------------------------

    /// Decode a PNG byte slice into a registered texture.
    pub fn load_png(&mut self, bytes: &[u8]) -> TextureHandle {
        let img = image::load_from_memory(bytes)
            .expect("decode png")
            .to_rgba8();
        let (w, h) = img.dimensions();
        self.create_texture_rgba(w, h, &img)
    }

    pub fn create_texture_rgba(&mut self, width: u32, height: u32, rgba: &[u8]) -> TextureHandle {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &self.tex_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.textures.push(TextureEntry {
            bind_group,
            width,
            height,
        });
        TextureHandle(self.textures.len() - 1)
    }

    // ---- Draw queue -----------------------------------------------------------

    fn push(&mut self, tex: TextureHandle, dest: [f32; 4], uv: [f32; 4], tint: Color) {
        self.instances.push(Instance { dest, uv, tint });
        match self.runs.last_mut() {
            Some((t, n)) if *t == tex.0 => *n += 1,
            _ => self.runs.push((tex.0, 1)),
        }
    }

    /// Draw a solid-color rectangle.
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let white = self.white;
        self.push(white, [x, y, w, h], [0.0, 0.0, 1.0, 1.0], color);
    }

    /// Draw a 1px-outline rectangle.
    pub fn draw_rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, t: f32, color: Color) {
        self.draw_rect(x, y, w, t, color);
        self.draw_rect(x, y + h - t, w, t, color);
        self.draw_rect(x, y, t, h, color);
        self.draw_rect(x + w - t, y, t, h, color);
    }

    /// Draw a sub-rectangle of a texture (in source pixels) into a destination
    /// rectangle (in virtual pixels), optionally horizontally flipped.
    pub fn draw_sprite(
        &mut self,
        tex: TextureHandle,
        dest: [f32; 4],
        src: [f32; 4],
        flip_x: bool,
        tint: Color,
    ) {
        let (tw, th) = self.texture_size(tex);
        let (tw, th) = (tw as f32, th as f32);
        let mut u0 = src[0] / tw;
        let v0 = src[1] / th;
        let mut u1 = (src[0] + src[2]) / tw;
        let v1 = (src[1] + src[3]) / th;
        if flip_x {
            std::mem::swap(&mut u0, &mut u1);
        }
        self.push(tex, dest, [u0, v0, u1, v1], tint);
    }

    /// Draw a whole texture at a destination rectangle.
    pub fn draw_texture(&mut self, tex: TextureHandle, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        self.push(tex, [x, y, w, h], [0.0, 0.0, 1.0, 1.0], tint);
    }

    /// Width in virtual pixels a string will occupy at the given scale.
    pub fn text_width(&self, text: &str, scale: f32) -> f32 {
        text.chars().count() as f32 * (self.font.cell_w as f32 - 2.0) * scale
    }

    /// Draw a string using the built-in bitmap font. Returns the end X.
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: Color) -> f32 {
        let f = FontLayout::from(&self.font);
        let tex = self.font.texture;
        let advance = (f.cell_w - 2.0) * scale; // trim a little inter-glyph space
        let gw = f.cell_w * scale;
        let gh = f.cell_h * scale;
        let mut cx = x;
        for ch in text.chars() {
            let c = ch as u32;
            if ch == ' ' || c < f.first || c > f.last {
                cx += advance;
                continue;
            }
            let idx = c - f.first;
            let col = idx % f.cols;
            let row = idx / f.cols;
            let src = [
                col as f32 * f.cell_w,
                row as f32 * f.cell_h,
                f.cell_w,
                f.cell_h,
            ];
            self.draw_sprite(tex, [cx, y, gw, gh], src, false, color);
            cx += advance;
        }
        cx
    }

    /// Draw text centered horizontally on `cx`.
    pub fn draw_text_centered(&mut self, text: &str, cx: f32, y: f32, scale: f32, color: Color) {
        let w = self.text_width(text, scale);
        self.draw_text(text, cx - w / 2.0, y, scale, color);
    }

    // ---- Presentation ---------------------------------------------------------

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Reconfigure using the current window size (used on surface loss).
    pub fn reconfigure(&mut self) {
        let size = self.window.inner_size();
        self.resize(size.width, size.height);
    }

    /// Compute the letterboxed viewport (x, y, w, h) inside the real surface
    /// that preserves the virtual aspect ratio.
    fn viewport(&self) -> (f32, f32, f32, f32) {
        let sw = self.config.width as f32;
        let sh = self.config.height as f32;
        let target = VIRTUAL_W / VIRTUAL_H;
        let actual = sw / sh;
        if actual > target {
            let w = sh * target;
            ((sw - w) / 2.0, 0.0, w, sh)
        } else {
            let h = sw / target;
            (0.0, (sh - h) / 2.0, sw, h)
        }
    }

    /// Submit all queued draws and present. Clears the queue afterwards.
    pub fn render(&mut self) {
        // Upload globals: orthographic, y-down, virtual-pixel space, clip z=0.5.
        let globals = Globals {
            view_proj: ortho_y_down(VIRTUAL_W, VIRTUAL_H),
        };
        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        // Grow instance buffer if needed.
        if self.instances.len() > self.instance_capacity {
            self.instance_capacity = self.instances.len().next_power_of_two();
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instances"),
                size: (self.instance_capacity * std::mem::size_of::<Instance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !self.instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instances),
            );
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.instances.clear();
                self.runs.clear();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => {
                self.reconfigure();
                self.instances.clear();
                self.runs.clear();
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let (vx, vy, vw, vh) = self.viewport();
            pass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_vertex_buffer(0, self.quad_vbuf.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.quad_ibuf.slice(..), wgpu::IndexFormat::Uint16);

            let mut first: u32 = 0;
            for (tex_idx, count) in &self.runs {
                let end = first + count;
                pass.set_bind_group(1, &self.textures[*tex_idx].bind_group, &[]);
                pass.draw_indexed(0..6, 0, first..end);
                first = end;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);

        self.instances.clear();
        self.runs.clear();
    }
}

/// Orthographic projection mapping (0,0)-(w,h) virtual pixels (y-down) into
/// wgpu clip space, with z fixed at 0.5 (inside the [0,1] depth range).
fn ortho_y_down(w: f32, h: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / w, 0.0, 0.0, 0.0],
        [0.0, -2.0 / h, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [-1.0, 1.0, 0.5, 1.0],
    ]
}

struct FontLayout {
    cell_w: f32,
    cell_h: f32,
    cols: u32,
    first: u32,
    last: u32,
}

impl From<&FontInfo> for FontLayout {
    fn from(f: &FontInfo) -> Self {
        FontLayout {
            cell_w: f.cell_w as f32,
            cell_h: f.cell_h as f32,
            cols: f.cols,
            first: f.first_char as u32,
            last: f.last_char as u32,
        }
    }
}
