//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use glamour::Size2;

use crate::graphics::atlas::AtlasRef;

/// Core of a sprite loaded from disk.
#[derive(Clone)]
pub(crate) struct Image {
    /// Image atlas ID.
    pub(crate) atlas_id: AtlasRef,

    /// Size of the image in pixels.
    pub(crate) size: Size2<u32>,
}
