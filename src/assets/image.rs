//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use glamour::Size2;

use crate::graphics::texture::{Texture, TextureRef};

use super::{
    loader::png::{PngLoader, PngReader},
    AssetSource, Id, Loadable,
};

/// Core of a sprite loaded from disk.
pub struct Image {
    /// PNG raw data.
    png: PngReader,

    /// Size of the image in pixels.
    size: Size2<u32>,
}

/*
impl Image {
    /// Read the image and split into equal horizontal parts.
    pub(crate) fn into_horizontal_parts(self, part_width: u32) -> Vec<Image> {
        let size = self.size();

        // Ensure that the image can be split into equal parts
        assert!(
            size.width % part_width == 0,
            "Cannot split image into equal horizontal parts of {part_width} pixels"
        );

        // Get the raw pixels, by either reading the PNG or collecting the already raw data
        let raw_bytes = self.into_rgba_image();

        // Loop over each section, recreating the data
        let (width, height) = (size.width as usize, size.height as usize);
        let sub_images = size.width / part_width;
        (0..sub_images)
            .map(|index| {
                // Setup the buffer
                let pixels = (part_width * size.height) as usize;
                let mut data = vec![0u32; pixels];

                // Copy the image slices
                for y in 0..height {
                    let bytes_to_copy = part_width as usize;

                    let src_start = y * width + (index * part_width) as usize;
                    let src_end = src_start + bytes_to_copy;

                    let dst_start = y * (part_width as usize);
                    let dst_end = dst_start + bytes_to_copy;

                    data[dst_start..dst_end].copy_from_slice(&raw_bytes[src_start..src_end]);
                }

                // Create the new image
                Image::Raw {
                    size: Size2::new(part_width, size.height),
                    data,
                }
            })
            .collect()
    }
}
*/

impl Texture for Image {
    fn size(&self) -> Size2<u32> {
        self.size
    }
}

impl Loadable for Image {
    type Upload = PngReader;

    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<(Self, Self::Upload)>
    where
        Self: Sized,
    {
        let reader = asset_source.load_if_exists::<PngLoader, _>(id)?;
        let (width, height) = reader.info().size();

        let size = Size2::new(width, height);
        let id = id.to_owned();

        Some((Self { size, png }, reader))
    }
}
