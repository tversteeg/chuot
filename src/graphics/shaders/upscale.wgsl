// Vertex shader

@group(0) @binding(0)
var input_tex: texture_2d<f32>;

struct ScreenInfo {
    @location(0) size: vec2<f32>,
    @location(1) scale: f32,
}

@group(1) @binding(0)
var<uniform> screen_info: ScreenInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a triangle to fill the screen.
    // The approach is based on: https://stackoverflow.com/a/59739538/4593433.
    var vertices = array(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    var out: VertexOutput;
    out.uv = vertices[in_vertex_index];
    out.clip_position = vec4<f32>(out.uv, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureLoad(input_tex, vec2<i32>(in.clip_position.xy), 0).rgb;

    return vec4<f32>(color, 1.0);
}
