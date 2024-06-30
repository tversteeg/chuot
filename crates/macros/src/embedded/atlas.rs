//! Create a single big texture atlas from all image files in the assets folder.

use std::{fs::File, path::PathBuf, time::Duration};

use oxipng::Options;
use phf_codegen::Map;
use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use proc_macro2::TokenStream;
use quote::quote;
use sprite_dicing::{DicedSprite, Pivot, Pixel, Prefs, SourceSprite, Texture};

/// Parse a list of textures with their paths.
pub fn parse_textures(textures: &[(String, PathBuf)]) -> TokenStream {
    // Keep a single buffer to reduce allocations for each image
    let mut buf = Vec::new();

    // Read each texture from disk and convert it into a texture for the sprite dicing algorithm
    let source_sprites = textures
        .iter()
        .map(|(id, path)| {
            // Read the PNG
            let mut decoder = Decoder::new(File::open(path).expect("Error opening texture"));

            // Discard text chunks
            decoder.set_ignore_text_chunk(true);
            // Make it faster by not checking if it's correct
            decoder.ignore_checksums(true);

            // Convert indexed images to RGBA
            decoder.set_transformations(
                Transformations::normalize_to_color8() | Transformations::ALPHA,
            );

            let mut reader = decoder.read_info().expect("Error reading PNG info");

            // Ensure we can use the PNG colors
            let (color_type, bits) = reader.output_color_type();
            assert!(
                !(color_type != ColorType::Rgba || bits != BitDepth::Eight),
                "Error reading PNG: image is not 8 bit RGBA but {} bit {}: {}",
                match bits {
                    BitDepth::One => 1,
                    BitDepth::Two => 2,
                    BitDepth::Four => 4,
                    BitDepth::Eight => 8,
                    BitDepth::Sixteen => 16,
                },
                match color_type {
                    ColorType::Grayscale => "grayscale",
                    ColorType::Rgb => "RGB",
                    ColorType::Indexed => "indexed",
                    ColorType::GrayscaleAlpha => "grayscale+alpha",
                    ColorType::Rgba => "RGBA",
                },
                path.display(),
            );

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

            // Use the ID as to find the diced parts later
            let id = id.to_owned();

            // Ensure each sprite doesn't get offset vertex coordinates
            let pivot = Some(Pivot::new(0.0, 0.0));

            // Create the source sprite structure
            SourceSprite { id, texture, pivot }
        })
        .collect::<Vec<_>>();

    // Dice the textures
    let prefs = Prefs {
        // Smallest block size, smaller sizes result in smaller resulting images but with more fragments and longer compile time
        unit_size: 16,
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
    // Get the result texture
    let diced_atlas = &diced.atlases[0];

    // Get the size of the generated atlas
    let atlas_width = diced_atlas.width as u16;
    let atlas_height = diced_atlas.height as u16;

    // Encode the generated diced atlas as a PNG
    let png_bytes = encode_png(diced_atlas);

    // Create the result texture map
    let mut textures_map = Map::<&str>::new();

    // Create the textures for the map
    for (index, source_sprite) in source_sprites.iter().enumerate() {
        let texture = texture(
            &source_sprite.id,
            source_sprite.texture.width as u16,
            source_sprite.texture.height as u16,
            // Use the index as the reference, it's trivial which is chosen as long as it's unique
            index as u16,
            &diced.sprites,
            diced_atlas.width as f32,
            diced_atlas.height as f32,
        );

        textures_map.entry(&source_sprite.id, &texture.to_string());
    }

    // Create the result code
    let textures_map: TokenStream = textures_map.build().to_string().parse().unwrap();

    // Create the object from the tightly packed arrays
    quote! {
        {
            static MAP: &chuot::assets::source::EmbeddedRawStaticAtlas = &chuot::assets::source::EmbeddedRawStaticAtlas {
                diced_atlas_png_bytes: {
                    static BYTES: &[u8] = &[#(#png_bytes),*];

                    BYTES
                },
                textures: {
                    static TEXTURES: &phf::Map<&'static str, chuot::assets::source::EmbeddedTexture> = &#textures_map;

                    TEXTURES
                },
                width: #atlas_width,
                height: #atlas_height,
            };

            MAP
        }
    }
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

    // Optimize the PNG
    oxipng::optimize_from_memory(
        &bytes,
        &Options {
            // Always write to output
            force: true,
            // Also simplify the alpha channel, removes color info for transparent pixels
            optimize_alpha: true,
            // Never make it grayscale
            grayscale_reduction: false,
            // Reducing the color type makes the PNG loader not work for some reason
            color_type_reduction: false,
            // Don't optimize for more than a minute
            timeout: Some(Duration::from_secs(60)),
            ..Default::default()
        },
    )
    .expect("Error optimizing PNG")
}

/// Construct an single texture.
fn texture(
    id: &str,
    width: u16,
    height: u16,
    reference: u16,
    diced_sprites: &[DicedSprite],
    diced_width: f32,
    diced_height: f32,
) -> TokenStream {
    // Parse each texture
    let texture_mappings = diced_sprites
        .iter()
        .filter(|diced| diced.id == id)
        .flat_map(|DicedSprite {
        vertices,
        uvs,
        indices,
        ..
    }| {
        // Recalculate the mesh positions back to the textures

        // Every vertex for every quad is unique and are added incrementally, so we don't need to actually index them
        assert_eq!(indices.len() / 6 * 4, vertices.len(), "Sprite dicing algorithm changed, can't assume every vertex is only used by a single quad anymore");

        // We assume the dicing algorithm keeps a specific structure
        // Because of this assumption we only need to take 2 vertices for each rectangle
        assert!(vertices[0].x < vertices[2].x);
        assert!(vertices[0].y < vertices[2].y);

        // Convert all coordinates into mapped rectangles
        vertices
            .chunks_exact(4)
            .map(|vertices| {
                assert!(vertices.len() > 2);

                (vertices[0].clone(), vertices[2].clone())
            })
            // We only need to take the top left UV coordinate because we can already calculate the width and height from the vertices
            .zip(uvs.iter().step_by(4))
            .map(move |((top_left, bottom_right), uv)| {
                // Get the position on the original texture
                let texture_u = top_left.x as u16;
                let texture_v = top_left.y as u16;

                // Get the size, apply to both the source and the target
                let width = bottom_right.x as u16 - texture_u;
                let height = bottom_right.y as u16 - texture_v;

                // Get the position on the newly diced map
                let diced_u = (uv.u * diced_width).round() as u16;
                let diced_v = (uv.v * diced_height).round() as u16;

                quote! {
                    chuot::assets::source::EmbeddedTextureDiceMapping {
                        diced_u: #diced_u,
                        diced_v: #diced_v,
                        texture_u: #texture_u,
                        texture_v: #texture_v,
                        width: #width,
                        height: #height,
                    }
                }
            })
    }).collect::<Vec<_>>();

    quote! {
        chuot::assets::source::EmbeddedTexture {
            width: #width,
            height: #height,
            reference: #reference,
            diced: {
                static MAPPINGS: &[chuot::assets::source::EmbeddedTextureDiceMapping] = &[#(#texture_mappings),*];

                MAPPINGS
            }
         }
    }
}
