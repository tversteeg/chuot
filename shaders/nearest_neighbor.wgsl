//! Optimized base shader without special rotation algorithms.
 
struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
}
 
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Use base shader from 'shaders/custom_base_shader.wgsl'
    return vs_main_impl(model, instance);
}

@fragment
fn fs_main_nearest_neighbor(in: VertexOutput) -> @location(0) vec4f {
    // Return the exact pixel
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
