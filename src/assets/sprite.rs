use std::borrow::Cow;

use assets_manager::loader::Loader;
use blit::ToBlitBuffer;
use image::ImageFormat;
use vek::Vec2;

use crate::sprite::Sprite;

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
