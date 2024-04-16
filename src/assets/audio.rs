//! Load `.ogg` audio assets.

use std::{borrow::Cow, io::Cursor};

use assets_manager::{loader::Loader, Asset, BoxedError};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use miette::Result;

use super::Loadable;

/// Audio asset for playing sounds and music.
#[derive(Debug)]
pub struct Audio(pub(crate) StaticSoundData);

impl Asset for Audio {
    // We only support OGG files currently
    const EXTENSION: &'static str = "ogg";

    type Loader = AudioLoader;
}

impl Loadable for Audio {
    const EXTENSION: &'static str = "ogg";

    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

/// Audio asset loader.
pub struct AudioLoader;

impl Loader<Audio> for AudioLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Audio, BoxedError> {
        // Allocate the bytes into a cursor
        let bytes = Cursor::new(content.into_owned());

        // Parse the sound file
        let sound_data = StaticSoundData::from_cursor(bytes, StaticSoundSettings::new())?;

        Ok(Audio(sound_data))
    }
}
