//! Audio asset.

use crate::{assets::Id, context::ContextInner};

use super::Loadable;

/// Audio asset that can be loaded with metadata.
pub(crate) struct Audio;

impl Loadable for Audio {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
