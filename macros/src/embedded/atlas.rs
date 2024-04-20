//! Create a single big texture atlas from all image files in the assets folder.

use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    str::FromStr,
};

use packr2::{PackerConfig, RectInput, RectOutput, Size, SplitPacker};
use png::{
    BitDepth, ColorType, Decoder, Encoder, ScaledFloat, SourceChromaticities, Transformations,
};
use proc_macro::TokenStream;
use quote::quote;
use sprite_dicing::{DicedSprite, Pivot, Pixel, Prefs, SourceSprite, Texture};

/// Parse a list of textures with their paths.
pub fn parse_textures(textures: &[(String, PathBuf)]) -> TokenStream {
    // Keep a single buffer to reduce allocations for each image
    let mut buf = Vec::new();

    // Read each texture from disk and convert it into a texture for the sprite dicing algorithm
    let source_sprites = textures
        .iter()
        .enumerate()
        .map(|(index, (id, path))| {
            // Read the PNG
            let mut decoder = Decoder::new(File::open(path).expect("Could not open texture"));

            // Discard text chunks
            decoder.set_ignore_text_chunk(true);
            // Make it faster by not checking if it's correct
            decoder.ignore_checksums(true);

            // Convert indexed images to RGBA
            decoder.set_transformations(
                Transformations::normalize_to_color8() | Transformations::ALPHA,
            );

            let mut reader = decoder.read_info().expect("Could not read PNG info");

            // Ensure we can use the PNG colors
            let (color_type, bits) = reader.output_color_type();
            if color_type != ColorType::Rgba || bits != BitDepth::Eight {
                panic!("PNG is not 8 bit RGB with an alpha channel");
            }

            // Resize the texture buffer so it fits the output
            buf.resize(reader.output_buffer_size(), 0);

            // Read the PNG frame, animated PNGs are not supported
            let info = reader
                .next_frame(&mut buf)
                .expect("Error reading PNG frame");
            let width = info.width;
            let height = info.height;

            // Grab the bytes
            let bytes = &buf[..info.buffer_size()];

            // Convert RGBA bytes to pixels
            let pixels = bytes
                .chunks_exact(4)
                .map(|rgba| Pixel::from_raw(rgba.try_into().unwrap()))
                .collect();

            // Create a texture for the sprite dicing algorithm
            let texture = Texture {
                width,
                height,
                pixels,
            };

            // Create a simple ID from the index
            let id = index.to_string();

            // Ensure each sprite doesn't get offset vertex coordinates
            let pivot = Some(Pivot::new(0.0, 0.0));

            // Create the source sprite structure
            SourceSprite { id, texture, pivot }
        })
        .collect::<Vec<_>>();

    // Pack all rectangles in the atlas
    let (atlas_width, atlas_height, offsets) = pack(&source_sprites);

    // Dice the textures
    let prefs = Prefs {
        // Smallest block size, smaller sizes result in smaller resulting images but with more fragments and longer compile time
        unit_size: 8,
        // Tightly pack the image
        padding: 0,
        // Keep all units in pixels, required for us to properly parse vertex positions
        ppu: 1.0,
        ..Default::default()
    };
    let diced = sprite_dicing::dice(&source_sprites, &prefs).expect("Error dicing textures");

    assert_eq!(
        diced.atlases.len(),
        1,
        "Texture didn't fit in diced sprite size"
    );
    // Encode the generated diced atlas as a PNG
    let png_bytes = encode_png(&diced.atlases[0]);

    // Parse each texture
    let texture_mappings = diced.sprites.into_iter().flat_map(|DicedSprite {
        id,
        vertices,
        uvs,
        indices,
        rect,
        pivot,
        ..
    }| {
        // Recalculate the mesh positions back to the textures

        // Convert ID back to number
        let texture_index = usize::from_str(&id).unwrap();

        // Get the offset in the output atlas
        let (offset_u, offset_v) = offsets[texture_index];

        assert_eq!(pivot, Pivot::new(0.0, 0.0), "Diced sprite pivot changed");

        // Every vertex for every quad is unique and are added incrementally, so we don't need to actually index them
        assert_eq!(indices.len() / 6 * 4, vertices.len(), "Sprite dicing algorithm changed, can't assume every vertex is only used by a single quad anymore");

        // We assume the dicing algorithm keeps a specific structure
        assert_eq!(vertices[0].x, vertices[1].x);
        assert_eq!(vertices[2].x, vertices[3].x);
        assert_eq!(vertices[0].y, vertices[3].y);
        assert_eq!(vertices[1].y, vertices[2].y);
        assert!(vertices[0].x < vertices[2].x);
        assert!(vertices[0].y < vertices[2].y);

        // Because of this assumption we only need to take 2 vertices for each rectangle
        let quad_vertices = vertices.chunks_exact(4).map(|vertices| (vertices[0].clone(), vertices[2].clone())).collect::<Vec<_>>();

        // Convert all coordinates into mapped rectangles
        quad_vertices
            .into_iter()
            // We only need to take the top left UV coordinate because we can already calculate the width and height from the vertices
            .zip(uvs.into_iter().step_by(4))
            .map(move |((top_left, bottom_right), uv)| {
                // Get the position on the original texture
                let top_left_u = top_left.x as u16;
                let top_left_v = top_left.y as u16;

                // Apply offsets
                let texture_u = top_left_u + offset_u;
                let texture_v = top_left_v + offset_v;

                // Get the size, apply to both the source and the target
                let width = bottom_right.x as u16 - top_left_u;
                let height = bottom_right.y as u16 - top_left_v;

                // Get the position on the newly diced map
                let diced_u = (uv.u * rect.width) as u16;
                let diced_v = (uv.v * rect.height) as u16;

                // Convert to array to save space
                quote! {
                    [#texture_index, #diced_u, #diced_v, #texture_u, #texture_v, #width, #height]
                }
            })
    }).collect::<Vec<_>>();

    // Create the object from the tightly packed arrays
    quote! {
        pixel_game_lib::assets::StaticRawSpriteAtlas {
            diced_atlas_png_bytes: vec![#(#png_bytes),*],
            texture_mappings: vec![#(#texture_mappings),*],
        }
    }
    .into()
}

/// Encode a pixel texture to a PNG file.
fn encode_png(texture: &Texture) -> Vec<u8> {
    // PNG output bytes
    let mut bytes = Vec::new();

    {
        // Encode the PNG
        let mut encoder = Encoder::new(&mut bytes, texture.width, texture.height); // Width is 2 pixels and height is 1.
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);

        // Write the PNG header to disk
        let mut writer = encoder.write_header().unwrap();

        // Write the texture data to disk
        writer
            .write_image_data(bytemuck::cast_slice(
                &texture
                    .pixels
                    .iter()
                    .map(|p| p.to_raw())
                    .collect::<Vec<_>>(),
            ))
            .expect("Error writing PNG file to disk");
    }

    bytes
}

/// Pack all rectangles into a single atlas.
///
/// # Returns
///
/// - Size of the atlas.
/// - Offsets for texture inside the packed resulting atlas.
fn pack(source_sprites: &[SourceSprite]) -> (u16, u16, Vec<(u16, u16)>) {
    // Convert textures to inputs for the packr algorithm
    let mut inputs = source_sprites
        .iter()
        .enumerate()
        .map(|(key, source_sprite)| {
            RectInput {
                size: Size::new(source_sprite.texture.width, source_sprite.texture.height),
                // Use the index as the key
                key,
            }
        })
        .collect();

    // Pack in a maximum atlas of 4096x4096
    let packer = SplitPacker::new(PackerConfig {
        max_width: 4096,
        max_height: 4096,
        allow_flipping: false,
    });
    let mut outputs = packr2::pack(&mut inputs, packer);

    // Sort the outputs by the key, so the index matches again
    outputs.sort_by_key(|output| output.key);

    // Get the size of the atlas
    let width = outputs
        .iter()
        .map(|output| output.rect.x + output.rect.w)
        .max()
        .unwrap_or_default() as u16;
    let height = outputs
        .iter()
        .map(|output| output.rect.y + output.rect.h)
        .max()
        .unwrap_or_default() as u16;

    // Only take the offsets
    let offsets = outputs
        .into_iter()
        .map(|output| {
            assert_eq!(output.atlas, 0, "Multiple atlasses not supported yet");

            (output.rect.x as u16, output.rect.y as u16)
        })
        .collect();

    (width, height, offsets)
}
