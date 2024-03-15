// Vertex shader

struct ScreenInfo {
    @location(0) size: vec2<f32>,
    @location(1) scale: f32,
}

@group(1) @binding(0)
var<uniform> screen_info: ScreenInfo;

@group(0) @binding(0)
var input_tex : texture_2d<f32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    // Generate a triangle to fill the screen.
    // The approach is based on: https://stackoverflow.com/a/59739538/4593433.
    var fullscreen_vertices = array(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(3.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 3.0, 0.0, 1.0)
    );

    return fullscreen_vertices[in.vertex_index];
}

// Fragment shader

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let color = textureLoad(input_tex, vec2<i32>(pos.xy / screen_info.scale), 0).rgb;

    return vec4<f32>(color, 1.0);
}
