//! Branchless scale3x inspired by https://www.shadertoy.com/view/4l2SRz
//!
//! This shader applies a novel single-pass rotsprite rotation by using UV subpixel relative coordinates to "downscale" an "upscaled" sample.

// Size of both width and height of the atlas texture
const ATLAS_TEXTURE_SIZE: f32 = 4096.0;

// Vertex shader

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
    // Whether any matrix operation besides simple translation and reflection is applied
    @location(1) @interpolate(flat) only_translated_or_reflected: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Create the 2D affine transformation matrix for each instance
    let instance_matrix = mat3x3f(
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

    // Check if we have any skewing, scaling or rotation
    out.only_translated_or_reflected = f32( 
        abs(instance.matrix.x) == 1.0 &&
        instance.matrix.y == 0.0 &&
        instance.matrix.z == 0.0 &&
        abs(instance.matrix.w) == 1.0
    );

    return out;
}

// Fragment shader

// Smallest value to compare pixel colors with
const EPSILON: vec4f = vec4f(1.0 / 255.0);

// Offset needed for moving a single pixel
const PIXEL_OFFSET: f32 = 1.0 / ATLAS_TEXTURE_SIZE;
// Offset needed for moving two pixels
const PIXEL_OFFSET_2: f32 = 2.0 / ATLAS_TEXTURE_SIZE;

// Calculate branchless equality between two vectors.
//
// Returns `1.0` if equal within the EPSILON, otherwise `0.0`.
fn vec4_eq(a: vec4f, b: vec4f) -> f32 {
    // Calculate the difference between each component, and return `0.0` or `1.0` depending on whether it's bigger than EPSILON
    let delta = step(abs(a - b), EPSILON);

    // If any component is zero multiplying them with the other components will still return zero, AKA there's inequality
    return delta.x * delta.y * delta.z * delta.w;
}

// Calculate branchless inequality between two vectors.
//
// Returns `0.0` if equal within the EPSILON, otherwise `1.0`.
fn vec4_neq(a: vec4f, b: vec4f) -> f32 {
    return 1.0 - vec4_eq(a, b);
}

/// Logical or for two float values which can be either `0.0` or `1.0`.
fn or(a: f32, b: f32) -> f32 {
    return min(a + b, 1.0);
}

// Apply the Scale2x algorithm.
fn scale2x(
    n: vec4f,
    e: vec4f,
    c: vec4f,
    s: vec4f,
    w: vec4f,
    subpixel: vec2f
) -> vec4f {
    // n != s && w != e
    let master = vec4_neq(n, s) * vec4_neq(w, e);

    // 0 1
    // 2 3
    let e0 = mix(c, w, vec4_eq(w, n) * master);
    let e1 = mix(c, e, vec4_eq(n, e) * master);
    let e2 = mix(c, w, vec4_eq(w, s) * master);
    let e3 = mix(c, e, vec4_eq(s, e) * master);

    let sub_step = step(vec2f(0.5), subpixel);

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
    nw: vec4f,
    n: vec4f,
    ne: vec4f,
    w: vec4f,
    c: vec4f,
    e: vec4f,
    sw: vec4f,
    s: vec4f,
    se: vec4f,
    subpixel: vec2f
) -> vec4f {
    // n != s && w != e
    let master = vec4_neq(n, s) * vec4_neq(w, e);

    // 0 1
    // 2 3
    let e0 = mix(c, w, vec4_eq(w, n) * vec4_eq(c, nw) * master);
    let e1 = mix(c, e, vec4_eq(n, e) * vec4_eq(c, ne) * master);
    let e2 = mix(c, w, vec4_eq(w, s) * vec4_eq(c, sw) * master);
    let e3 = mix(c, e, vec4_eq(s, e) * vec4_eq(c, sw) * master);

    let sub_step = step(vec2f(0.5), subpixel);

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
    nw: vec4f,
    n: vec4f,
    ne: vec4f,
    w: vec4f,
    c: vec4f,
    e: vec4f,
    sw: vec4f,
    s: vec4f,
    se: vec4f,
    subpixel: vec2f
) -> vec4f {
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
    let sub_step_1 = step(vec2f(1.0 / 3.0), subpixel);
    let sub_step_2 = step(vec2f(2.0 / 3.0), subpixel);

    // Choose the column per row
    let row_n = mix(e_nw, mix(e_n, e_ne, sub_step_2.x), sub_step_1.x);
    let row_c = mix(e_w, mix(c, e_e, sub_step_2.x), sub_step_1.x);
    let row_s = mix(e_sw, mix(e_s, e_se, sub_step_2.x), sub_step_1.x);

    // Choose the row
    return mix(row_s, mix(row_c, row_n, sub_step_2.y), sub_step_1.y);
}

