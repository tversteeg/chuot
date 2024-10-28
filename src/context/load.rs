//! Define how assets can be loaded from different resources in contexts.

use std::rc::Rc;

use super::ContextInner;
use crate::assets::loadable::sprite::Sprite;

/// How a sprite should be loaded.
pub trait LoadMethod {
    /// Return a reference to the actual sprite.
    #[allow(private_interfaces)]
    fn sprite(&self, ctx: &mut ContextInner) -> Rc<Sprite>;
}

/// Load a sprite from a path reference.
pub struct ByPath<'path>(&'path str);

impl<'path> ByPath<'path> {
    /// Create from an existing path.
    #[inline]
    #[must_use]
    pub const fn new(path: &'path str) -> Self {
        Self(path)
    }

    /// Get the internal path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &'path str {
        self.0
    }
}

impl LoadMethod for ByPath<'_> {
    #[inline]
    #[must_use]
    #[allow(private_interfaces)]
    fn sprite(&self, ctx: &mut ContextInner) -> Rc<Sprite> {
        ctx.sprite(self.0)
    }
}

/// Load a sprite directly from memory.
pub struct FromMemory(Rc<Sprite>);

impl FromMemory {
    /// Create from an existing sprite.
    #[inline]
    #[must_use]
    pub(crate) fn new(sprite: Sprite) -> Self {
        Self(Rc::new(sprite))
    }
}

impl LoadMethod for FromMemory {
    #[inline]
    #[allow(private_interfaces)]
    fn sprite(&self, _ctx: &mut ContextInner) -> Rc<Sprite> {
        Rc::clone(&self.0)
    }
}
