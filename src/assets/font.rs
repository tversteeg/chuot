use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;

/// Font metadata to load.
#[derive(Deserialize)]
pub(crate) struct FontMetadata {
    /// Width of a single character.
    pub(crate) char_width: u8,
    /// Height of a single character.
    pub(crate) char_height: u8,
}

impl Asset for FontMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}
