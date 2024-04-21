//! TOML asset loading.

use serde::Deserialize;

use super::Loader;

/// TOML audio asset loader.
pub struct TomlLoader;

impl<T> Loader<T> for TomlLoader
where
    T: for<'de> Deserialize<'de>,
{
    const EXTENSION: &'static str = "toml";

    fn load(bytes: &[u8]) -> T {
        // Convert raw bytes to a valid UTF-8 string
        let string = String::from_utf8(bytes.to_vec())
            .expect("Error parsing file due to invalid UTF-8 bytes");

        toml::from_str::<T>(&string).expect("Error parsing TOML file")
    }
}
