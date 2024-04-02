//! Zero-cost abstraction types for building more complicated text drawing constructions.

use glamour::Vector2;

use crate::Context;

/// Specify how the text should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::draw_text`].
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
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            ctx.load_font_if_not_loaded(self.path);

            ctx.fonts
                .get_mut(self.path)
                .expect("Error accessing font in context")
                .draw(self.position, self.text, &mut ctx.instances)
        });
    }
}