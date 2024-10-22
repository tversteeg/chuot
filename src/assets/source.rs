//! How and where assets are loaded.

use std::{
    path::{MAIN_SEPARATOR, PathBuf},
    str::FromStr as _,
};

use phf::Map;

use super::{Id, loader::Loader};
use crate::graphics::atlas::TextureRef;

/// Empty array when embedding nothing.
static RUNTIME_EMBEDDED_ASSETS: &[EmbeddedRawAsset] = &[];
/// Empty atlas when embedding nothing.
static RUNTIME_EMBEDDED_ATLAS: &EmbeddedRawStaticAtlas = &EmbeddedRawStaticAtlas {
    diced_atlas_png_bytes: &[],
    width: 0,
    height: 0,
    textures: &phf::Map::new(),
};

/// Source of all assets.
///
/// It's recommended to instantiate this with the [`crate::load_assets`] macro.
/// That way an embedded asset pipeline can be directly used with the `embed-assets` feature flag.
pub struct AssetSource {
    /// Path to the directory of all assets.
    ///
    /// Not set when using the `embed-assets` feature flag, because all assets will be embedded into the binary.
    runtime_asset_dir: Option<PathBuf>,
    /// Assets directly embedded into the binary.
    embedded_assets: &'static [EmbeddedRawAsset],
    /// Diced raw texture atlas.
    embedded_atlas: &'static EmbeddedRawStaticAtlas,
    /// State for watching the folder for hot reloading functionality.
    #[cfg(not(target_arch = "wasm32"))]
    hot_reload_folder_watcher:
        Option<notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>>,
}

impl AssetSource {
    /// Create the assets.
    ///
    /// Should only be called from the macros crate.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        let runtime_asset_dir = None;
        let embedded_assets = RUNTIME_EMBEDDED_ASSETS;
        let embedded_atlas = RUNTIME_EMBEDDED_ATLAS;

        Self {
            runtime_asset_dir,
            embedded_assets,
            embedded_atlas,
            #[cfg(not(target_arch = "wasm32"))]
            hot_reload_folder_watcher: None,
        }
    }

    /// Set a runtime asset directory where assets can be loaded from, and enable hot-reloading for that directory.
    #[inline(always)]
    #[must_use]
    pub fn with_runtime_dir(mut self, runtime_asset_dir: &str) -> Self {
        self.runtime_asset_dir = Some(PathBuf::from_str(runtime_asset_dir).unwrap());

        // Enable hot reloading, if not on the web
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.hot_reload_folder_watcher =
                Some(super::hot_reload::watch_assets_folder(runtime_asset_dir));
        }

        self
    }

    /// Embed raw assets into the source.
    #[inline(always)]
    #[must_use]
    pub const fn with_embedded_assets(
        mut self,
        embedded_assets: &'static [EmbeddedRawAsset],
    ) -> Self {
        self.embedded_assets = embedded_assets;

        self
    }

    /// Embed a raw texture atlas into the source.
    #[inline(always)]
    #[must_use]
    pub const fn with_embedded_atlas(
        mut self,
        embedded_astlas: &'static EmbeddedRawStaticAtlas,
    ) -> Self {
        self.embedded_atlas = embedded_astlas;

        self
    }

    /// Load a new asset based on the loader.
    #[inline]
    #[must_use]
    pub fn load_if_exists<L, T>(&self, id: &Id) -> Option<T>
    where
        L: Loader<T>,
    {
        // First try to read from memory
        // TODO: use a map for this
        if let Some(bytes) = self.embedded_assets.iter().find_map(|raw_asset| {
            (raw_asset.id == id && raw_asset.extension == L::EXTENSION).then_some(raw_asset.bytes)
        }) {
            // Create object
            return Some(L::load(bytes, id));
        }

        // If not found load from disk if dir set
        if let Some(runtime_asset_dir) = &self.runtime_asset_dir {
            // Convert ID back to file
            let file_path = runtime_asset_dir.join(format!(
                "{}.{}",
                id.replace('.', std::str::from_utf8(&[MAIN_SEPARATOR as u8]).unwrap()),
                L::EXTENSION
            ));

            // Read the file, return None if it failed for whatever reason
            let bytes = std::fs::read(file_path).ok()?;

            // Create object
            Some(L::load(&bytes, id))
        } else {
            None
        }
    }

    /// Get the texture for an embedded texture if it exists.
    #[must_use]
    #[inline]
    pub(crate) fn embedded_texture(&self, id: &Id) -> Option<&EmbeddedTexture> {
        self.embedded_atlas.textures.get(id)
    }

    /// Get the embedded atlas texture.
    #[must_use]
    #[inline]
    pub(crate) const fn embedded_atlas(&self) -> &EmbeddedRawStaticAtlas {
        self.embedded_atlas
    }
}

impl Default for AssetSource {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Single embedded asset in the binary.
#[allow(clippy::exhaustive_structs)]
pub struct EmbeddedRawAsset {
    /// Parsed ID, excludes the file extension.
    pub id: &'static str,
    /// File extension excluding the first `'.'`.
    pub extension: &'static str,
    /// Raw bytes of the asset.
    pub bytes: &'static [u8],
}

/// Embedded texture.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct EmbeddedTexture {
    /// Width of the texture if re-constructed.
    pub width: u16,
    /// Height of the texture if re-constructed.
    pub height: u16,
    /// Reference of the texture.
    pub reference: TextureRef,
    /// Diced mappings to the atlas.
    pub diced: &'static [EmbeddedTextureDiceMapping],
}

/// Embedded diced sprite atlas in the binary.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct EmbeddedTextureDiceMapping {
    /// U coordinate for source on the diced texture.
    pub diced_u: u16,
    /// V coordinate for source on the diced texture.
    pub diced_v: u16,
    /// U coordinate for target on the source texture.
    pub texture_u: u16,
    /// V coordinate for target on the source texture.
    pub texture_v: u16,
    /// Width of the diced sub image.
    pub width: u16,
    /// Height of the diced sub image.
    pub height: u16,
}

/// Embedded diced sprite atlas in the binary.
#[allow(clippy::exhaustive_structs)]
pub struct EmbeddedRawStaticAtlas {
    /// PNG bytes of the diced atlas.
    pub diced_atlas_png_bytes: &'static [u8],
    /// Fitted width of the atlas.
    pub width: u16,
    /// Fitted height of the atlas.
    pub height: u16,
    /// Embedded static textures in the atlas.
    pub textures: &'static Map<&'static str, EmbeddedTexture>,
}
