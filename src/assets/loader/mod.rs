//! Asset loader helper.

pub mod ogg;
pub mod png;
pub mod ron;

/// How an asset should be loaded.
pub trait Loader<T> {
    /// Extension for the file that this loader loads.
    const EXTENSION: &'static str;

    /// Load an asset from raw bytes.
    ///
    /// # Panics
    ///
    /// - When anything went wrong with loading the asset.
    fn load(bytes: &[u8]) -> T;
}
