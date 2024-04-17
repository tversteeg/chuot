//! Asset loader helper.

/// How an asset should be loaded.
pub trait Loader<T> {
    /// Load an asset from raw bytes.
    ///
    /// # Panics
    ///
    /// - When anything went wrong with loading the asset.
    fn load(bytes: &[u8]) -> T;
}
