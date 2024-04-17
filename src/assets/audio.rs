//! Load `.ogg` audio assets.

use std::{borrow::Cow, io::Cursor};

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use miette::Result;

use super::{loader::Loader, AssetSource, Loadable};

/// Audio asset for playing sounds and music.
#[derive(Debug)]
pub struct Audio(pub(crate) StaticSoundData);

impl Loadable for Audio {
    fn load(asset_source: &AssetSource) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

/// Audio asset loader.
pub struct AudioLoader;

impl Loader<Audio> for AudioLoader {
    fn load(bytes: &[u8]) -> Audio {
        // Allocate the bytes into a cursor
        let bytes = Cursor::new(bytes.to_vec());

        // Parse the sound file
        let sound_data = StaticSoundData::from_cursor(bytes, StaticSoundSettings::new())
            .expect("Error loading audio");

        Audio(sound_data)
    }
}
