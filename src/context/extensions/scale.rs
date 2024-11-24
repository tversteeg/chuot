//! Scaling.

use super::Empty;

/// Allow modifying scaling.
#[doc(hidden)]
pub trait Scale: Sized {
    /// Implentented by crate.
    fn inner_scale(self, scale: (f32, f32)) -> Scaling;
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
    #[inline]
    fn inner_scale(mut self, (scale_x, scale_y): (f32, f32)) -> Scaling {
        self.scale_x *= scale_x;
        self.scale_y *= scale_y;

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

impl Scale for Empty {
    fn inner_scale(self, scale: (f32, f32)) -> Scaling {
        Scaling::new(scale)
    }
}
