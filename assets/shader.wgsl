 struct VertexOutput {
    // These two fields must be here
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // This function needs this name and input types

    // Use the complicated vertex shader setup from the engine
    return vs_main_impl(model, instance);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // This function needs this name and output types

    // Return the pixel in a nearest-neighbor fashion
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) 
        // Add a red hue to the pixel
        + vec4f(1.0, 0.0, 0.0, 0.0);
}
