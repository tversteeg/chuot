//! Custom asset loading.
//!
//! Assets will be embedded into the binary when the `hot-reloading-assets` feature flag is disabled or a WASM build is made, since there's no filesystem there.
//!
//! Asset loading is done through the various calls in [`crate::Context`].

mod audio;
mod image;

/// Asset manager files re-exported for the macro.
#[doc(hidden)]
pub use assets_manager::source;
/// Re-export [`assets_manager`] types for loading and defining custom assets.
pub use assets_manager::{asset::*, loader::*, BoxedError};
pub use audio::{Audio, AudioLoader};
pub use image::{Image, ImageLoader};

use assets_manager::AssetReadGuard;

/// Reference to a single asset.
pub type AssetRef<'a, T> = AssetReadGuard<'a, T>;

/// Define the asset types when hot-reloading is enabled.
#[cfg(not(any(
    target_arch = "wasm32",
    not(feature = "hot-reloading-assets"),
    not(doctest)
)))]
#[doc(hidden)]
pub type AssetCacheSource = assets_manager::source::FileSystem;

/// Define the asset types when all assets should be embedded.
#[cfg(any(
    target_arch = "wasm32",
    not(feature = "hot-reloading-assets"),
    not(doctest)
))]
#[doc(hidden)]
pub type AssetCacheSource = assets_manager::source::Embedded<'static>;
