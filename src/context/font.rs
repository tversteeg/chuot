//! Zero-cost abstraction types for handling the loaded fonts.

use std::marker::PhantomData;

use super::{
    Context,
    extensions::{Empty, camera::MainCamera, pivot::Pivoting},
    load::FromMemory,
    sprite::SpriteContext,
    text::TextContext,
};
use crate::assets::loadable::sprite::SpritePivot;

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
        glyph: impl Into<usize>,
    ) -> SpriteContext<'_, FromMemory, Empty, Empty, Empty, Empty, Pivoting, MainCamera> {
        // Reduce compilation times
        fn inner<'ctx>(
            this: &FontContext<'_, 'ctx>,
            glyph: usize,
        ) -> SpriteContext<'ctx, FromMemory, Empty, Empty, Empty, Empty, Pivoting, MainCamera>
        {
            let sprite = this.ctx.write(|ctx| {
                // Push the instance if the texture is already uploaded
                let font = ctx.font(this.font);

                // Get the character
                let char_offset = glyph - font.metadata.first_char;
                font.sprites[char_offset]
            });

            // Create the sprite context to continue with
            SpriteContext {
                load: FromMemory::new(sprite),
                ctx: this.ctx,
                translation: Empty,
                previous_translation: Empty,
                rotation: Empty,
                scaling: Empty,
                pivot: Pivoting::new(SpritePivot::Center, SpritePivot::Center),
                phantom: PhantomData,
            }
        }

        inner(self, glyph.into())
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

    /// Get the size of a single glyph of this font.
    ///
    /// # Returns
    ///
    /// - `(glyph_width, glyph_height)`, size of a single glyph sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn glyph_size(&self) -> (f32, f32) {
        self.ctx.write(|ctx| {
            let metadata = ctx.font(self.font).metadata;

            (metadata.glyph_width, metadata.glyph_height)
        })
    }

    /// Get the width of a single glyph of this font.
    ///
    /// # Returns
    ///
    /// - `width`, horizontal size of a single glyph sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn glyph_width(&self) -> f32 {
        self.ctx
            .write(|ctx| ctx.font(self.font).metadata.glyph_width)
    }

    /// Get the height of a single glyph of this font.
    ///
    /// # Returns
    ///
    /// - `height`, vertical size of a single glyph sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn glyph_height(&self) -> f32 {
        self.ctx
            .write(|ctx| ctx.font(self.font).metadata.glyph_height)
    }

    /// Get the first 'character' glyph of this font.
    ///
    /// # Returns
    ///
    /// - `char`, first character that's rendered from the font sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn first_char(&self) -> usize {
        self.ctx
            .write(|ctx| ctx.font(self.font).metadata.first_char)
    }

    /// Get the last 'character' glyph of this font.
    ///
    /// # Returns
    ///
    /// - `char`, last character that's rendered from the font sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn last_char(&self) -> usize {
        self.ctx.write(|ctx| ctx.font(self.font).metadata.last_char)
    }

    /// Get how many glyph sprites are in this font.
    ///
    /// # Returns
    ///
    /// - `len`, glyph sprites in this font.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn chars(&self) -> usize {
        self.ctx.write(|ctx| {
            let metadata = ctx.font(self.font).metadata;

            metadata.last_char - metadata.first_char
        })
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