// Similar for the cleanEdge algorithm
fn similar(color1: vec4f, color2: vec4f) -> bool {
    return (color1.a == 0.0 && color2.a == 0.0) || all(color1.rgb == color2.rgb);
}

// Similar with 3 colors for the cleanEdge algorithm
fn similar3(color1: vec4f, color2: vec4f, color3: vec4f) -> bool {
    return similar(color1, color2) && similar(color2, color3);
}

// Similar with 4 colors for the cleanEdge algorithm
fn similar4(color1: vec4f, color2: vec4f, color3: vec4f, color4: vec4f) -> bool {
    return similar(color1, color2) && similar(color2, color3) && similar(color3, color4);
}

// Similar with 5 colors for the cleanEdge algorithm
fn similar5(color1: vec4f, color2: vec4f, color3: vec4f, color4: vec4f, color5: vec4f) -> bool {
    return similar(color1, color2) && similar(color2, color3) && similar(color3, color4) && similar(color4, color5);
}

// Higher color for the cleanEdge algorithm
fn higher(this_color: vec4f, other_color: vec4f) -> bool {
    // Vec3(1.0) is the color with the highest priority
    // Other colors will be tested based on the distance to this color to determine which colors take priority for overlaps

    // If similar return false
    return !similar(this_color, other_color) &&
        select(
            this_color.a > other_color.a,
            // Practically equivalent to `distance(this_color.rgb, vec3(1)) < distance(other_color.rgb, vec3(1))`
            this_color.r + this_color.g + this_color.b >= other_color.r + other_color.g + other_color.b,
            this_color.a == other_color.a);
}

// Distance to line for the cleanEdge algorithm
fn dist_to_line(test_point: vec2f, point1: vec2f, point2: vec2f, dir: vec2f) -> f32 {
    let line_dir = point2 - point1;
    let perp_dir = vec2f(line_dir.y, -line_dir.x);
    let dir_to_point1 = point1 - test_point;

    return sign(dot(perp_dir, dir)) * dot(normalize(perp_dir), dir_to_point1);
}

// cleanEdge distance check
fn dist_check(dist: f32, c: vec4f, a: vec4f, b: vec4f) -> vec4f {
    // If distance is inside line, return -1, otherwise return the closest
    return select(
        select(a, b, distance(c, a) <= distance(c, b)),
        vec4(-1.0),
        dist > LINE_WIDTH / 2.0);
}

// cleanEdge distance caluclation
fn midpoint(
    flip: bool,
    point: vec2f,
    point_dir: vec2f,
    offset_a1: vec2f,
    offset_a2: vec2f,
    offset_b1: vec2f,
    offset_b2: vec2f
) -> f32 {
    // Midpoints of neighbor two-pixel groupings
    return select(
        dist_to_line(
            point,
            fma(offset_b1, point_dir, CENTER),
            fma(offset_b2, point_dir, CENTER),
            point_dir),
        LINE_WIDTH - dist_to_line(
            point,
            fma(offset_a1, point_dir, CENTER),
            fma(offset_a2, point_dir, CENTER),
            -point_dir),
        flip);
}

// cleanEdge line width parameter
const LINE_WIDTH: f32 = 1.0; 
// cleanEdge center value
const CENTER = vec2f(0.5, 0.5);

