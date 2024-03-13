//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::borrow::Cow;

use assets_manager::{loader::Loader, Asset, BoxedError};
use blit::{BlitBuffer, ToBlitBuffer};
use image::{ImageFormat, RgbaImage};
use vek::Extent2;

use crate::graphics::texture::{Texture, UploadedTextureState};

/// Core of a sprite loaded from disk.
#[derive(Debug)]
pub(crate) struct Image {
    /// Image data.
    data: BlitBuffer,
    /// GPU texture state.
    texture_state: Option<UploadedTextureState>,
}

impl Asset for Image {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = ImageLoader;
}

impl Texture for Image {
    fn size(&self) -> Extent2<u32> {
        Extent2::new(self.data.width(), self.data.height())
    }

    fn pixels(&self) -> &[u32] {
        self.data.pixels()
    }
}

/// Image asset loader.
pub(crate) struct ImageLoader;

impl Loader<Image> for ImageLoader {
    fn load(content: Cow<[u8]>, ext: &str) -> Result<Image, BoxedError> {
        assert_eq!(ext.to_lowercase(), "png");

        let data = image::load_from_memory_with_format(&content, ImageFormat::Png)?
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        let texture_state = None;

        Ok(Image {
            data,
            texture_state,
        })
    }
}
