//! RON asset loading.

use nanoserde::DeRon;

use crate::assets::Id;

use super::Loader;

/// RON file asset loader.
#[non_exhaustive]
pub struct RonLoader;

impl<T: DeRon> Loader<T> for RonLoader {
    const EXTENSION: &'static str = "ron";

    #[inline]
    fn load(bytes: &[u8], id: &Id) -> T {
        // Convert raw bytes to a valid UTF-8 string
        let string = String::from_utf8_lossy(bytes);

        // Deserialize the RON
        match DeRon::deserialize_ron(&string) {
            Ok(de) => de,
            Err(err) => panic!("Error loading RON asset with ID '{id}':\n{err}"),
        }
    }
}