// Slice distance for the cleanEdge algorithm
fn slice_dist(
    point: vec2f,
    main_dir: vec2f,
    point_dir: vec2f,
    n: vec4f,
    ne: vec4f,
    nee: vec4f,
    w: vec4f,
    c: vec4f,
    e: vec4f,
    ee: vec4f,
    sw: vec4f,
    s: vec4f,
    se: vec4f,
    see: vec4f,
    ssw: vec4f,
    ss: vec4f,
    sse: vec4f,
) -> vec4f {

    // Edge detection

    let dist_against = fma(
        4.0,
        distance(e, s),
            distance(ne, c)
            + distance(c, sw)
            + distance(ee, se)
            + distance(se, ss)
        );

    let dist_towards = fma(
        4.0,
        distance(c, se),
            distance(n, e)
            + distance(e, see)
            + distance(w, s)
            + distance(s, sse)
        );

    // Check for equivalent edges or checkerboard patterns
    if !(dist_against < dist_towards || dist_against < dist_towards + 0.001 && !higher(c, e)) 
        || (similar4(e, s, w, n) && similar3(ne, se, sw) && !similar(c, e)) {
        return vec4f(-1.0);
    }

    // Flip point
    let flipped_point = fma(main_dir, point - 0.5, CENTER);

    var flip = false;
    if similar3(e, s, sw) && !similar3(e, s, w) && !similar(ne, sw) {
        // Lower shallow 2:1 slant
        if similar(c, se) && higher(c, e) {
            // Single pixel wide diagonal, don't flip
        } else if higher(c, e) || (similar(n, e) && !similar(c, se) && !higher(c, n)) {
            // Priority edge cases, flip
            flip = true;
        }

        // Midpoints of neighbor two-pixel groupings
        let dist = midpoint(
            flip,
            flipped_point,
            point_dir, 
            vec2f(1.5, -1.0),
            vec2f(-0.5, 0.0),
            vec2f(1.5, 0.0),
            vec2f(-0.5, 1.0),
        );

        return dist_check(dist, c, e, s);
    } else if similar3(ne, e, s) && !similar3(n, e, s) && !similar(ne, sw) {
        // Forward steep 2:1 slant
        if similar(c, se) && higher(c, s) {
            // Single pixel wide diagonal, don't flip
        } else if higher(c, s) || (similar(w, s) && !similar(c, se) && !higher(c, s)) {
            // Priority edge cases, flip
            flip = true;
        }

        let dist = midpoint(
            flip,
            flipped_point,
            point_dir, 
            vec2f(0.0, -0.5),
            vec2f(-1.0, 1.5),
            vec2f(1.0, -0.5),
            vec2f(0.0, 1.5),
        );

        return dist_check(dist, c, e, s);
    } else if similar(e, s) {
        // 45 degrees diagonal
        if similar(c, se) && higher(c, e) {
            // Single pixel wide diagonal, don't flip

            if !similar(c, ss) && !similar(c, ee) {
                // Line against triple color stripe edge case
                flip = true;
            }
        } else if higher(c, e) || (similar(c, w) && similar4(w, e, s, n)) {
            flip = true;
        }

        // Single pixel 2:1 slope, don't flip
        if (((similar(e, sw) && similar3(n, e, se)) || (similar(ne, s) && similar3(w, s, se))) && !similar(c, se)) {
			flip = true;
		} 

        let dist = midpoint(
            flip,
            flipped_point,
            point_dir, 
            vec2f(1.0, -1.0),
            vec2f(-1.0, 1.0),
            vec2f(1.0, 0.0),
            vec2f(0.0, 1.0),
        );

        return dist_check(dist, c, e, s);
    } else if similar3(ee, se, s) && !similar3(ee, se, c) && !similar(nee, s) {
        // Far corner of shallow slant
        if similar(e, see) && higher(e, ee) {
            // Single pixel wide diagonal, don't flip
        } else if higher(e, ee) || (similar(ne, ee) && !similar(e, see) && !higher(e, ne)) {
            // Priority edge cases, flip
            flip = true;
        }

        let dist = midpoint(
            flip,
            flipped_point,
            point_dir, 
            vec2f(2.5, -1.0),
            vec2f(0.5, 0.0),
            vec2f(2.5, 0.0),
            vec2f(0.5, 1.0),
        );

        return dist_check(dist, e, ee, ss);
    } else if similar3(e, se, ss) && !similar3(c, se, ss) && !similar(e, ssw) {
        // Far corner of steep slant
        if similar(s, sse) && higher(s, ss) {
            // Single pixel wide diagonal, don't flip
        } else if higher(s, ss) || (similar(sw, ss) && !similar(s, sse) && !higher(s, ss)) {
            // Priority edge cases, flip
            flip = true;
        }

        let dist = midpoint(
            flip,
            flipped_point,
            point_dir, 
            vec2f(0.0, 0.5),
            vec2f(-1.0, 2.5),
            vec2f(1.0, 0.5),
            vec2f(0.0, 2.5),
        );

        return dist_check(dist, s, se, ss);
    }

    return vec4(-1.0);
}

// Scale3x
@fragment
fn fs_main_scale3x(in: VertexOutput) -> @location(0) vec4f {
    // Take the sample of the exact pixel
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Don't apply the algorithm when no rotations or skewing occurs
    if in.only_translated_or_reflected == 1.0 {
        return c;
    }

    // Offset of the UV within the pixel
    let subpixel = fract(in.tex_coords * ATLAS_TEXTURE_SIZE);
    
    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let nw = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, PIXEL_OFFSET));
    let n = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, PIXEL_OFFSET));
    let ne = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET));
    let w = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, 0.0));
    let e = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET, 0.0));
    let sw = textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2f(PIXEL_OFFSET));
    let s = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, -PIXEL_OFFSET));
    let se = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET, -PIXEL_OFFSET));

    // Apply a Scale3x block
    return scale3x(nw, n, ne, w, c, e, sw, s, se, subpixel);
}

