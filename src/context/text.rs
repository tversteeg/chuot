//! Zero-cost abstraction types for building more complicated text drawing constructions.

use glamour::Vector2;

use crate::Context;

/// Specify how the text should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::text`].
pub struct DrawTextContext<'path, 'text, 'ctx> {
    /// Path of the font to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the text will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Position to draw the text at.
    pub(crate) position: Vector2,
    /// Text to draw.
    pub(crate) text: &'text str,
}

impl<'path, 'text, 'ctx> DrawTextContext<'path, 'text, 'ctx> {
    /// Move the position of the text.
    ///
    /// # Arguments
    ///
    /// * `position` - Absolute position of the target text on the buffer in pixels, will be offset by the text offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate(mut self, offset: impl Into<Vector2>) -> Self {
        self.position += offset.into();

        self
    }

    /// Draw the text.
    ///
    /// Text glyphs and other sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            ctx.assets
                .font(self.path)
                .draw(self.position, self.text, &mut ctx.instances)
        });
    }
}
