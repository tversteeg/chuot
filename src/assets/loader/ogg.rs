//! OGG asset loading.

use std::io::Cursor;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

use super::Loader;

/// OGG audio asset loader.
pub struct OggLoader;

impl Loader<StaticSoundData> for OggLoader {
    const EXTENSION: &'static str = "ogg";

    fn load(bytes: &[u8]) -> StaticSoundData {
        // Allocate the bytes into a cursor
        let bytes = Cursor::new(bytes.to_vec());

        // Parse the sound file
        StaticSoundData::from_cursor(bytes, StaticSoundSettings::new())
            .expect("Error loading audio")
    }
}
