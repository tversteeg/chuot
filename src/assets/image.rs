//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use glamour::Size2;

use crate::graphics::atlas::AtlasRef;

use super::{AssetSource, Id, Loadable};

/// Core of a sprite loaded from disk.
#[derive(Clone)]
pub(crate) struct Image {
    /// Image atlas ID.
    pub(crate) atlas_id: AtlasRef,

    /// Size of the image in pixels.
    pub(crate) size: Size2<u32>,
}

impl Loadable for Image {
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        let atlas_id = assets.atlas_id(id)?;
        let size = assets.texture_size(id)?;

        Some(Self { atlas_id, size })
    }
}
