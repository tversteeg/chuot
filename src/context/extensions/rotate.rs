//! Rotation.

use super::Empty;

/// Allow modifying rotation.
pub trait Rotate: Sized {
    /// Implentented by crate.
    fn inner_rotate(self, rotate: f32) -> Rotation;
}

/// Rotation in radians.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct Rotation(f32);

impl Rotation {
    /// Create from tuple.
    pub(crate) const fn new(rotation: f32) -> Self {
        Self(rotation)
    }

    /// Rotation value.
    #[inline]
    #[must_use]
    pub(crate) const fn value(self) -> f32 {
        self.0
    }
}

impl Rotate for Rotation {
    #[inline]
    fn inner_rotate(mut self, rotate: f32) -> Rotation {
        self.0 += rotate;

        self
    }
}

impl Rotate for Empty {
    #[inline]
    fn inner_rotate(self, rotate: f32) -> Rotation {
        Rotation::new(rotate)
    }
}
