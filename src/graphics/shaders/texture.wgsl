// Vertex shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct ScreenInfo {
    @location(0) size: vec2<f32>,
    @location(1) scale: f32,
}

@group(1) @binding(0)
var<uniform> screen_info: ScreenInfo;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    // Matrix type is not supported in vertex input, construct it from 3 vec3's
    @location(2) mat_0: vec3<f32>,
    @location(3) mat_1: vec3<f32>,
    @location(4) mat_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat3x3<f32>(
        instance.mat_0,
        instance.mat_1,
        instance.mat_2,
    );

    // Translate, rotate and skew with the instance matrix
    let projected_position = instance_matrix * vec3<f32>(model.position.xy, 1.0);

    // Draw upscaled
    let scaled_position = projected_position.xy * screen_info.scale;

    // Move from 0..width to -1..1
    let screen_size_half = screen_info.size / 2.0;
    let offset = 1.0 - scaled_position / screen_size_half;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(vec3(-offset.x, offset.y, model.position.z), 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

