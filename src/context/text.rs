//! Zero-cost abstraction types for building more complicated text drawing constructions.

use crate::Context;

/// Specify how the text should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::text`].
pub struct TextContext<'path, 'text, 'ctx> {
    /// Path of the font to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the text will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// X position to draw the text at.
    pub(crate) x: f32,
    /// Y position to draw the text at.
    pub(crate) y: f32,
    /// Text to draw.
    pub(crate) text: &'text str,
}

impl<'path, 'text, 'ctx> TextContext<'path, 'text, 'ctx> {
    /// Move the position of the text.
    ///
    /// # Arguments
    ///
    /// * `(x, y)` - Absolute position tuple of the target text on the buffer in pixels, will be offset by the text offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate(mut self, position: impl Into<(f32, f32)>) -> Self {
        let (x, y) = position.into();
        self.x += x;
        self.y += y;

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
    pub fn draw(self) {}
}

/// Render methods for text.
impl Context {
    /// Handle text assets, mostly used for drawing.
    ///
    /// This will load the text asset from disk and upload it to the GPU the first time this text is referenced.
    /// Check the [`TextContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the text, see [`Self`] for more information about asset loading and storing.
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to specify the location and other drawing properties of the text.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    #[must_use]
    pub const fn text<'path, 'text>(
        &self,
        path: &'path str,
        text: &'text str,
    ) -> TextContext<'path, 'text, '_> {
        TextContext {
            path,
            ctx: self,
            x: 0.0,
            y: 0.0,
            text,
        }
    }
}
