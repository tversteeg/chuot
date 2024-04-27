//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use glamour::Size2;
use hashbrown::HashMap;

use crate::graphics::atlas::AtlasRef;

use super::Id;

/// Core of a sprite loaded from disk.
#[derive(Clone)]
pub(crate) struct Image {
    /// Image atlas ID.
    pub(crate) atlas_id: AtlasRef,

    /// Size of the image in pixels.
    pub(crate) size: Size2<u32>,
}

/// Image cache for allowing multiple code paths to upload and reference images.
pub(crate) struct ImageCache {
    /// Images to still load and upload to the GPU.
    to_load: Vec<Id>,
    /// Map of already uploaded images with their atlas ID.
    uploaded: HashMap<Id, AtlasRef>,
}

impl ImageCache {
    /// Create a new empty image cache.
    pub(crate) fn new() -> Self {
        let to_load = Vec::new();
        let uploaded = HashMap::new();

        Self { to_load, uploaded }
    }

    /// Get or load a new image if it doesn't exist.
    pub(crate) fn get_or_load(&mut self, id: &Id) -> Option<AtlasRef> {
        // First look if it's already uploaded
        if let Some(atlas_id) = self.atlas_id(id) {
            return Some(atlas_id);
        }

        // It's not uploaded, add it to the queue
        self.to_load.push(id.clone());

        // Return the new incremented reference
        Some((self.to_load.len() + self.uploaded.len() - 1) as AtlasRef)
    }

    /// Take all images that need to be uploaded.
    pub(crate) fn take_to_load(&mut self) -> impl Iterator<Item = (Id, AtlasRef)> + '_ {
        // Add to uploaded
        let already_uploaded_len = self.uploaded.len();
        self.uploaded
            .extend(self.to_load.iter().enumerate().map(|(relative_index, id)| {
                (
                    id.clone(),
                    // Add the index of the new item to the length of the already uploaded images so we get the future ID
                    (relative_index + already_uploaded_len) as AtlasRef,
                )
            }));

        // Remove from the old vector
        self.to_load
            .drain(..)
            .enumerate()
            // Add the index of the new item to the length of the already uploaded images so we get the future ID
            .map(move |(relative_index, id)| {
                (id, (relative_index + already_uploaded_len) as AtlasRef)
            })
    }

    /// Request the atlas ID for an image.
    ///
    /// Will first look in already uploaded, and if not found loop over the new images to upload.
    pub(crate) fn atlas_id(&self, id: &Id) -> Option<AtlasRef> {
        // First look if it's already uploaded
        if let Some(atlas_id) = self.uploaded.get(id) {
            return Some(*atlas_id);
        }

        // Then try to find the item in the new images to upload
        self.to_load
            .iter()
            .enumerate()
            .find_map(|(index, to_upload_id)| {
                if to_upload_id == id {
                    // Add the index of the new item to the length of the already uploaded images so we get the future ID
                    Some((index + self.uploaded.len()) as AtlasRef)
                } else {
                    None
                }
            })
    }
}
