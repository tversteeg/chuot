//! RON asset loading.

use nanoserde::DeRon;

use super::Loader;

/// RON audio asset loader.
#[non_exhaustive]
pub struct RonLoader;

impl<T: DeRon> Loader<T> for RonLoader {
    const EXTENSION: &'static str = "ron";

    #[inline]
    fn load(bytes: &[u8]) -> T {
        // Convert raw bytes to a valid UTF-8 string
        let string = String::from_utf8(bytes.to_vec()).unwrap();

        // Deserialize the RON
        DeRon::deserialize_ron(&string).unwrap()
    }
}
