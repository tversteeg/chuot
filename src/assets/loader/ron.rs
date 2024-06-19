//! RON asset loading.

use nanoserde::{DeRon, DeRonState};

use super::Loader;

/// RON audio asset loader.
#[non_exhaustive]
pub struct RonLoader;

impl<T: DeRon> Loader<T> for RonLoader {
    const EXTENSION: &'static str = "ron";

    #[inline]
    fn load(bytes: &[u8]) -> T {
        // Convert raw bytes to a valid UTF-8 string
        let string = String::from_utf8(bytes.to_vec())
            .expect("Error parsing file due to invalid UTF-8 bytes");

        // Get an iterator over the characters
        let mut string_chars = string.chars();

        // RON deserialization state
        let mut state = DeRonState::default();

        // Deserialize the RON
        T::de_ron(&mut state, &mut string_chars).expect("Error parsing RON file")
    }
}
