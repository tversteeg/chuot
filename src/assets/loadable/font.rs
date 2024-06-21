//! Font asset.

use crate::{assets::Id, context::ContextInner};

use super::Loadable;

/// Font asset that can be loaded with metadata.
pub(crate) struct Font;

impl Loadable for Font {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
