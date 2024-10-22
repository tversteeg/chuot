//! Audio asset.

use kira::sound::static_sound::StaticSoundData;

use super::Loadable;
use crate::{
    assets::{Id, loader::ogg::OggLoader},
    context::ContextInner,
};

/// Audio asset that can be loaded with metadata.
pub(crate) struct Audio(pub(crate) StaticSoundData);

impl Loadable for Audio {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self> {
        let sound_data = ctx
            .asset_source
            .load_if_exists::<OggLoader, StaticSoundData>(id)?;

        Some(Self(sound_data))
    }
}
