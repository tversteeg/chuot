//! OGG asset loading.

use std::io::Cursor;

use kira::sound::static_sound::StaticSoundData;

use super::Loader;
use crate::assets::Id;

/// OGG audio asset loader.
#[non_exhaustive]
pub struct OggLoader;

impl Loader<StaticSoundData> for OggLoader {
    const EXTENSION: &'static str = "ogg";

    #[inline]
    fn load(bytes: &[u8], id: &Id) -> StaticSoundData {
        // Allocate the bytes into a cursor
        let bytes = Cursor::new(bytes.to_vec());

        // Parse the sound file
        match StaticSoundData::from_cursor(bytes) {
            Ok(sound) => sound,
            Err(err) => panic!("Error loading audio file from ID '{id}': {err}"),
        }
    }
}
