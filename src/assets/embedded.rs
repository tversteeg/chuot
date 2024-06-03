//! Assets that have been embedded during compile time.

use std::io::Cursor;

use glamour::{Point2, Rect, Size2};
use hashbrown::HashMap;
use imgref::ImgVec;
use png::Decoder;

use crate::{
    assets::{image::Image, Id},
    graphics::{atlas::Atlas, gpu::Gpu},
};

use super::loader::Loader;

/// Compile time assets that haven't been parsed yet.
pub struct EmbeddedAssets {
    /// All loaded assets by parsed path.
    pub assets: &'static [EmbeddedRawAsset],
    /// Diced raw texture atlas.
    pub atlas: EmbeddedRawStaticAtlas,
}

impl EmbeddedAssets {
    /// Get the raw texture atlas.
    pub(crate) fn atlas(self) -> EmbeddedRawStaticAtlas {
        self.atlas
    }
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
    /// Texture index in the atlas.
    #[cfg(feature = "read-image")]
    pub index: usize,
    /// U & V coordinates for reconstructing the source texture.
    #[cfg(feature = "read-image")]
    pub source: Point2<u16>,
}

/// Embedded diced sprite atlas in the binary.
pub struct EmbeddedRawStaticAtlas {
    /// PNG bytes of the diced atlas.
    pub diced_atlas_png_bytes: &'static [u8],
    /// Rectangle mapping for the textures.
    ///
    /// Structure is `[texture_index, diced_u, diced_v, texture_u, texture_v, width, height]`.
    pub texture_mappings: &'static [TextureMapping],
    /// All IDS of the textures.
    ///
    /// Index determines the position.
    pub texture_ids: &'static [&'static str],
    /// Full items on the atlas.
    ///
    /// Index determines the position.
    pub texture_rects: &'static [Rect],
    /// Fitted width of the atlas.
    pub width: u16,
    /// Fitted height of the atlas.
    pub height: u16,
}

