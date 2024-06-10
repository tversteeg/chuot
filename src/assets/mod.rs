//! Custom asset loading.
//!
//! Assets will be embedded into the binary when the `embed-assets` feature flag is enabled.
//! When building for WASM this feature flag is a requirement.
//!
//! Asset loading is done through the various calls in [`crate::Context`].

mod audio;
pub(crate) mod image;
pub mod loader;
#[doc(hidden)]
pub mod source;

use std::rc::Rc;

use downcast_rs::Downcast;
use glamour::Size2;
use hashbrown::HashMap;
use smol_str::SmolStr;

#[doc(hidden)]
pub use source::AssetSource;

use crate::{
    font::Font,
    graphics::{atlas::AtlasRef, command::GpuCommand},
    sprite::Sprite,
};

use self::{audio::Audio, image::ImageManager, loader::png::PngReader};

/// Identifier for any loadable asset, can be assigned multiple times for different types.
///
/// When the string is smaller than 23 bytes this will be stored on the stack.
pub type Id = SmolStr;

/// Any asset that's loadable from any amount of binary files.
///
/// # Example
///
/// ```
/// use chuot::assets::{loader::toml::TomlLoader, AssetSource, Id, Loadable};
/// use serde::Deserialize;
///
/// /// We define a custom settings object that will be loaded from a '.toml' file.
/// #[derive(Deserialize)]
/// struct Settings {
///     property_a: String,
///     property_b: i32,
/// }
///
/// impl Loadable for Settings {
///     fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
///     where
///         Self: Sized,
///     {
///         // Use the TOML loader to load our asset
///         assets.load_if_exists::<TomlLoader, _>(id)
///     }
/// }
/// ```
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
    /// # Panics
    ///
    /// - When parsing binary bytes of asset into type fails.
    /// - When asset does not exist in the source.
    #[inline]
    #[must_use]
    fn load(id: &Id, asset_source: &AssetSource) -> Self
    where
        Self: Sized,
    {
        match Self::load_if_exists(id, asset_source) {
            Some(asset) => asset,
            None => panic!("Error loading asset: '{id}' does not exist"),
        }
    }

    /// Create a new runtime asset from the default value.
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
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
    /// Get an asset or load it from the asset source.
    #[inline]
    #[track_caller]
    pub(crate) fn get_or_insert(&mut self, id: &str, asset_source: &AssetSource) -> Rc<T> {
        // Return a reference to the asset when it already exists, otherwise insert it
        self.get(id)
            // Asset not found, load it
            .map_or_else(|| self.load(id, asset_source), |asset| asset)
    }

    /// Return an asset if it exists.
    #[inline]
    #[track_caller]
    pub(crate) fn get(&self, id: &str) -> Option<Rc<T>> {
        self.assets.get(id).cloned()
    }

    /// Upload a asset from an asset source.
    #[inline]
    #[track_caller]
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn load(&mut self, id: &str, asset_source: &AssetSource) -> Rc<T> {
        log::debug!("Asset '{id}' not loaded yet, loading from source");

        // Create the ID
        let id = Id::new(id);

        // Load the asset
        let asset = T::load(&id, asset_source);

        // Upload it
        self.insert(id, asset)
    }

    /// Insert the loaded asset so it can be accessed.
    #[inline]
    #[track_caller]
    pub(crate) fn insert(&mut self, id: Id, asset: T) -> Rc<T> {
        // Wrap the asset
        let asset = Rc::new(asset);

        // Store the asset so it can be accessed later again
        self.assets.insert(id, Rc::clone(&asset));

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
    /// Get an asset or load it from the asset source.
    #[inline]
    pub(crate) fn get_or_insert<T>(&mut self, id: &str, asset_source: &AssetSource) -> Rc<T>
    where
        T: Loadable,
    {
        // Return a reference to the asset when it already exists, otherwise insert it
        self.get(id)
            // Asset not found, load it
            .map_or_else(|| self.insert(id, asset_source), |asset| asset)
    }

    /// Return an asset if it exists.
    #[inline]
    #[track_caller]
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn get<T>(&self, id: &str) -> Option<Rc<T>>
    where
        T: Loadable,
    {
        // Try to find the asset
        let dyn_asset = self.assets.get(id)?;

        // Try to downcast it to the requested type
        match Rc::clone(dyn_asset).downcast_rc::<T>() {
            Ok(asset) => Some(asset),
            Err(_) => {
                panic!("Could downcast asset with ID '{id}', loaded type is different from requested type")
            }
        }
    }

    /// Upload a new asset.
    #[inline]
    #[track_caller]
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn insert<T>(&mut self, id: &str, asset_source: &AssetSource) -> Rc<T>
    where
        T: Loadable,
    {
        log::debug!("Asset '{id}' not loaded yet, loading from source");

        // Create owned ID
        let id = Id::new(id);

        // Load the asset
        let asset: Rc<dyn Loadable> = Rc::new(T::load(&id, asset_source));

        // Store the asset so it can be accessed later again
        self.assets.insert(id, Rc::clone(&asset));

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
    /// Special manager for images.
    images: ImageManager,
    /// Source for all un-loaded assets.
    source: AssetSource,
}

impl AssetsManager {
    /// Create from an asset source.
    pub(crate) fn new(source: AssetSource) -> Self {
        let sprites = AssetManager::default();
        let fonts = AssetManager::default();
        let audio = AssetManager::default();
        let images = ImageManager::new();
        let custom = CustomAssetManager::default();

        Self {
            sprites,
            fonts,
            audio,
            custom,
            images,
            source,
        }
    }

    /// Create a new empty sprite asset.
    ///
    /// # Panics
    ///
    /// - When sprite asset could not be created.
    #[inline]
    pub(crate) fn new_sprite(&mut self, id: &str, size: Size2) {
        let width = size.width as u32;
        let height = size.width as u32;

        /*
        self.source
            .create_image(Id::new(id), Size2::new(width, height));
            */
        todo!()
    }

    /// Get or load a sprite.
    ///
    /// # Panics
    ///
    /// - When sprite asset could not be loaded.
    #[inline]
    pub(crate) fn sprite(&mut self, id: &str) -> Rc<Sprite> {
        self.sprites.get_or_insert(id, &self.source)
    }

    /// Get or load a font.
    ///
    /// # Panics
    ///
    /// - When font asset could not be loaded.
    #[inline]
    pub(crate) fn font(&mut self, id: &str) -> Rc<Font> {
        self.fonts.get_or_insert(id, &self.source)
    }

    /// Get or load an audio file.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    #[inline]
    pub(crate) fn audio(&mut self, id: &str) -> Rc<Audio> {
        self.audio.get_or_insert(id, &self.source)
    }

    /// Get or load a custom asset.
    ///
    /// # Panics
    ///
    /// - When audio asset could not be loaded.
    /// - When type used to load the asset mismatches the type used to get it.
    #[inline]
    pub(crate) fn custom<T>(&mut self, id: &str) -> Rc<T>
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
    pub(crate) fn custom_owned<T>(&mut self, id: &str) -> T
    where
        T: Loadable + Clone,
    {
        // Create a clone of the asset
        Rc::<T>::unwrap_or_clone(self.custom.get_or_insert::<T>(id, &self.source))
    }

    /*
    /// Take a list of unuploaded images we still need to upload.
    #[cfg(feature = "embed-assets")]
    pub(crate) fn take_images_for_uploading(&mut self) -> Vec<(AtlasRef, PngReader)> {
        // TODO: allow runtime images
        Vec::new()
    }

    /// Update based on the assets that have been changed.
    #[cfg(feature = "hot-reload-assets")]
    pub(crate) fn process_hot_reloaded_assets(&mut self) {
        // Clean the image cache
        self.source.process_hot_reloaded_assets();

        // Take each item from the updated assets list
        for changed_asset in hot_reload::global_assets_updated().lock().unwrap().drain() {
            log::debug!("Reloading asset '{changed_asset}'");

            // Remove the changed asset from each manager, this will trigger the asset to load again when accessed
            self.sprites.assets.remove(&changed_asset);
            self.fonts.assets.remove(&changed_asset);
            self.audio.assets.remove(&changed_asset);
            self.custom.assets.remove(&changed_asset);
        }
    }
    */
}
