//! Optimized base shader without special rotation algorithms.

// Size of both width and height of the atlas texture
const ATLAS_TEXTURE_SIZE: f32 = 4096.0;

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct TextureInfo {
    @location(0) offset: vec2f,
    // Not used
    @location(1) size: vec2f,
}

struct ScreenInfo {
    @location(0) size: vec2f,
    // WASM needs the types to be aligned to 16 bytes
    @location(1) _padding: vec2f,
}

@group(1) @binding(0)
var<uniform> tex_info: array<TextureInfo, 1024>;

@group(2) @binding(0)
var<uniform> screen_info: ScreenInfo;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
}

struct InstanceInput {
    // Matrix type is not supported in vertex input, construct it from 3 vec3's
    // The last row of the matrix is always 0 0 1 so we can save some bytes by constructing that ourselves
    @location(2) matrix: vec4f,
    // X and Y position used in the transformation matrix
    @location(3) translation: vec2f,
    // Sub rectangle of the texture to render, offset of the texture will be determined by the texture info uniform
    @location(4) sub_rectangle: vec4f,
    // Which texture to render, dimensions are stored in the uniform buffer
    @location(5) tex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Create the 2D affine transformation matrix for each instance
    let instance_matrix = mat3x3<f32>(
        vec3f(instance.matrix.xy, 0.0),
        vec3f(instance.matrix.zw, 0.0),
        vec3f(instance.translation, 1.0),
    );

    // Get the texture rectangle from the atlas
    let offset = tex_info[instance.tex_index].offset;

    // Resize the quad to the size of the texture
    let model_position = model.position.xy * instance.sub_rectangle.zw;

    // Translate, rotate and skew with the instance matrix
    let projected_position = instance_matrix * vec3f(model_position, 1.0);

    // Move from 0..width to -1..1
    let screen_size_half = screen_info.size / 2.0;
    let screen_offset = 1.0 - projected_position.xy / screen_size_half;
    // Move the 0..1 texture coordinates to relative coordinates within the 4096x4096 atlas texture for the specified texture
    // Also apply the sub rectangle offset from the instance
    let tex_coords = (offset + instance.sub_rectangle.xy + instance.sub_rectangle.zw * model.tex_coords) / ATLAS_TEXTURE_SIZE;

    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.clip_position = vec4f(-screen_offset.x, screen_offset.y, model.position.z, 1.0);

    return out;
}

@fragment
fn fs_main_nearest_neighbor(in: VertexOutput) -> @location(0) vec4f {
    // Return the exact pixel
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