// Diag2x
@fragment
fn fs_main_diag2x(in: VertexOutput) -> @location(0) vec4f {
    // Take the sample of the exact pixel
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Don't apply the algorithm when no rotations or skewing occurs
    if in.only_translated_or_reflected == 1.0 {
        return c;
    }

    // Offset of the UV within the pixel
    let subpixel = fract(in.tex_coords * ATLAS_TEXTURE_SIZE);
    
    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let nw = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, PIXEL_OFFSET));
    let n = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, PIXEL_OFFSET));
    let ne = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET));
    let w = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, 0.0));
    let e = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET, 0.0));
    let sw = textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2f(PIXEL_OFFSET));
    let s = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, -PIXEL_OFFSET));
    let se = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET, -PIXEL_OFFSET));

    // Apply a Diag2x block
    return diag2x(nw, n, ne, w, c, e, sw, s, se, subpixel);
}

// Scale2x
@fragment
fn fs_main_scale2x(in: VertexOutput) -> @location(0) vec4f {
    // Take the sample of the exact pixel
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Don't apply the algorithm when no rotations or skewing occurs
    if in.only_translated_or_reflected == 1.0 {
        return c;
    }

    // Offset of the UV within the pixel
    let subpixel = fract(in.tex_coords * ATLAS_TEXTURE_SIZE);
    
    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let n = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, PIXEL_OFFSET));
    let w = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, 0.0));
    let e = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(PIXEL_OFFSET, 0.0));
    let s = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(0.0, -PIXEL_OFFSET));

    // Apply a Scale2x block
    return scale2x(n, w, c, e, s, subpixel);
}

// Torcado's cleanEdge
@fragment
fn fs_main_clean_edge(in: VertexOutput) -> @location(0) vec4f {
    // Take the sample of the exact pixel
    let c = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Don't apply the algorithm when no rotations or skewing occurs
    if in.only_translated_or_reflected == 1.0 {
        return c;
    }

    // Offset of the UV within the pixel
    let local = fract(in.tex_coords * ATLAS_TEXTURE_SIZE);
    let point_dir = sign(local - 0.5);

    // Sample the pixels around the center with (n)orth, (e)ast, (s)outh, (w)est
    let nnw = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET,-PIXEL_OFFSET_2) * point_dir);
	let nn  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( 0.0,-PIXEL_OFFSET_2) * point_dir);
	let nne = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET,-PIXEL_OFFSET_2) * point_dir);
	
	let nww = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET_2,-PIXEL_OFFSET_2) * point_dir);
	let nw  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET,-PIXEL_OFFSET) * point_dir);
	let n   = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( 0.0,-PIXEL_OFFSET) * point_dir);
	let ne  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET,-PIXEL_OFFSET) * point_dir);
	let nee = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET_2,-PIXEL_OFFSET) * point_dir);
	
	let ww  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET_2, 0.0) * point_dir);
	let w   = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, 0.0) * point_dir);
	let e   = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET, 0.0) * point_dir);
	let ee  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET_2, 0.0) * point_dir);
	
	let sww = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET_2, PIXEL_OFFSET) * point_dir);
	let sw  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, PIXEL_OFFSET) * point_dir);
	let s   = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( 0.0, PIXEL_OFFSET) * point_dir);
	let se  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET, PIXEL_OFFSET) * point_dir);
	let see = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET_2, PIXEL_OFFSET) * point_dir);
	
	let ssw = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f(-PIXEL_OFFSET, PIXEL_OFFSET_2) * point_dir);
	let ss  = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( 0.0, PIXEL_OFFSET_2) * point_dir);
	let sse = textureSample(t_diffuse, s_diffuse, in.tex_coords + vec2f( PIXEL_OFFSET, PIXEL_OFFSET_2) * point_dir);

    // North slices
	let n_col = slice_dist(local, vec2f(1.0,-1.0), point_dir, s, se, see, w, c, e, ee, nw, n, ne, nee, nnw, nn, nne);

    // West slices
	let w_col = slice_dist(local, vec2f(-1.0, 1.0), point_dir, n, nw, nww, e, c, w, ww, se, s, sw, sww, sse, ss, ssw);

    // Corner slices
    let c_col = slice_dist(local, vec2f(1.0, 1.0), point_dir, n, ne, nee, w, c, e, ee, sw, s, se, see, ssw, ss, sse);

    // if n_col.r >= 0.0 { return n_col };
    // if w_col.r >= 0.0 { return w_col };
    // if c_col.r >= 0.0 { return c_col };
    // return c;
    return mix(
        mix(
            mix(
                c,
                c_col,
                step(0.0, c_col.r)
            ),
            w_col,
            step(0.0, w_col.r)
        ),
        n_col,
        step(0.0, n_col.r)
    );
}

