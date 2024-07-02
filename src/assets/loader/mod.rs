//! Asset loader helper.

pub mod ogg;
pub mod png;
pub mod ron;

use super::Id;

/// How an asset should be loaded.
pub trait Loader<T> {
    /// Extension for the file that this loader loads.
    const EXTENSION: &'static str;

    /// Load an asset from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw bytes from the file, could be loaded either from disk or from memory.
    /// * `id` - ID of the asset to load, mainly used for printing somewhat useful panic messages.
    ///
    /// # Panics
    ///
    /// - When anything went wrong with loading the asset.
    fn load(bytes: &[u8], id: &Id) -> T;
}
