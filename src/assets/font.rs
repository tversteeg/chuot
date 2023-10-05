use assets_manager::{loader::TomlLoader, AnyCache, Asset, BoxedError, Compound, SharedString};
use serde::Deserialize;
use vek::Extent2;

use crate::{font::Font, sprite::Sprite};

impl Compound for Font {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache.load_owned::<Sprite>(id)?.into_blit_buffer();

        // Load the metadata
        let metadata = cache.load::<FontMetadata>(id)?.read();
        let char_size = Extent2::new(metadata.char_width, metadata.char_height);

        Ok(Self { sprite, char_size })
    }
}

/// Font metadata to load.
#[derive(Deserialize)]
struct FontMetadata {
    /// Width of a single character.
    char_width: u8,
    /// Height of a single character.
    char_height: u8,
}

impl Asset for FontMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}