impl EmbeddedRawStaticAtlas {
    /// Parse into a static sprite atlas.
    pub(crate) fn parse_and_upload(self, gpu: &Gpu) -> Atlas {
        // First decode the PNG bytes

        // TODO: move this call to a higher level so it's only called once when `read-image` feature flag is enabled
        let png = self.compressed_png();

        // Then create a new GPU atlas texture

        // Create the atlas
        let atlas = Atlas::new(self.texture_rects.to_vec(), gpu);

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
            atlas.update_pixels_raw_offset(
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

    /// Read the images from the assets into memory.
    #[cfg(feature = "read-image")]
    fn to_memory(&self) -> HashMap<Id, ImgVec<u32>> {
        log::debug!("Reading images into memory because the `read-image` feature flag is enabled");

        // TODO: move this call to a higher level so it's only called once when `read-image` feature flag is enabled
        let png = self.compressed_png();

        // Pre-allocate the images that will be filled by the diced slices
        let mut images = self
            .texture_rects
            .iter()
            .map(|texture_rect| {
                let width = texture_rect.width() as usize;
                let height = texture_rect.height() as usize;

                ImgVec::new(vec![0_u32; width * height], width, height)
            })
            .collect::<Vec<_>>();

        // Read and push all sections
        for mapping in self.texture_mappings {
            let width = mapping.size.width as usize;
            let height = mapping.size.height as usize;

            // Source region to copy
            let diced_texture = png.sub_image(
                mapping.diced.x as usize,
                mapping.diced.y as usize,
                width,
                height,
            );
            // Target region to copy to
            let mut target_texture = images[mapping.index].sub_image_mut(
                mapping.source.x as usize,
                mapping.source.y as usize,
                width,
                height,
            );

            // Copy the pixels
            diced_texture
                .rows()
                .zip(target_texture.rows_mut())
                .for_each(|(source, target)| {
                    target.copy_from_slice(source);
                });
        }

        // Create a hashmap from the array
        images
            .into_iter()
            .zip(self.texture_ids.iter())
            .map(|(image, id)| (Id::new_static(id), image))
            .collect()
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

    /// Read the compressed atlas PNG.
    pub(crate) fn compressed_png(&self) -> ImgVec<u32> {
        // Create a consuming cursor from the bytes
        let cursor = Cursor::new(self.diced_atlas_png_bytes);

        // Decode the PNG
        let decoder = Decoder::new(cursor);
        // Start parsing the PNG
        let mut reader = decoder.read_info().unwrap();

        // Allocate the buffer for the pixels
        let mut png_pixels = vec![0_u32; reader.output_buffer_size()];

        // Read the bytes into the buffer
        let info = reader
            .next_frame(bytemuck::cast_slice_mut(&mut png_pixels))
            .unwrap();

        // Treat the 4 color components as a single numeric value
        ImgVec::new(png_pixels, info.width as usize, info.height as usize)
    }
}

/// Asset source for all assets embedded in the binary.
pub struct AssetSource {
    /// Static texture ID to atlas ID mapping.
    static_texture_id_to_atlas_id: HashMap<Id, u16>,
    /// Static texture ID to size.
    static_texture_id_to_size: HashMap<Id, Size2<u32>>,
    /// All loaded assets by parsed path.
    assets: &'static [EmbeddedRawAsset],
    /// Pixels of the images.
    #[cfg(feature = "read-image")]
    images: HashMap<Id, ImgVec<u32>>,
}

impl AssetSource {
    /// Construct a new source from embedded raw assets.
    pub(crate) fn new(assets: &'static [EmbeddedRawAsset], atlas: &EmbeddedRawStaticAtlas) -> Self {
        let static_texture_id_to_atlas_id = atlas.texture_id_to_atlas_id_map();
        let static_texture_id_to_size = atlas.texture_id_to_size_map();

        Self {
            assets,
            static_texture_id_to_atlas_id,
            static_texture_id_to_size,
            #[cfg(feature = "read-image")]
            images: atlas.to_memory(),
        }
    }

    /// Load a new asset based on the loader.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`Loadable`] function to load the asset with.
    ///
    /// # Returns
    ///
    /// - An asset when it's found and has the correct type.
    /// - `None` if the asset could not be found.
    ///
    /// # Panics
    ///
    /// - When loading the asset fails.
    #[track_caller]
    pub fn load_if_exists<L, T>(&self, id: &Id) -> Option<T>
    where
        L: Loader<T>,
    {
        log::debug!(
            "Loading part of asset '{id}' with extension '{}'",
            L::EXTENSION
        );

        Some(L::load(self.raw_asset(id, L::EXTENSION)?))
    }

    /// Load a new image based on the loader.
    ///
    /// This is a special case because images need to be uploaded to the GPU at a later stage.
    ///
    /// # Arguments
    ///
    /// * `id` - Image asset ID passed to the [`Loadable`] function to load the image with.
    ///
    /// # Returns
    ///
    /// - An image when it's found and has the correct type.
    /// - `None` if the image asset could not be found.
    #[track_caller]
    pub(crate) fn get_or_load_image_if_exists(&self, id: &Id) -> Option<Image> {
        log::debug!("Loading image asset '{id}'");

        // Get or load the image
        self.static_texture_id_to_atlas_id
            .get(id)
            .and_then(|atlas_id| {
                self.static_texture_id_to_size.get(id).map(|size| Image {
                    atlas_id: *atlas_id,
                    size: *size,
                    #[cfg(feature = "read-image")]
                    image_source: self.images[id].clone(),
                })
            })
    }

    /// Get the bytes of an asset that matches the ID and the extension.
    fn raw_asset(&self, id: &Id, extension: &str) -> Option<&'static [u8]> {
        self.assets.iter().find_map(|raw_asset| {
            if raw_asset.id == id && raw_asset.extension == extension {
                Some(raw_asset.bytes)
            } else {
                None
            }
        })
    }
}
