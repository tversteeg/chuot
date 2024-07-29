//! Scaling.

/// Allow modifying scaling.
pub trait Scale: Sized {
    /// What type the builder will return.
    type Into: Sized;

    /// Only scale the horizontal size.
    ///
    /// # Arguments
    ///
    /// * `scale_x` - Horizontal scale on the buffer. `-1.0` to flip.
    #[inline(always)]
    #[must_use]
    fn scale_x(self, scale_x: f32) -> Self::Into {
        self.inner_scale((scale_x, 0.0))
    }

    /// Only move the vertical position.
    ///
    /// # Arguments
    ///
    /// * `scale_y` - Vertical scale on the buffer. `-1.0` to flip.
    #[inline(always)]
    #[must_use]
    fn scale_y(self, scale_y: f32) -> Self::Into {
        self.inner_scale((0.0, scale_y))
    }

    /// Move the position.
    ///
    /// # Arguments
    ///
    /// * `(scale_x, scale_y)` - Scale tuple on the buffer.
    #[inline]
    #[must_use]
    fn scale(self, scale: impl Into<(f32, f32)>) -> Self::Into {
        self.inner_scale(scale.into())
    }

    /// Implentented by crate.
    #[doc(hidden)]
    fn inner_scale(self, scale: (f32, f32)) -> Self::Into;
}

/// Vertical and horizontal scaling.
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct Scaling {
    /// Horizontal position.
    pub(crate) scale_x: f32,
    /// Vertical position.
    pub(crate) scale_y: f32,
}

impl Scaling {
    /// Create from tuple.
    pub(crate) const fn new((scale_x, scale_y): (f32, f32)) -> Self {
        Self { scale_x, scale_y }
    }
}

impl Scale for Scaling {
    type Into = Self;

    #[inline]
    fn inner_scale(mut self, (scale_x, scale_y): (f32, f32)) -> Self::Into {
        self.scale_x += scale_x;
        self.scale_y += scale_y;

        self
    }
}

impl Default for Scaling {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}
