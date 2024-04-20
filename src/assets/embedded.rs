//! Assets that have been embedded during compile time.

use std::io::Cursor;

use glamour::{Point2, Rect, Size2};
use hashbrown::HashMap;
use imgref::Img;
use png::Decoder;

use crate::{
    assets::Id,
    graphics::{atlas::StaticAtlas, gpu::Gpu},
};

/// Compile time assets that haven't been parsed yet.
pub struct EmbeddedAssets {
    /// All loaded assets by parsed path.
    pub assets: &'static [EmbeddedRawAsset],
    /// Diced raw texture atlas.
    pub atlas: EmbeddedRawStaticAtlas,
}

/// Single embedded asset in the binary.
pub struct EmbeddedRawAsset {
    /// Parsed ID, excludes the file extension.
    pub id: &'static str,
    /// File extension excluding the first `'.'`.
    pub extension: &'static str,
    /// Raw bytes of the asset.
    pub bytes: &'static [u8],
}

/// Embedded diced sprite atlas in the binary.
#[derive(Debug)]
pub struct TextureMapping {
    /// U & V coordinates for source on the diced texture.
    pub diced: Point2<u16>,
    /// U & V coordinates for target on the source texture.
    pub texture: Point2<u16>,
    /// Size of the diced sub image.
    pub size: Size2<u16>,
}

/// Embedded diced sprite atlas in the binary.
pub struct EmbeddedRawStaticAtlas {
    /// PNG bytes of the diced atlas.
    pub diced_atlas_png_bytes: Vec<u8>,
    /// Rectangle mapping for the textures.
    ///
    /// Structure is `[texture_index, diced_u, diced_v, texture_u, texture_v, width, height]`.
    pub texture_mappings: Vec<TextureMapping>,
    /// All IDS of the textures.
    ///
    /// Index determines the position.
    pub texture_ids: Vec<&'static str>,
    /// Full items on the atlas.
    ///
    /// Index determines the position.
    pub texture_rects: Vec<Rect<f32>>,
    /// Fitted width of the atlas.
    pub width: u16,
    /// Fitted height of the atlas.
    pub height: u16,
}

impl EmbeddedRawStaticAtlas {
    /// Parse into a static sprite atlas.
    pub(crate) fn parse_and_upload(self, gpu: &Gpu) -> StaticAtlas {
        // First decode the PNG bytes

        // Create a consuming cursor from the bytes
        let cursor = Cursor::new(self.diced_atlas_png_bytes);

        // Decode the PNG
        let decoder = Decoder::new(cursor);
        // Start parsing the PNG
        let mut reader = decoder.read_info().unwrap();

        // Allocate the buffer for the pixels
        let mut png_pixels = vec![0u32; reader.output_buffer_size()];

        // Read the bytes into the buffer
        let info = reader
            .next_frame(bytemuck::cast_slice_mut(&mut png_pixels))
            .unwrap();

        // Treat the 4 color components as a single numeric value
        let png = Img::new(png_pixels, info.width as usize, info.height as usize);

        // Lastly create a new GPU atlas texture

        // TODO: use different smaller size
        let size = Size2::new(4096, 4096);

        // Create the atlas
        let atlas = StaticAtlas::new(size, self.texture_rects, gpu);

        // Upload all sections
        for mapping in self.texture_mappings {
            // Copy the pixels from the slice into the target
            let diced_texture = png.sub_image(
                mapping.diced.x as usize,
                mapping.diced.y as usize,
                mapping.size.width as usize,
                mapping.size.height as usize,
            );
            let diced_texture_pixels = diced_texture.pixels().collect::<Vec<_>>();

            // Push the slice
            atlas.update(
                Rect::new(
                    Point2::new(mapping.texture.x as u32, mapping.texture.y as u32),
                    Size2::new(mapping.size.width as u32, mapping.size.height as u32),
                ),
                &diced_texture_pixels,
                &gpu.queue,
            );
        }

        atlas
    }

    /// Create a texture mapping to atlas index table.
    pub(crate) fn texture_id_to_atlas_id_map(&self) -> HashMap<Id, u16> {
        self.texture_ids
            .iter()
            .enumerate()
            .map(|(index, id)| ((*id).into(), index as u16))
            .collect()
    }

    /// Create a texture mapping to size.
    pub(crate) fn texture_id_to_size_map(&self) -> HashMap<Id, Size2<u32>> {
        self.texture_ids
            .iter()
            .zip(self.texture_rects.iter())
            .map(|(id, rect)| {
                (
                    (*id).into(),
                    Size2::new(rect.size.width as u32, rect.size.height as u32),
                )
            })
            .collect()
    }
}
