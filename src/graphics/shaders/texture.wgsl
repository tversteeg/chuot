// Vertex shader

struct ScreenInfo {
    @location(0) size: vec2<f32>,
    @location(1) scale: f32,
}

@group(1) @binding(0)
var<uniform> screen_info: ScreenInfo;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) instance_offset: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // Move from 0..width to -1..1
    var screen_size_half = screen_info.size / 2;
    var scaled_position = model.position.xy * screen_info.scale;
    var offset = 1.0 - (model.instance_offset + scaled_position) / screen_size_half;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(vec3(-offset.x, offset.y, model.position.z), 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

