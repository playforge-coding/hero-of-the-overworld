// Sprite pipeline: instanced textured quads in virtual-pixel space.
// Every drawable in the game (sprites, UI panels, HP bars, text glyphs) is a
// tinted textured quad, so this single shader covers the whole renderer.

struct Globals {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

@group(1) @binding(0) var atlas: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

// Per-vertex: a unit quad corner in 0..1.
struct VertexIn {
    @location(0) corner: vec2<f32>,
};

// Per-instance data.
struct InstanceIn {
    @location(1) dest: vec4<f32>, // x, y, w, h  (top-left origin, virtual pixels)
    @location(2) uv:   vec4<f32>, // u0, v0, u1, v1
    @location(3) tint: vec4<f32>, // rgba multiply
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(v: VertexIn, i: InstanceIn) -> VertexOut {
    let world = i.dest.xy + v.corner * i.dest.zw;
    var out: VertexOut;
    out.clip = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = mix(i.uv.xy, i.uv.zw, v.corner);
    out.tint = i.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas, atlas_sampler, in.uv);
    let color = sampled * in.tint;
    if (color.a < 0.001) {
        discard;
    }
    return color;
}
