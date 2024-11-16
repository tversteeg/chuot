//! Generic pivoting struct but only used for sprites.

use nanoserde::DeRon;

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, DeRon)]
#[non_exhaustive]
pub enum Pivot {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    ///
    /// Equivalent to:
    ///
    /// ```
    /// Pivot::Fraction { x: 0.5, y: 0.5 }
    /// ```
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    ///
    /// Equivalent to:
    ///
    /// ```
    /// Pivot::Fraction { x: 0.5, y: 0.0 }
    /// ```
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    ///
    /// Equivalent to:
    ///
    /// ```
    /// Pivot::Fraction { x: 0.0, y: 0.0 }
    /// ```
    #[default]
    LeftTop,
    /// Sprite will be offset with the absolute custom coordinates counting from the left top.
    Custom {
        /// Absolute X offset as pixels from the left.
        x: f32,
        /// Absolute Y offset as pixels from the top.
        y: f32,
    },
    /// Sprite will be offset with the relative coordinates as a fraction counting from the left top.
    ///
    /// `(0.0, 0.0)` means the top left.
    /// `(1.0, 1.0)` means the bottom left.
    Fraction {
        /// Relative X offset as a fraction from the left.
        x: f32,
        /// Relative Y offset as a fraction from the top.
        y: f32,
    },
}

impl Pivot {
    /// Get the pivot based on the sprite size.
    #[inline]
    pub(crate) fn pivot(&self, sprite_width: f32, sprite_height: f32) -> (f32, f32) {
        match self {
            Self::Middle => (-sprite_width / 2.0, -sprite_height / 2.0),
            Self::MiddleTop => (-sprite_width / 2.0, 0.0),
            Self::LeftTop => (0.0, 0.0),
            Self::Custom { x, y } => (-x, -y),
            Self::Fraction { x, y } => (-x * sprite_width, -y * sprite_height),
        }
    }
}
