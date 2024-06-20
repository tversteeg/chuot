//! Sprite asset.

use crate::assets::{loader::png::PngLoader, source::AssetSource, Id};

use super::Loadable;

/// Sprite asset that can be loaded with metadata.
pub(crate) struct Sprite {
    /// Reference to the texture on the GPU.
    texture_id: i32,
}

impl Loadable for Sprite {
    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        // Load the PNG
        let png = asset_source.load_if_exists::<PngLoader, _>(id)?;

        // Upload it to the GPU, returning a reference
        let texture_id = asset_source.upload_texture(png);

        Some(Self { texture_id })
    }
}
