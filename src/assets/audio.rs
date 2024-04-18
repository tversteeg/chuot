//! Load `.ogg` audio assets.

use kira::sound::static_sound::StaticSoundData;

use super::{loader::ogg::OggLoader, AssetSource, Id, Loadable};

/// Audio asset for playing sounds and music.
#[derive(Debug)]
pub struct Audio(pub(crate) StaticSoundData);

impl Loadable for Audio {
    type Upload = ();

    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<((), Self)>
    where
        Self: Sized,
    {
        Some(((), Self(asset_source.load_if_exists::<OggLoader, _>(id)?)))
    }
}
