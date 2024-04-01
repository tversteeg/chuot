//! Types for creating custom assets.
//!
//! When hot-reloading is enabled the asset cache is global and all references are static.
//!
//! Asset loading is done through the [`crate::Context::asset`] and [`crate::Context::asset_owned`] calls.

mod image;

use assets_manager::AssetReadGuard;
pub use assets_manager::{asset::*, Asset, BoxedError};
pub use image::{Image, ImageLoader};

/// Reference to a single asset.
pub type AssetRef<T> = AssetReadGuard<'static, T>;

/// Define the asset types when hot-reloading is enabled.
#[cfg(not(any(target_arch = "wasm32", feature = "embedded-assets")))]
mod hot_reloaded {
    use std::sync::OnceLock;

    use assets_manager::{source::FileSystem, AssetCache};

    /// Globally shared cache.
    static ASSETS: OnceLock<AssetCache<FileSystem>> = OnceLock::new();

    /// Get or initialize the asset cache.
    pub(crate) fn asset_cache() -> &'static AssetCache<FileSystem> {
        ASSETS.get_or_init(|| AssetCache::with_source(FileSystem::new("assets").unwrap()))
    }
}
#[cfg(not(any(target_arch = "wasm32", feature = "embedded-assets")))]
pub(crate) use hot_reloaded::*;

// TODO: store the cache state in the context somehow

/// Define the asset types when embedding is enabled.
#[cfg(any(target_arch = "wasm32", feature = "embedded-assets"))]
mod embedded {
    use std::sync::OnceLock;

    use assets_manager::{source::Embedded, AssetCache};

    /// Globally shared cache.
    static ASSETS: OnceLock<AssetCache<Embedded<'static>>> = OnceLock::new();

    /// Get or initialize the asset cache.
    pub(crate) fn asset_cache() -> &'static AssetCache<Embedded<'static>> {
        ASSETS.get_or_init(|| {
            AssetCache::with_source(
                Embedded::from(assets_manager::source::embed!("assets")).to_owned(),
            )
        })
    }
}
#[cfg(any(target_arch = "wasm32", feature = "embedded-assets"))]
pub(crate) use embedded::*;

/// How the assets are loaded.
pub(crate) struct Assets;

impl Assets {
    /// Initialize the cache.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Load an asset.
    #[inline]
    pub(crate) fn asset<T>(&self, path: &str) -> AssetRef<T>
    where
        T: Compound,
    {
        asset_cache().load_expect(path).read()
    }

    /// Load a clone of an asset.
    #[inline]
    pub fn asset_owned<T>(&self, path: &str) -> T
    where
        T: Compound,
    {
        asset_cache()
            .load_owned(path)
            .expect("Could not load owned asset")
    }
}
