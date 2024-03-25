//! Branchless scale3x inspired by https://www.shadertoy.com/view/4l2SRz
//!
//! This shader applies a novel single-pass rotsprite rotation by using UV subpixel relative coordinates to "downscale" an "upscaled" Scale4X sample.

// Size of both width and height of the atlas texture
const ATLAS_TEXTURE_SIZE: f32 = 4096.0;

// Vertex shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct TextureInfo {
    @location(0) offset: vec2<f32>,
    @location(1) size: vec2<f32>,
}

struct ScreenInfo {
    @location(0) size: vec2<f32>,
    // WASM needs to types to be aligned to 16 bytes
    @location(1) _padding: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> tex_info: array<TextureInfo, 1024>;

@group(2) @binding(0)
var<uniform> screen_info: ScreenInfo;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    // Matrix type is not supported in vertex input, construct it from 3 vec3's
    // The last row of the matrix is always 0 0 1 so we can save some bytes by constructing that ourselves
    @location(2) mat_0: vec2<f32>,
    @location(3) mat_1: vec2<f32>,
    @location(4) mat_2: vec2<f32>,
    // Which texture to render, dimensions are stored in the uniform buffer
    @location(5) tex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    // Whether any matrix operation besides simple translation and reflection is applied
    @location(1) @interpolate(flat) only_translated_or_reflected: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Create the 2D affine transformation matrix for each instance
    let instance_matrix = mat3x3<f32>(
        vec3<f32>(instance.mat_0, 0.0),
        vec3<f32>(instance.mat_1, 0.0),
        vec3<f32>(instance.mat_2, 1.0),
    );

    // Get the texture rectangle from the atlas
    let subrect = tex_info[instance.tex_index];

    // Resize the quad to the size of the texture
    let model_position = model.position.xy * subrect.size;

    // Translate, rotate and skew with the instance matrix
    let projected_position = instance_matrix * vec3<f32>(model_position, 1.0);

    // Move from 0..width to -1..1
    let screen_size_half = screen_info.size / 2.0;
    let offset = 1.0 - projected_position.xy / screen_size_half;
    // Move the 0..1 texture coordinates to relative coordinates within the 4096x4096 atlas texture for the specified texture
    let tex_coords = (subrect.offset + subrect.size * model.tex_coords) / ATLAS_TEXTURE_SIZE;

    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.clip_position = vec4<f32>(vec3<f32>(-offset.x, offset.y, model.position.z), 1.0);

    // Check if we have any skewing, scaling or rotation
    out.only_translated_or_reflected = f32( 
        abs(instance.mat_0.x) == 1.0 &&
        instance.mat_0.y == 0.0 &&
        instance.mat_1.x == 0.0 &&
        abs(instance.mat_1.y) == 1.0
    );

    return out;
}

// Fragment shader

// Smallest value to compare pixel colors with
const EPSILON: vec4<f32> = vec4<f32>(1.0 / 255.0);

// Offset needed for moving a single pixel
const PIXEL_OFFSET: f32 = 1.0 / ATLAS_TEXTURE_SIZE;

// Calculate branchless equality between two vectors.
//
// Returns `1.0` if equal within the EPSILON, otherwise `0.0`.
fn vec4_eq(a: vec4<f32>, b: vec4<f32>) -> f32 {
    // Calculate the difference between each component, and return `0.0` or `1.0` depending on whether it's bigger than EPSILON
    let delta = step(abs(a - b), EPSILON);

    // If any component is zero multiplying them with the other components will still return zero, AKA there's inequality
    return delta.x * delta.y * delta.z * delta.w;
}

// Calculate branchless inequality between two vectors.
//
// Returns `0.0` if equal within the EPSILON, otherwise `1.0`.
fn vec4_neq(a: vec4<f32>, b: vec4<f32>) -> f32 {
    return 1.0 - vec4_eq(a, b);
}

/// Logical or for two float values which can be either `0.0` or `1.0`.
fn or(a: f32, b: f32) -> f32 {
    return min(a + b, 1.0);
}

// Apply the Scale2x algorithm.
fn scale2x(
    c: vec4<f32>,
    n: vec4<f32>,
    e: vec4<f32>,
    s: vec4<f32>,
    w: vec4<f32>,
    subpixel: vec2<f32>
) -> vec4<f32> {
    // n != s && w != e
    let master = vec4_neq(n, s) * vec4_neq(w, e);

    // 0 1
    // 2 3
    let e0 = mix(c, w, vec4_eq(w, n) * master);
    let e1 = mix(c, e, vec4_eq(n, e) * master);
    let e2 = mix(c, w, vec4_eq(w, s) * master);
    let e3 = mix(c, e, vec4_eq(s, e) * master);

    let sub_step = step(vec2<f32>(0.5), subpixel);

    return mix(
        // Choose between E2 or E0
        mix(e2, e0, sub_step.y),
        // Choose between E3 or E1
        mix(e3, e1, sub_step.y),
        // Choose between E0 & E2 or E1 & E3
        sub_step.x
    );
}

