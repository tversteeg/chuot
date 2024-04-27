//! Custom asset loading.
//!
//! Assets will be embedded into the binary when the `hot-reloading-assets` feature flag is disabled or a WASM build is made, since there's no filesystem there.
//!
//! Asset loading is done through the various calls in [`crate::Context`].

mod audio;
#[cfg(feature = "embed-assets")]
#[doc(hidden)]
pub mod embedded;
pub(crate) mod image;
pub mod loader;
#[cfg(not(feature = "embed-assets"))]
#[doc(hidden)]
pub mod runtime;

pub use audio::Audio;

use std::rc::Rc;

use downcast_rs::Downcast;
use hashbrown::HashMap;
use smol_str::SmolStr;

#[cfg(feature = "embed-assets")]
pub use embedded::AssetSource;
#[cfg(feature = "embed-assets")]
pub(crate) use embedded::{EmbeddedAssets, EmbeddedRawStaticAtlas};
#[cfg(not(feature = "embed-assets"))]
pub use runtime::AssetSource;
#[cfg(not(feature = "embed-assets"))]
pub(crate) use runtime::{EmbeddedAssets, EmbeddedRawStaticAtlas};

use crate::{font::Font, sprite::Sprite};

/// Asset ID.
///
/// When the string is smaller than 23 bytes this will be stored on the stack.
pub type Id = SmolStr;

/// Any asset that's loadable from any amount of binary files.
pub trait Loadable: Downcast {
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
downcast_rs::impl_downcast!(Loadable);

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
    pub(crate) fn get_or_insert<'id>(&mut self, id: &'id str, asset_source: &AssetSource) -> Rc<T> {
        if let Some(asset) = self.get(id) {
            asset
        } else {
            self.insert(id, asset_source)
        }
    }

    /// Return an asset if it exists.
    #[inline]
    #[track_caller]
    pub(crate) fn get<'id>(&self, id: &'id str) -> Option<Rc<T>> {
        self.assets.get(id).cloned()
    }

    /// Upload a new asset.
    #[inline]
    #[track_caller]
    pub(crate) fn insert<'id>(&mut self, id: &'id str, asset_source: &AssetSource) -> Rc<T> {
        log::debug!("Asset '{id}' not loaded yet, loading from source");

        // Create owned ID
        let id = Id::new(id);

        // Load the asset
        let asset = T::load(&id, asset_source);
        let asset = Rc::new(asset);

        // Store the asset so it can be accessed later again
        self.assets.insert(id, asset.clone());

        asset
    }
}

impl<T: Loadable> Default for AssetManager<T> {
    fn default() -> Self {
        let assets = HashMap::new();

        Self { assets }
    }
}

/// Global asset manager for a any type not known at compile time.
///
/// When hot-reloading is enabled all assets are loaded from disk, otherwise all assets are embedded in the binary.
pub(crate) struct CustomAssetManager {
    /// All loaded assets.
    assets: HashMap<Id, Rc<dyn Loadable + 'static>>,
}

impl CustomAssetManager {
    /// Get an asset or throw an exception.
    #[inline]
    #[track_caller]
    pub(crate) fn get_or_insert<'id, T>(
        &mut self,
        id: &'id str,
        asset_source: &AssetSource,
    ) -> Rc<T>
    where
        T: Loadable,
    {
        if let Some(asset) = self.get(id) {
            asset
        } else {
            self.insert(id, asset_source)
        }
    }

    /// Return an asset if it exists.
    #[inline]
    #[track_caller]
    pub(crate) fn get<'id, T>(&self, id: &'id str) -> Option<Rc<T>>
    where
        T: Loadable,
    {
        // Try to find the asset
        let Some(dyn_asset) = self.assets.get(id) else {
            return None;
        };

        // Try to downcast it to the requested type
        match dyn_asset.clone().downcast_rc::<T>() {
            Ok(asset) => Some(asset),
            Err(_) => {
                panic!("Could downcast asset, loaded type is different from requested type")
            }
        }
    }

    /// Upload a new asset.
    #[inline]
    #[track_caller]
    pub(crate) fn insert<'id, T>(&mut self, id: &'id str, asset_source: &AssetSource) -> Rc<T>
    where
        T: Loadable,
    {
        log::debug!("Asset '{id}' not loaded yet, loading from source");

        // Create owned ID
        let id = Id::new(id);

        // Load the asset
        let asset: Rc<dyn Loadable> = Rc::new(T::load(&id, asset_source));

        // Store the asset so it can be accessed later again
        self.assets.insert(id, asset.clone());

        // Safe to unwrap because we created the type here
        match asset.downcast_rc::<T>() {
            Ok(asset) => asset,
            Err(_) => panic!("Error downcasting type"),
        }
    }
}

impl Default for CustomAssetManager {
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
    /// Custom type erased assets.
    custom: CustomAssetManager,
    /// Source for all un-loaded assets.
    source: AssetSource,
}

impl AssetsManager {
    /// Create from an asset source.
    pub(crate) fn new(source: AssetSource) -> Self {
        let sprites = AssetManager::default();
        let fonts = AssetManager::default();
        let audio = AssetManager::default();
        let custom = CustomAssetManager::default();

        Self {
            sprites,
            fonts,
            audio,
            custom,
            source,
        }
    }

    /// Get or load a sprite.
    ///
    /// # Panics
    ///
    /// - When sprite asset could not be loaded.
    #[inline]
    pub(crate) fn sprite<'id>(&mut self, id: &'id str) -> Rc<Sprite> {
        self.sprites.get_or_insert(id, &self.source)
    }

    /// Get or load a font.
    ///
    /// # Panics
    ///
    /// - When font asset could not be loaded.
    #[inline]
    pub(crate) fn font<'id>(&mut self, id: &'id str) -> Rc<Font> {
        self.fonts.get_or_insert(id, &self.source)
    }

    /// Get or load an audio file.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    #[inline]
    pub(crate) fn audio<'id>(&mut self, id: &'id str) -> Rc<Audio> {
        self.audio.get_or_insert(id, &self.source)
    }

    /// Get or load a custom asset.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    /// - When type used to load the asset mismatches the type used to get it.
    #[inline]
    pub(crate) fn custom<'id, T>(&mut self, id: &'id str) -> Rc<T>
    where
        T: Loadable,
    {
        self.custom.get_or_insert(id, &self.source)
    }

    /// Get a clone or load a custom asset.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    /// - When type used to load the asset mismatches the type used to get it.
    #[inline]
    pub(crate) fn custom_owned<'id, T>(&mut self, id: &'id str) -> T
    where
        T: Loadable + Clone,
    {
        // Create a clone of the asset
        Rc::<T>::unwrap_or_clone(self.custom.get_or_insert::<T>(id, &self.source))
    }
}
