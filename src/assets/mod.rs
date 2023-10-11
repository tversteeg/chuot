use std::{ops::Deref, sync::OnceLock};

use assets_manager::{AssetCache, AssetGuard, Compound};

/// Either an instantiated asset or a reference to it.
#[derive(Debug)]
pub enum AssetOrPath<T: Compound> {
    /// Asset path.
    Path(String),
    /// Instantiated asset.
    Owned(T),
}

impl<'a, T: Compound> From<&'a AssetOrPath<T>> for LoadedAsset<'a, T> {
    fn from(value: &'a AssetOrPath<T>) -> Self {
        match value {
            AssetOrPath::Path(path) => LoadedAsset::Guard(crate::asset::<T, _>(path)),
            AssetOrPath::Owned(asset) => LoadedAsset::Ref(asset),
        }
    }
}

impl<T: Compound> From<String> for AssetOrPath<T> {
    fn from(val: String) -> Self {
        AssetOrPath::Path(val)
    }
}

impl<T: Compound> From<&str> for AssetOrPath<T> {
    fn from(val: &str) -> Self {
        AssetOrPath::Path(val.to_string())
    }
}

/// Loaded asset.
pub enum LoadedAsset<'a, T: Compound> {
    /// Loaded from path with guard.
    Guard(AssetGuard<'a, T>),
    /// Reference.
    Ref(&'a T),
}

impl<T: Compound> Deref for LoadedAsset<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self {
            LoadedAsset::Guard(guard) => guard.deref(),
            LoadedAsset::Ref(reference) => *reference,
        }
    }
}

/// How the assets are loaded.
#[cfg(not(feature = "embedded-assets"))]
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
        #[cfg(not(feature = "embedded-assets"))]
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
