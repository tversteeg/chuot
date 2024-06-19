//! Font asset.

use crate::assets::{source::AssetSource, Id};

use super::Loadable;

/// Font asset that can be loaded with metadata.
pub(crate) struct Font;

impl Loadable for Font {
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
