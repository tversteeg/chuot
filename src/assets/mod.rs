//! Custom asset loading.
//!
//! Assets will be embedded into the binary when the `hot-reloading-assets` feature flag is disabled or a WASM build is made, since there's no filesystem there.
//!
//! Asset loading is done through the various calls in [`crate::Context`].

mod audio;
#[doc(hidden)]
pub mod embedded;
pub(crate) mod image;
pub mod loader;

use std::rc::Rc;

pub use audio::Audio;
use embedded::EmbeddedRawAsset;
use glamour::Size2;
use hashbrown::HashMap;
use loader::Loader;
use smol_str::SmolStr;

use crate::{font::Font, sprite::Sprite};

/// Asset ID.
///
/// When the string is smaller than 23 bytes this will be stored on the stack.
pub type Id = SmolStr;

/// Any asset that's loadable from any amount of binary files.
pub trait Loadable {
    /// Convert a file object to this type if it exists, if it doesn't return `None`.
    ///
    /// # Panics
    ///
    /// - When parsing binary bytes of asset into type fails.
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized;

    /// Convert a file object to this type.
    ///
    /// - When parsing binary bytes of asset into type fails.
    /// - When asset does not exist in the source.
    fn load(id: &Id, asset_source: &AssetSource) -> Self
    where
        Self: Sized,
    {
        match Self::load_if_exists(id, asset_source) {
            Some(asset) => asset,
            None => panic!("Error loading asset: '{id}' does not exist"),
        }
    }
}

/// Global asset manager for a single type known at compile time.
///
/// When hot-reloading is enabled all assets are loaded from disk, otherwise all assets are embedded in the binary.
///
/// Improves performance because the types don't need to be boxed inside the vector.
pub(crate) struct AssetManager<T: Loadable> {
    /// All loaded assets.
    assets: HashMap<Id, Rc<T>>,
}

impl<T: Loadable> AssetManager<T> {
    /// Get an asset or throw an exception.
    #[inline]
    #[track_caller]
    pub(crate) fn get_or_insert(&mut self, id: &Id, asset_source: &AssetSource) -> Rc<T> {
        if let Some(asset) = self.get(id) {
            asset
        } else {
            self.insert(id, asset_source)
        }
    }

    /// Return an asset if it exists.
    #[inline]
    #[track_caller]
    pub(crate) fn get(&self, id: &Id) -> Option<Rc<T>> {
        self.assets.get(id).cloned()
    }

    /// Upload a new asset.
    #[inline]
    #[track_caller]
    pub(crate) fn insert(&mut self, id: &Id, asset_source: &AssetSource) -> Rc<T> {
        log::debug!("Asset '{id}' not loaded yet, loading from source");

        // Load the asset
        let asset = T::load(id, asset_source);
        let asset = Rc::new(asset);

        // Store the asset so it can be accessed later again
        self.assets.insert(id.to_owned(), asset.clone());

        asset
    }
}

impl<T: Loadable> Default for AssetManager<T> {
    fn default() -> Self {
        let assets = HashMap::new();

        Self { assets }
    }
}

/// Assets for all types used in- and outside the engine.
pub(crate) struct AssetsManager {
    /// Sprite assets.
    sprites: AssetManager<Sprite>,
    /// Font assets.
    fonts: AssetManager<Font>,
    /// Audio assets.
    audio: AssetManager<Audio>,
    /// Source for all un-loaded assets.
    source: AssetSource,
}

impl AssetsManager {
    /// Create from an asset source.
    pub(crate) fn new(source: AssetSource) -> Self {
        let sprites = AssetManager::default();
        let fonts = AssetManager::default();
        let audio = AssetManager::default();

        Self {
            sprites,
            fonts,
            audio,
            source,
        }
    }

    /// Get or load a sprite.
    ///
    /// # Panics
    ///
    /// - When sprite asset could not be loaded.
    #[inline]
    pub(crate) fn sprite(&mut self, id: impl Into<Id>) -> Rc<Sprite> {
        self.sprites.get_or_insert(&id.into(), &self.source)
    }

    /// Get or load a font.
    ///
    /// # Panics
    ///
    /// - When font asset could not be loaded.
    #[inline]
    pub(crate) fn font(&mut self, id: impl Into<Id>) -> Rc<Font> {
        self.fonts.get_or_insert(&id.into(), &self.source)
    }

    /// Get or load an audio file.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    #[inline]
    pub(crate) fn audio(&mut self, id: impl Into<Id>) -> Rc<Audio> {
        self.audio.get_or_insert(&id.into(), &self.source)
    }

    /// Get the static atlas ID for a texture asset.
    ///
    /// # Panics
    ///
    /// - When texture asset does not exist.
    #[inline]
    pub(crate) fn static_atlas_id(&mut self, id: impl Into<Id>) -> u16 {
        *self
            .source
            .static_texture_id_to_atlas_id
            .get(&id.into())
            .expect("Error loading static atlas ID: texture does not exist")
    }
}

/// Asset source for all assets embedded in the binary.
pub(crate) struct AssetSource {
    /// Static texture ID to atlas ID mapping.
    pub(crate) static_texture_id_to_atlas_id: HashMap<Id, u16>,
    /// Static texture ID to size.
    pub(crate) static_texture_id_to_size: HashMap<Id, Size2<u32>>,
    /// All loaded assets by parsed path.
    assets: &'static [EmbeddedRawAsset],
}

impl AssetSource {
    /// Construct a new source from embedded raw assets.
    pub(crate) fn new(
        assets: &'static [EmbeddedRawAsset],
        static_texture_id_to_atlas_id: HashMap<Id, u16>,
        static_texture_id_to_size: HashMap<Id, Size2<u32>>,
    ) -> Self {
        Self {
            assets,
            static_texture_id_to_atlas_id,
            static_texture_id_to_size,
        }
    }

    /// Load a new asset based on the loader.
    #[track_caller]
    pub(crate) fn load_if_exists<L, T>(&self, id: &Id) -> Option<T>
    where
        L: Loader<T>,
    {
        log::debug!(
            "Loading part of asset '{id}' with extension '{}'",
            L::EXTENSION
        );

        Some(L::load(self.raw_asset(id, L::EXTENSION)?))
    }

    /// Get the bytes of an asset that matches the ID and the extension.
    pub(crate) fn raw_asset(&self, id: &Id, extension: &str) -> Option<&'static [u8]> {
        self.assets.iter().find_map(|raw_asset| {
            if raw_asset.id == id && raw_asset.extension == extension {
                Some(raw_asset.bytes)
            } else {
                None
            }
        })
    }
}