// Apply the Diag2x algorithm (https://www.slimesalad.com/forum/viewtopic.php?t=8333).
//
// Fixes the bowtie pixels of Scale2x.
fn diag2x(
    nw: vec4<f32>,
    n: vec4<f32>,
    ne: vec4<f32>,
    w: vec4<f32>,
    c: vec4<f32>,
    e: vec4<f32>,
    sw: vec4<f32>,
    s: vec4<f32>,
    se: vec4<f32>,
    subpixel: vec2<f32>
) -> vec4<f32> {
    // n != s && w != e
    let master = vec4_neq(n, s) * vec4_neq(w, e);

    // 0 1
    // 2 3
    let e0 = mix(c, w, vec4_eq(w, n) * vec4_eq(c, nw) * master);
    let e1 = mix(c, e, vec4_eq(n, e) * vec4_eq(c, ne) * master);
    let e2 = mix(c, w, vec4_eq(w, s) * vec4_eq(c, sw) * master);
    let e3 = mix(c, e, vec4_eq(s, e) * vec4_eq(c, sw) * master);

    let sub_step = step(vec2<f32>(0.5), subpixel);

    return mix(
        // Choose between E2 or E0
        mix(e2, e0, sub_step.y),
        // Choose between E3 or E1
        mix(e3, e1, sub_step.y),
        // Choose between E0 & E2 or E1 & E3
        sub_step.x
    );
}

// Apply the Scale3x algorithm.
fn scale3x(
    nw: vec4<f32>,
    n: vec4<f32>,
    ne: vec4<f32>,
    w: vec4<f32>,
    c: vec4<f32>,
    e: vec4<f32>,
    sw: vec4<f32>,
    s: vec4<f32>,
    se: vec4<f32>,
    subpixel: vec2<f32>
) -> vec4<f32> {
    // n != s && w != e
    let master = vec4_neq(n, s) * vec4_neq(w, e);

    let w_eq_n = vec4_eq(w, n);
    let e_eq_n = vec4_eq(e, n);
    let w_eq_s = vec4_eq(w, s);
    let e_eq_s = vec4_eq(e, s);

    let nw_neq_c = vec4_neq(nw, c);
    let ne_neq_c = vec4_neq(ne, c);
    let sw_neq_c = vec4_neq(sw, c);
    let se_neq_c = vec4_neq(se, c);

    // Calculate the upscaled 9 pixels per current pixel
    // Will only detect edges, otherwise it will always return the center pixel due to master
    let e_nw = mix(c, w, w_eq_n * master);
    let e_ne = mix(c, e, e_eq_n * master);
    let e_sw = mix(c, w, w_eq_s * master);
    let e_se = mix(c, e, e_eq_s * master);
    let e_n = mix(c, n, or(w_eq_n * ne_neq_c, e_eq_n * nw_neq_c) * master);
    let e_w = mix(c, w, or(w_eq_n * sw_neq_c, w_eq_s * nw_neq_c) * master);
    let e_e = mix(c, e, or(e_eq_n * se_neq_c, e_eq_s * ne_neq_c) * master);
    let e_s = mix(c, s, or(w_eq_s * se_neq_c, e_eq_s * sw_neq_c) * master);

    // Divide the subpixel into 3 thirds
    let sub_step_1 = step(vec2<f32>(1.0 / 3.0), subpixel);
    let sub_step_2 = step(vec2<f32>(2.0 / 3.0), subpixel);

    // Choose the column per row
    let row_n = mix(e_nw, mix(e_n, e_ne, sub_step_2.x), sub_step_1.x);
    let row_c = mix(e_w, mix(c, e_e, sub_step_2.x), sub_step_1.x);
    let row_s = mix(e_sw, mix(e_s, e_se, sub_step_2.x), sub_step_1.x);

    // Choose the row
    return mix(row_s, mix(row_c, row_n, sub_step_2.y), sub_step_1.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Take the sample of the exact pixel
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Don't apply the algorithm when no rotations or skewing occurs
    if in.only_translated_or_reflected == 1.0 {
        return c;
    }

    // Offset of the UV within the pixel
    let subpixel = fract(in.tex_coords * ATLAS_TEXTURE_SIZE);
    
    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let nw = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(-PIXEL_OFFSET, PIXEL_OFFSET));
    let n = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(0.0, PIXEL_OFFSET));
    let ne = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(PIXEL_OFFSET));
    let w = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(-PIXEL_OFFSET, 0.0));
    let e = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(PIXEL_OFFSET, 0.0));
    let sw = textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2<f32>(PIXEL_OFFSET));
    let s = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(0.0, -PIXEL_OFFSET));
    let se = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2<f32>(PIXEL_OFFSET, -PIXEL_OFFSET));

    // Apply a Scale3x block
    return scale3x(nw, n, ne, w, c, e, sw, s, se, subpixel);
}
