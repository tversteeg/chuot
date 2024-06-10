//! Commands to be executed on th GPU before the rendering pipeline.

use glamour::Size2;
use imgref::ImgVec;

use crate::assets::{image::DicedImageMapping, Id};

/// Different commands for to be executed on the GPU.
pub(crate) enum GpuCommand {
    /// Create a new empty image.
    CreateImage {
        /// Image ID.
        id: Id,
        /// Size of the new image.
        size: Size2<u16>,
    },
    /// Remove the image.
    RemoveImage {
        /// Image ID.
        id: Id,
    },
    /// Resize an existing image.
    ResizeImage {
        /// Image ID.
        id: Id,
        /// Size of the image to resize to.
        new_size: Size2<u16>,
    },
    /// Update a portion of the image.
    UpdateImage {
        /// Image ID.
        id: Id,
        /// Source of the image to update.
        source: ImgVec<u32>,
        /// Mappings of each part to update in relative coordinates.
        mappings: Vec<DicedImageMapping>,
    },
}
