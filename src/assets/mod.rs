//! Types for creating custom assets.

pub(crate) mod image;

use std::sync::OnceLock;

use assets_manager::AssetCache;

/// How the assets are loaded.
#[cfg(not(any(target_arch = "wasm32", feature = "embedded-assets")))]
type Assets = AssetCache<assets_manager::source::FileSystem>;
/// How the assets are loaded.
#[cfg(any(target_arch = "wasm32", feature = "embedded-assets"))]
type Assets = AssetCache<assets_manager::source::Embedded<'static>>;

/// All external data.
static ASSETS: OnceLock<Assets> = OnceLock::new();

/// Get or initialize the asset cache.
pub(crate) fn asset_cache() -> &'static Assets {
    let cache = ASSETS.get_or_init(|| {
        // Load the assets from disk, allows hot-reloading
        #[cfg(not(any(target_arch = "wasm32", feature = "embedded-assets")))]
        let source = assets_manager::source::FileSystem::new("assets").unwrap();

        // Embed all assets into the binary
        #[cfg(any(target_arch = "wasm32", feature = "embedded-assets"))]
        let source =
            assets_manager::source::Embedded::from(assets_manager::source::embed!("assets"));

        AssetCache::with_source(source)
    });

    // Enable hot reloading
    #[cfg(feature = "hot-reloading-assets")]
    cache.enhance_hot_reloading();

    cache
}
