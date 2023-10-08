pub(crate) mod font;
pub(crate) mod sprite;

use std::sync::OnceLock;

use assets_manager::{AssetCache, AssetGuard, Compound};

/// How the assets are loaded.
#[cfg(all(feature = "hot-reloading-assets", not(feature = "embedded-assets")))]
type Assets = AssetCache<assets_manager::source::FileSystem>;
/// How the assets are loaded.
#[cfg(feature = "embedded-assets")]
type Assets = AssetCache<assets_manager::source::Embedded<'static>>;

/// All external data.
static ASSETS: OnceLock<Assets> = OnceLock::new();

/// Load a reference to any asset.
///
/// Sets up the asset manager once, which can be accessed with the global function in this module.
///
/// # Arguments
///
/// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
pub fn asset<T, S>(path: S) -> AssetGuard<'static, T>
where
    T: Compound,
    S: AsRef<str>,
{
    asset_cache().load_expect(path.as_ref()).read()
}

/// Load a clone of any asset.
///
/// Sets up the asset manager once, which can be accessed with the global function in this module.
///
/// # Arguments
///
/// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
pub fn asset_owned<T, S>(path: S) -> T
where
    T: Compound,
    S: AsRef<str>,
{
    asset_cache()
        .load_owned(path.as_ref())
        .expect("Could not load owned asset")
}

/// Get or initialize the asset cache.
fn asset_cache() -> &'static Assets {
    let cache = ASSETS.get_or_init(|| {
        // Load the assets from disk, allows hot-reloading
        #[cfg(all(feature = "hot-reloading-assets", not(feature = "embedded-assets")))]
        let source = assets_manager::source::FileSystem::new("assets").unwrap();

        // Embed all assets into the binary
        #[cfg(feature = "embedded-assets")]
        let source =
            assets_manager::source::Embedded::from(assets_manager::source::embed!("assets"));

        AssetCache::with_source(source)
    });

    // Enable hot reloading
    #[cfg(feature = "hot-reloading-assets")]
    cache.enhance_hot_reloading();

    cache
}
