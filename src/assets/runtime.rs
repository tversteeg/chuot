//! Assets that will be loaded during runtime.

use std::{
    cell::RefCell,
    path::{PathBuf, MAIN_SEPARATOR},
    rc::Rc,
    str::FromStr,
};

use glamour::Size2;
use hashbrown::HashMap;

use crate::{
    assets::{image::Image, Id},
    graphics::{
        atlas::{Atlas, AtlasRef},
        gpu::Gpu,
    },
};

use super::loader::{
    png::{PngLoader, PngReader},
    Loader,
};

/// Compile time assets that haven't been parsed yet.
#[allow(clippy::exhaustive_structs)]
pub struct EmbeddedAssets(pub &'static str);

impl EmbeddedAssets {
    /// Get an empty static atlas.
    #[allow(clippy::unused_self)]
    pub(crate) const fn atlas(self) -> EmbeddedRawStaticAtlas {
        EmbeddedRawStaticAtlas
    }
}

/// Embedded diced sprite atlas in the binary.
#[non_exhaustive]
pub struct EmbeddedRawStaticAtlas;

impl EmbeddedRawStaticAtlas {
    /// Create an empty atlas.
    #[allow(clippy::unused_self)]
    pub(crate) fn parse_and_upload(self, gpu: &Gpu) -> Atlas {
        Atlas::new(Vec::new(), gpu)
    }
}

/// Asset source for all assets embedded in the binary.
pub struct AssetSource {
    /// Asset directory.
    assets_dir: PathBuf,
    /// Cache of images.
    image_cache: Rc<RefCell<ImageCache>>,
}

impl AssetSource {
    /// Construct a new source from a directory in the project folder.
    pub(crate) fn new(assets_dir: &str) -> Self {
        let assets_dir = PathBuf::from_str(assets_dir).unwrap();
        let image_cache = Rc::new(RefCell::new(ImageCache::new()));

        Self {
            assets_dir,
            image_cache,
        }
    }

    /// Load a new asset based on the loader.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`crate::assets::Loadable`] function to load the asset with.
    ///
    /// # Returns
    ///
    /// - An asset when it's found and has the correct type.
    /// - `None` if the asset could not be found.
    ///
    /// # Panics
    ///
    /// - When loading the asset fails.
    #[inline]
    #[must_use]
    pub fn load_if_exists<L, T>(&self, id: &Id) -> Option<T>
    where
        L: Loader<T>,
    {
        log::debug!(
            "Loading part of asset '{id}' with extension '{}'",
            L::EXTENSION
        );

        // Convert ID back to file
        let file_path = self.assets_dir.join(format!(
            "{}.{}",
            id.replace('.', std::str::from_utf8(&[MAIN_SEPARATOR as u8]).unwrap()),
            L::EXTENSION
        ));

        // Read the file, return None if it failed for whatever reason
        let bytes = std::fs::read(file_path).ok()?;

        // Create object
        Some(L::load(&bytes))
    }

    /// Load a new image based on the loader.
    ///
    /// This is a special case because images need to be uploaded to the GPU at a later stage.
    ///
    /// # Arguments
    ///
    /// * `id` - Image asset ID passed to the [`Loadable`] function to load the image with.
    ///
    /// # Returns
    ///
    /// - An image when it's found and has the correct type.
    /// - `None` if the image asset could not be found.
    pub(crate) fn get_or_load_image_if_exists(&self, id: &Id) -> Option<Image> {
        log::debug!("Loading image asset '{id}'",);

        // Get or load the image
        let atlas_id = self.image_cache.borrow_mut().get_or_load(id);

        Some(Image {
            atlas_id,
            // TODO: offset this somehow
            size: self.image_size(id)?,
        })
    }

    /// Get the size of an image based on a texture asset ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`Loadable`] function to get the size from.
    ///
    /// # Returns
    ///
    /// - A size when the asset is found and has the correct type.
    /// - `None` if the asset could not be found.
    pub(crate) fn image_size(&self, id: &Id) -> Option<Size2<u32>> {
        // Hacky way to get the texture by reading the whole PNG
        // TODO: find better way
        let png = self.load_if_exists::<PngLoader, _>(id)?;
        let info = png.info();

        Some(Size2::new(info.width, info.height))
    }

    /// Take and load all images for uploading.
    pub(crate) fn take_images_for_uploading(&mut self) -> Vec<(AtlasRef, PngReader)> {
        // Load the images
        let mut to_load = self
            .image_cache
            .borrow_mut()
            .take_to_load()
            .map(|(id, atlas_id)| {
                (
                    atlas_id,
                    self.load_if_exists::<PngLoader, _>(&id)
                        .expect("File suddenly disappeared"),
                )
            })
            .collect::<Vec<_>>();

        // Sort so all atlas references are inserted in the proper order
        to_load.sort_by_key(|(atlas_id, _img)| *atlas_id);

        to_load
    }

    /// Update based on the assets that have been changed.
    #[cfg(feature = "hot-reload-assets")]
    pub(crate) fn process_hot_reloaded_assets(&mut self) {
        // Remove each item from the image cache, but don't consume the assets because it will be used after this by the `AssetsManager`
        for changed_asset in crate::assets::hot_reload::global_assets_updated()
            .lock()
            .unwrap()
            .iter()
        {
            self.image_cache.borrow_mut().remove(changed_asset);
        }
    }
}

/// Image cache for allowing multiple code paths to upload and reference images.
pub(crate) struct ImageCache {
    /// Images to still load and upload to the GPU.
    to_load: HashMap<Id, AtlasRef>,
    /// Map of already uploaded images with their atlas ID.
    uploaded: HashMap<Id, AtlasRef>,
    /// Atlas index of images already uploaded.
    atlas_index: AtlasRef,
}

impl ImageCache {
    /// Create a new empty image cache.
    pub(crate) fn new() -> Self {
        let to_load = HashMap::new();
        let uploaded = HashMap::new();
        let atlas_index = 0;

        Self {
            to_load,
            uploaded,
            atlas_index,
        }
    }

    /// Get or load a new image if it doesn't exist.
    pub(crate) fn get_or_load(&mut self, id: &Id) -> AtlasRef {
        // TODO: check if path exists

        // First look if it's already uploaded
        if let Some(atlas_id) = self.atlas_id(id) {
            return atlas_id;
        }

        let atlas_index = self.atlas_index;
        // Take the next item in the atlas
        self.atlas_index += 1;

        // It's not uploaded, add it to the queue
        self.to_load.insert(id.clone(), atlas_index);

        // Return the new incremented reference
        atlas_index
    }

    /// Take all images that need to be uploaded.
    pub(crate) fn take_to_load(&mut self) -> impl Iterator<Item = (Id, AtlasRef)> + '_ {
        // Add to uploaded
        self.uploaded.extend(self.to_load.clone());

        // Remove from the old vector
        self.to_load.drain()
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
        self.to_load.get(id).copied()
    }

    /// Remove an asset from the cache.
    ///
    /// This can be used to trigger a reload.
    #[cfg(feature = "hot-reload-assets")]
    pub(crate) fn remove(&mut self, id: &Id) {
        self.uploaded.remove(id);
    }
}
