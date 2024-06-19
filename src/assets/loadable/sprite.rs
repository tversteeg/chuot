//! Sprite asset.

use crate::assets::{source::AssetSource, Id};

use super::Loadable;

/// Sprite asset that can be loaded with metadata.
pub(crate) struct Sprite;

impl Loadable for Sprite {
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
