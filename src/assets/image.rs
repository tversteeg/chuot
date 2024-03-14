//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::borrow::Cow;

use assets_manager::{loader::Loader, Asset, BoxedError};
use image::{DynamicImage, ImageFormat, RgbaImage};
use vek::Extent2;

use crate::graphics::texture::Texture;

/// Core of a sprite loaded from disk.
#[derive(Debug)]
pub(crate) struct Image {
    /// Image data.
    image: DynamicImage,
}

impl Asset for Image {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = ImageLoader;
}

impl Texture for Image {
    fn size(&self) -> Extent2<u32> {
        Extent2::new(self.image.width(), self.image.height())
    }

    fn to_rgba_image(&self) -> RgbaImage {
        self.image.to_rgba8()
    }
}

/// Image asset loader.
pub(crate) struct ImageLoader;

impl Loader<Image> for ImageLoader {
    fn load(content: Cow<[u8]>, ext: &str) -> Result<Image, BoxedError> {
        assert_eq!(ext.to_lowercase(), "png");

        // Load the PNG image
        let image = image::load_from_memory_with_format(&content, ImageFormat::Png)?;

        Ok(Image { image })
    }
}
