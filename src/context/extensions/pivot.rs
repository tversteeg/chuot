//! Pivoting.

use super::Empty;
use crate::pivot::Pivot as SpritePivot;

/// Allow modifying pivot.
pub trait Pivot: Sized {
    /// Get the result struct that can be used to obtain a value.
    fn default_or_value(self) -> Pivoting;

    /// Get the pivot value used for offsetting.
    fn pivot_value(self, source: SpritePivot) -> SpritePivot;
}

/// Pivoting.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct Pivoting(SpritePivot);

impl Pivoting {
    /// Create from tuple.
    pub(crate) const fn new(pivot: SpritePivot) -> Self {
        Self(pivot)
    }
}

impl Pivot for Pivoting {
    #[inline]
    fn default_or_value(self) -> Self {
        self
    }

    #[inline]
    fn pivot_value(self, _source: SpritePivot) -> SpritePivot {
        self.0
    }
}

impl Pivot for Empty {
    #[inline]
    fn default_or_value(self) -> Pivoting {
        Pivoting::new(SpritePivot::default())
    }

    #[inline]
    fn pivot_value(self, source: SpritePivot) -> SpritePivot {
        source
    }
}
