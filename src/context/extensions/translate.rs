//! Translation.

/// Allow modifying horizontal and vertical movement.
pub trait Translate: Sized {
    /// What type the builder will return.
    type Into: Sized;

    /// Only move the horizontal position.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    fn translate_x(self, x: f32) -> Self::Into {
        self.translate((x, 0.0))
    }

    /// Only move the vertical position.
    ///
    /// # Arguments
    ///
    /// * `y` - Vertical position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    fn translate_y(self, y: f32) -> Self::Into {
        self.translate((0.0, y))
    }

    /// Move the position.
    ///
    /// # Arguments
    ///
    /// * `(x, y)` - Position tuple on the buffer in pixels.
    #[inline]
    #[must_use]
    fn translate(self, position: impl Into<(f32, f32)>) -> Self::Into {
        self.inner_translate(position.into())
    }

    /// Implentented by crate.
    #[doc(hidden)]
    fn inner_translate(self, position: (f32, f32)) -> Self::Into;
}

/// Allow modifying horizontal and vertical movement of the previous update tick.
///
/// This ensures smooth rendering can be automatically computed during the render tick.
pub trait TranslatePrevious: Sized {
    /// What type the builder will return.
    type Into: Sized;

    /// Only move the previous horizontal position for smooth rendering based on the blending factor.
    ///
    /// This only makes sense to call when there's multiple update ticks in a single render tick.
    ///
    /// Calculated as:
    ///
    /// ```
    /// # fn func(x: f32, previous_x: f32, ctx: chuot::Context) -> f32{
    /// chuot::lerp(previous_x, x, ctx.blending_factor())
    /// # }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `previous_x` - Horizontal position in the previous update tick on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    fn translate_previous_x(self, previous_x: f32) -> Self::Into {
        self.inner_translate_previous((previous_x, 0.0))
    }

    /// Only move the previous vertical position for smooth rendering based on the blending factor.
    ///
    /// This only makes sense to call when there's multiple update ticks in a single render tick.
    ///
    /// Calculated as:
    ///
    /// ```
    /// # fn func(y: f32, previous_y: f32, ctx: chuot::Context) -> f32{
    /// chuot::lerp(previous_y, y, ctx.blending_factor())
    /// # }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `previous_y` - Vertical position in the previous update tick on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    fn translate_previous_y(self, previous_y: f32) -> Self::Into {
        self.inner_translate_previous((0.0, previous_y))
    }

    /// Move the previous position for smooth rendering based on the blending factor.
    ///
    /// This only makes sense to call when there's multiple update ticks in a single render tick.
    ///
    /// Calculated as:
    ///
    /// ```
    /// # fn func(x: f32, y: f32, previous_x: f32, previous_y: f32, ctx: chuot::Context) -> (f32, f32) {(
    /// chuot::lerp(previous_x, x, ctx.blending_factor()),
    /// chuot::lerp(previous_y, y, ctx.blending_factor())
    /// # )}
    /// ```
    ///
    /// # Arguments
    ///
    /// * `(previous_x, previous_y)` - Position tuple in the previous update tick on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    fn translate_previous(self, previous_position: impl Into<(f32, f32)>) -> Self::Into {
        self.inner_translate_previous(previous_position.into())
    }

    /// Implentented by crate.
    #[doc(hidden)]
    fn inner_translate_previous(self, previous_position: (f32, f32)) -> Self::Into;
}

/// Vertical and horizontal translation.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct Translation {
    /// Horizontal position.
    pub(crate) x: f32,
    /// Vertical position.
    pub(crate) y: f32,
}

impl Translation {
    /// Create from tuple.
    pub(crate) const fn new((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl Translate for Translation {
    type Into = Self;

    #[inline]
    fn inner_translate(mut self, (x, y): (f32, f32)) -> Self::Into {
        self.x += x;
        self.y += y;

        self
    }
}

/// Vertical and horizontal previous translation.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct PreviousTranslation {
    /// Horizontal position.
    pub(crate) previous_x: f32,
    /// Vertical position.
    pub(crate) previous_y: f32,
}

impl PreviousTranslation {
    /// Create from tuple.
    pub(crate) const fn new((previous_x, previous_y): (f32, f32)) -> Self {
        Self {
            previous_x,
            previous_y,
        }
    }
}

impl TranslatePrevious for PreviousTranslation {
    type Into = Self;

    #[inline]
    fn inner_translate_previous(mut self, (previous_x, previous_y): (f32, f32)) -> Self::Into {
        self.previous_x += previous_x;
        self.previous_y += previous_y;

        self
    }
}
