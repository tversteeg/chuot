//! Translation.

use super::Empty;

/// Allow modifying horizontal and vertical movement.
#[doc(hidden)]
pub trait Translate: Sized {
    /// Implentented by crate.
    fn inner_translate(self, position: (f32, f32)) -> Translation;
}

/// Allow modifying horizontal and vertical movement of the previous update tick.
///
/// This ensures smooth rendering can be automatically computed during the render tick.
#[doc(hidden)]
pub trait TranslatePrevious: Sized {
    /// Implentented by crate.
    fn inner_translate_previous(self, previous_position: (f32, f32)) -> PreviousTranslation;
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
    #[inline]
    fn inner_translate(mut self, (x, y): (f32, f32)) -> Self {
        self.x += x;
        self.y += y;

        self
    }
}

impl Translate for Empty {
    #[inline]
    fn inner_translate(self, position: (f32, f32)) -> Translation {
        Translation::new(position)
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
    #[inline]
    pub(crate) const fn new((previous_x, previous_y): (f32, f32)) -> Self {
        Self {
            previous_x,
            previous_y,
        }
    }
}

impl TranslatePrevious for PreviousTranslation {
    #[inline]
    fn inner_translate_previous(
        mut self,
        (previous_x, previous_y): (f32, f32),
    ) -> PreviousTranslation {
        self.previous_x += previous_x;
        self.previous_y += previous_y;

        self
    }
}

impl TranslatePrevious for Empty {
    #[inline]
    fn inner_translate_previous(self, previous_position: (f32, f32)) -> PreviousTranslation {
        PreviousTranslation::new(previous_position)
    }
}
