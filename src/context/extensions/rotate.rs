//! Rotation.

/// Allow modifying rotation.
pub trait Rotate: Sized {
    /// What type the builder will return.
    type Into: Sized;

    /// Rotate.
    ///
    /// Rotation will always be applied before translation, this mean it will always rotate around the center point specified in the sprite offset metadata.
    ///
    /// # Arguments
    ///
    /// * `rotation` - Rotation in radians, will be applied using the algorithm passed in [`crate::config::Config::with_rotation_algorithm`].
    #[must_use]
    fn rotate(self, rotation: f32) -> Self::Into;
}

/// Rotation in radians.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct Rotation(pub(crate) f32);

impl Rotation {
    /// Create from tuple.
    pub(crate) const fn new(rotation: f32) -> Self {
        Self(rotation)
    }
}

impl Rotate for Rotation {
    type Into = Self;

    #[inline]
    fn rotate(mut self, rotation: f32) -> Self::Into {
        self.0 += rotation;

        self
    }
}
