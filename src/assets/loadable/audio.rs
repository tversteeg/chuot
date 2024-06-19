//! Audio asset.

use crate::assets::{source::AssetSource, Id};

use super::Loadable;

/// Audio asset that can be loaded with metadata.
pub(crate) struct Audio;

impl Loadable for Audio {
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
