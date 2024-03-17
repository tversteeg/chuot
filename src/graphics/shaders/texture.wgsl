// Scaling algorithm based on: https://www.shadertoy.com/view/4l2SRz

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
    let scaled_position = projected_position.xy;

    // Move from 0..width to -1..1
    let screen_size_half = screen_info.size / 2.0;
    let offset = 1.0 - scaled_position / screen_size_half;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(vec3<f32>(-offset, model.position.z), 1.0);
    return out;
}

// Fragment shader

const epsilon: vec4<f32> = vec4<f32>(1.0 / 255.0);

fn vec4_equal(a: vec4<f32>, b: vec4<f32>) -> f32 {
    let delta = step(abs(a - b), epsilon);

    return delta.x * delta.y * delta.z * delta.w;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate the relative UV offset to see the next pixel
    let pixel_offset = 1.0 / screen_info.size;
    
    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let n = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(0.0, pixel_offset.y));
    let w = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(-pixel_offset.x, 0.0));
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let e = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(pixel_offset.x, 0.0));
    let s = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(0.0, -pixel_offset.y));

    let n_df_s = 1.0 - vec4_equal(n, s);
    let w_df_e = 1.0 - vec4_equal(w, e);
    let master = n_df_s * w_df_e;
    // 0 1
    // 2 3
    let e0 = mix(c, w, vec4_equal(w, n) * master);
    let e1 = mix(c, e, vec4_equal(n, e) * master);
    let e2 = mix(c, w, vec4_equal(w, s) * master);
    let e3 = mix(c, e, vec4_equal(s, e) * master);

    let subpixel = fract(in.tex_coords);
    let sub_step = vec2<f32>(step(0.5, subpixel.x), step(0.5, subpixel.y));
    let scale2x = mix(
        mix(e2, e0, sub_step.y),
        mix(e3, e1, sub_step.y),
        sub_step.x
    );

    return scale2x;
}
