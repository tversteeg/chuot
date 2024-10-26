//! Pivoting.

use super::Empty;
use crate::pivot::Pivot as SpritePivot;

/// Allow modifying pivot.
pub trait Pivot: Sized {
    /// Implentented by crate.
    fn default_or_value(self) -> Pivoting;
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

    /// Pivoting value.
    #[inline]
    #[must_use]
    pub(crate) const fn value(self) -> SpritePivot {
        self.0
    }
}

impl Pivot for Pivoting {
    #[inline]
    fn default_or_value(self) -> Self {
        self
    }
}

impl Pivot for Empty {
    #[inline]
    fn default_or_value(self) -> Pivoting {
        Pivoting::new(SpritePivot::default())
    }
}
