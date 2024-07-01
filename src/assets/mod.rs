//! Asset loading and management.

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod hot_reload;
pub mod loadable;
pub mod loader;
#[doc(hidden)]
pub mod source;

use std::rc::Rc;

use hashbrown::HashMap;
use smol_str::SmolStr;

use self::loadable::Loadable;

/// Identifier for any loadable asset, can be assigned multiple times for different types.
///
/// When the string is smaller than 23 bytes this will be stored on the stack.
pub type Id = SmolStr;

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
    /// Return an asset if it exists.
    #[inline]
    pub(crate) fn get(&self, id: &Id) -> Option<Rc<T>> {
        self.assets.get(id).cloned()
    }

    /// Insert the loaded asset so it can be accessed.
    #[inline]
    pub(crate) fn insert(&mut self, id: Id, asset: T) -> Rc<T> {
        // Wrap the asset
        let asset = Rc::new(asset);

        // Store the asset so it can be accessed later again
        self.assets.insert(id, Rc::clone(&asset));

        asset
    }

    /// Remove a loaded asset, mainly used for hot-reloading.
    #[inline]
    pub(crate) fn remove(&mut self, id: &Id) {
        self.assets.remove(id);
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
    /// Return an asset if it exists.
    #[inline]
    pub(crate) fn get<T>(&self, id: &str) -> Option<Rc<T>>
    where
        T: Loadable,
    {
        // Try to find the asset
        let dyn_asset = self.assets.get(id)?;

        // Try to downcast it to the requested type
        Rc::clone(dyn_asset)
            .downcast_rc::<T>()
            .map_or_else(|_| panic!("Could downcast asset with ID '{id}', loaded type is different from requested type"), Some)
    }

    /// Upload a new asset.
    #[inline]
    pub(crate) fn insert<T>(&mut self, id: Id, asset: T) -> Rc<T>
    where
        T: Loadable,
    {
        // Load the asset
        let asset: Rc<dyn Loadable> = Rc::new(asset);

        // Store the asset so it can be accessed later again
        self.assets.insert(id, Rc::clone(&asset));

        // Safe to unwrap because we created the type here
        asset
            .downcast_rc::<T>()
            .unwrap_or_else(|_| panic!("Error downcasting type"))
    }

    /// Remove a loaded asset, mainly used for hot-reloading.
    #[inline]
    pub(crate) fn remove(&mut self, id: &Id) {
        self.assets.remove(id);
    }
}

impl Default for CustomAssetManager {
    fn default() -> Self {
        let assets = HashMap::new();

        Self { assets }
    }
}
