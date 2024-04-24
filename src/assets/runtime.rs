//! Assets that will be loaded during runtime.

use std::{path::PathBuf, str::FromStr};

use glamour::Size2;
use hashbrown::HashMap;

use crate::{
    assets::Id,
    graphics::{atlas::Atlas, gpu::Gpu},
};

use super::loader::{png::PngLoader, Loader};

/// Compile time assets that haven't been parsed yet.
pub struct EmbeddedAssets(pub &'static str);

impl EmbeddedAssets {
    /// Get an empty static atlas.
    pub(crate) fn atlas(self) -> EmbeddedRawStaticAtlas {
        EmbeddedRawStaticAtlas
    }
}

/// Embedded diced sprite atlas in the binary.
pub struct EmbeddedRawStaticAtlas;

impl EmbeddedRawStaticAtlas {
    /// Create an empty atlas.
    pub(crate) fn parse_and_upload(self, gpu: &Gpu) -> Atlas {
        Atlas::new(Vec::new(), gpu)
    }

    /// Create a texture mapping to atlas index table.
    pub(crate) fn texture_id_to_atlas_id_map(&self) -> HashMap<Id, u16> {
        HashMap::new()
    }

    /// Create a texture mapping to size.
    pub(crate) fn texture_id_to_size_map(&self) -> HashMap<Id, Size2<u32>> {
        HashMap::new()
    }
}

/// Asset source for all assets embedded in the binary.
pub struct AssetSource {
    /// Asset directory.
    assets_dir: PathBuf,
}

impl AssetSource {
    /// Construct a new source from a directory in the project folder.
    pub(crate) fn new(assets_dir: &str) -> Self {
        let assets_dir = PathBuf::from_str(assets_dir).unwrap();

        Self { assets_dir }
    }

    /// Load a new asset based on the loader.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`Loadable`] function to load the asset with.
    ///
    /// # Returns
    ///
    /// - An asset when it's found and has the correct type.
    /// - `None` if the asset could not be found.
    ///
    /// # Panics
    ///
    /// - When loading the asset fails.
    #[track_caller]
    pub fn load_if_exists<L, T>(&self, id: &Id) -> Option<T>
    where
        L: Loader<T>,
    {
        log::debug!(
            "Loading part of asset '{id}' with extension '{}'",
            L::EXTENSION
        );

        Some(L::load(self.raw_asset(id, L::EXTENSION)?))
    }

    /// Get the atlas ID based on a texture asset ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`Loadable`] function to get the atlas ID from.
    ///
    /// # Returns
    ///
    /// - An atlas ID when the asset is found and has the correct type.
    /// - `None` if the asset could not be found.
    #[track_caller]
    pub fn atlas_id(&self, id: &Id) -> Option<u16> {
        // TODO
        Some(0)
    }

    /// Get the size of a texture based on a texture asset ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Asset ID passed to the [`Loadable`] function to get the size from.
    ///
    /// # Returns
    ///
    /// - A size when the asset is found and has the correct type.
    /// - `None` if the asset could not be found.
    #[track_caller]
    pub fn texture_size(&self, id: &Id) -> Option<Size2<u32>> {
        // Hacky way to get the texture by reading the whole PNG
        // TODO: find better way
        let png = self.load_if_exists::<PngLoader, _>(id)?;
        let info = png.info();

        Some(Size2::new(info.width, info.height))
    }

    /// Get the bytes of an asset that matches the ID and the extension.
    pub(crate) fn raw_asset(&self, id: &Id, extension: &str) -> Option<&[u8]> {
        // Convert ID back to file
        let file_path = self
            .assets_dir
            .join(format!("{}.{extension}", id.replace(".", "/")));

        // Read the file, return None if it failed for whatever reason
        let bytes = std::fs::read(file_path).ok()?;

        // TODO: fix
        let bytes_ref = Box::leak(Box::new(bytes));

        Some(bytes_ref)
    }
}
