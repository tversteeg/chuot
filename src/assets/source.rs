//! Where assets are retrieved from.

use std::{
    path::{PathBuf, MAIN_SEPARATOR},
    str::FromStr,
};

use glamour::{Point2, Rect, Size2};

use super::{loader::Loader, Id};

/// Empty array when embedding nothing.
static RUNTIME_EMBEDDED_ASSETS: &[EmbeddedRawAsset] = &[];

/// Source of all assets.
#[allow(clippy::exhaustive_structs)]
pub struct AssetSource {
    /// Path to the directory of all assets.
    ///
    /// Not set when using the `embed-assets` feature flag, because all assets will be embedded into the binary.
    runtime_asset_dir: Option<PathBuf>,
    /// Assets directly embedded into the binary.
    embedded_assets: &'static [EmbeddedRawAsset],
    /// Diced raw texture atlas.
    embedded_atlas: EmbeddedRawStaticAtlas,
}

impl AssetSource {
    /// Create the assets.
    ///
    /// Should only be called from the macros crate.
    #[inline(always)]
    pub fn new() -> Self {
        let runtime_asset_dir = None;
        let embedded_assets = RUNTIME_EMBEDDED_ASSETS;
        let embedded_atlas = EmbeddedRawStaticAtlas::default();

        Self {
            runtime_asset_dir,
            embedded_assets,
            embedded_atlas,
        }
    }

    /// Set a runtime asset directory where assets can be loaded from.
    #[inline(always)]
    pub fn with_runtime_dir(mut self, runtime_asset_dir: &str) -> Self {
        self.runtime_asset_dir = Some(PathBuf::from_str(runtime_asset_dir).unwrap());

        self
    }

    /// Embed raw assets into the source.
    #[inline(always)]
    pub fn with_embedded_assets(mut self, embedded_assets: &'static [EmbeddedRawAsset]) -> Self {
        self.embedded_assets = embedded_assets;

        self
    }

    /// Embed a raw texture atlas into the source.
    #[inline(always)]
    pub fn with_embedded_atlas(mut self, embedded_astlas: EmbeddedRawStaticAtlas) -> Self {
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
            if raw_asset.id == id && raw_asset.extension == L::EXTENSION {
                Some(raw_asset.bytes)
            } else {
                None
            }
        }) {
            // Create object
            return Some(L::load(&bytes));
        }

        // If not found load from disk if dir set
        if let Some(runtime_asset_dir) = &self.runtime_asset_dir {
            log::debug!(
                "Loading part of asset '{id}' with extension '{}'",
                L::EXTENSION
            );

            // Convert ID back to file
            let file_path = runtime_asset_dir.join(format!(
                "{}.{}",
                id.replace('.', std::str::from_utf8(&[MAIN_SEPARATOR as u8]).unwrap()),
                L::EXTENSION
            ));

            // Read the file, return None if it failed for whatever reason
            let bytes = std::fs::read(file_path).ok()?;

            // Create object
            Some(L::load(&bytes))
        } else {
            None
        }
    }
}

/// Single embedded asset in the binary.
pub struct EmbeddedRawAsset {
    /// Parsed ID, excludes the file extension.
    pub id: &'static str,
    /// File extension excluding the first `'.'`.
    pub extension: &'static str,
    /// Raw bytes of the asset.
    pub bytes: &'static [u8],
}

/// Embedded diced sprite atlas in the binary.
#[derive(Debug)]
pub struct TextureMapping {
    /// U & V coordinates for source on the diced texture.
    pub diced: Point2<u16>,
    /// U & V coordinates for target on the source texture.
    pub texture: Point2<u16>,
    /// Size of the diced sub image.
    pub size: Size2<u16>,
    /// Texture index in the atlas.
    #[cfg(feature = "read-image")]
    pub index: usize,
    /// U & V coordinates for reconstructing the source texture.
    #[cfg(feature = "read-image")]
    pub source: Point2<u16>,
}

/// Embedded diced sprite atlas in the binary.
#[derive(Default)]
pub struct EmbeddedRawStaticAtlas {
    /// PNG bytes of the diced atlas.
    pub diced_atlas_png_bytes: &'static [u8],
    /// Rectangle mapping for the textures.
    ///
    /// Structure is `[texture_index, diced_u, diced_v, texture_u, texture_v, width, height]`.
    pub texture_mappings: &'static [TextureMapping],
    /// All IDS of the textures.
    ///
    /// Index determines the position.
    pub texture_ids: &'static [&'static str],
    /// Full items on the atlas.
    ///
    /// Index determines the position.
    pub texture_rects: &'static [Rect],
    /// Fitted width of the atlas.
    pub width: u16,
    /// Fitted height of the atlas.
    pub height: u16,
}
