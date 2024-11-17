//! Pivoting.

use crate::assets::loadable::sprite::SpritePivot;

use super::Empty;

/// Allow modifying pivot.
pub trait Pivot: Sized {
    /// Get the result struct that can be used to obtain a value.
    fn default_or_value(self) -> Pivoting;

    /// Get the pivot value used for offsetting.
    fn pivot_value(
        self,
        source_x: SpritePivot,
        source_y: SpritePivot,
    ) -> (SpritePivot, SpritePivot);
}

/// Pivoting.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct Pivoting {
    /// X axis.
    x: SpritePivot,
    /// Y axis.
    y: SpritePivot,
}

impl Pivoting {
    /// Create from tuple.
    pub(crate) const fn new(x: SpritePivot, y: SpritePivot) -> Self {
        Self { x, y }
    }
}

impl Pivot for Pivoting {
    #[inline]
    fn default_or_value(self) -> Self {
        self
    }

    #[inline(always)]
    fn pivot_value(
        self,
        _source_x: SpritePivot,
        _source_y: SpritePivot,
    ) -> (SpritePivot, SpritePivot) {
        (self.x, self.y)
    }
}

impl Pivot for Empty {
    #[inline]
    fn default_or_value(self) -> Pivoting {
        Pivoting::new(SpritePivot::default(), SpritePivot::default())
    }

    #[inline]
    fn pivot_value(
        self,
        source_x: SpritePivot,
        source_y: SpritePivot,
    ) -> (SpritePivot, SpritePivot) {
        (source_x, source_y)
    }
}
