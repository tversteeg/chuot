use std::{borrow::Cow, f64::consts::TAU, num::NonZeroU16};

use assets_manager::{loader::Loader, AnyCache, Asset, BoxedError, Compound, SharedString};
use blit::{BlitBuffer, ToBlitBuffer};
use image::ImageFormat;
use vek::Vec2;

use crate::sprite::Sprite;

impl Asset for Sprite {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = SpriteLoader;
}

impl Default for Sprite {
    fn default() -> Self {
        let sprite = BlitBuffer::from_buffer(&[0], 1, 0);
        let offset = Vec2::zero();

        Self { sprite, offset }
    }
}

/// Sprite asset loader.
pub struct SpriteLoader;

impl Loader<Sprite> for SpriteLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Sprite, assets_manager::BoxedError> {
        let sprite = image::load_from_memory_with_format(&content, ImageFormat::Png)?
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        let offset = Vec2::zero();

        Ok(Sprite { sprite, offset })
    }
}
