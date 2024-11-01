//! Zero-cost abstraction types for handling the loaded fonts.

use std::marker::PhantomData;

use super::{
    Context,
    extensions::{Empty, camera::MainCamera, pivot::Pivoting},
    load::FromMemory,
    sprite::SpriteContext,
    text::TextContext,
};
use crate::Pivot;

/// Handle loaded fonts.
///
/// Used by [`crate::Context::font`].
pub struct FontContext<'font, 'ctx> {
    /// Path of the font.
    pub(crate) font: &'font str,
    /// Reference to the context.
    pub(crate) ctx: &'ctx Context,
}

impl<'font, 'ctx> FontContext<'font, 'ctx> {
    /// Get the sprite for a single glyph.
    ///
    /// The sprite will be pivoted in the middle and displayed using the main camera.
    ///
    /// # Arguments
    ///
    /// * `glyph` - character that would normally be drawn to get the sprite of.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When character is out of range.
    #[inline]
    #[must_use]
    pub fn glyph(
        &self,
        glyph: char,
    ) -> SpriteContext<'_, FromMemory, Empty, Empty, Empty, Empty, Pivoting, MainCamera> {
        let sprite = self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let font = ctx.font(self.font);

            // Get the character
            let char_offset = glyph as usize - font.metadata.first_char;
            font.sprites[char_offset]
        });

        // Create the sprite context to continue with
        SpriteContext {
            load: FromMemory::new(sprite),
            ctx: self.ctx,
            translation: Empty,
            previous_translation: Empty,
            rotation: Empty,
            scaling: Empty,
            pivot: Pivoting::new(Pivot::Middle),
            phantom: PhantomData,
        }
    }

    /// Handle text drawing.
    ///
    /// # Arguments
    ///
    /// * `text` - String of characters to draw.
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
    pub const fn text<'text>(
        self,
        text: &'text str,
    ) -> TextContext<'font, 'text, 'ctx, Empty, Empty, MainCamera> {
        TextContext {
            font: self.font,
            ctx: self.ctx,
            text,
            translation: Empty,
            previous_translation: Empty,
            phantom: PhantomData,
        }
    }
}

/// Methods for fonts.
impl Context {
    /// Handle font assets.
    ///
    /// This will load the font asset from disk and upload it to the GPU the first time this text is referenced.
    /// Check the [`FontContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `font` - Asset path of the font, see [`Self`] for more information about asset loading and storing.
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to handle tho font.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    #[must_use]
    pub const fn font<'font>(&self, font: &'font str) -> FontContext<'font, '_> {
        FontContext { font, ctx: self }
    }
}
