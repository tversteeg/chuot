//! Zero-cost abstraction types for building more complicated text drawing constructions.

use std::marker::PhantomData;

use glam::Affine2;

use super::extensions::{
    Empty,
    camera::{IsUiCamera, MainCamera, UiCamera},
    translate::{PreviousTranslation, Translate, TranslatePrevious, Translation},
};
use crate::Context;

/// Specify how the text should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::text`].
pub struct TextContext<'font, 'text, 'ctx, T, P, C> {
    /// Path of the font to draw.
    pub(crate) font: &'font str,
    /// Reference to the context the text will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Text to draw.
    pub(crate) text: &'text str,
    /// Possible translation implementation, determined by type.
    pub(crate) translation: T,
    /// Possible previous translation implementation, determined by type.
    pub(crate) previous_translation: P,
    /// Generic types without any concrete fields.
    pub(crate) phantom: PhantomData<C>,
}

impl<'font, 'text, 'ctx, T: Translate, P: TranslatePrevious, C: IsUiCamera>
    TextContext<'font, 'text, 'ctx, T, P, C>
{
    /// Only move the horizontal position.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    pub fn translate_x(self, x: f32) -> TextContext<'font, 'text, 'ctx, Translation, P, C> {
        self.translate_impl((x, 0.0))
    }

    /// Only move the vertical position.
    ///
    /// # Arguments
    ///
    /// * `y` - Vertical position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    pub fn translate_y(self, y: f32) -> TextContext<'font, 'text, 'ctx, Translation, P, C> {
        self.translate_impl((0.0, y))
    }

    /// Move the position.
    ///
    /// # Arguments
    ///
    /// * `(x, y)` - Position tuple on the buffer in pixels.
    #[inline]
    #[must_use]
    pub fn translate(
        self,
        position: impl Into<(f32, f32)>,
    ) -> TextContext<'font, 'text, 'ctx, Translation, P, C> {
        self.translate_impl(position.into())
    }

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
    pub fn translate_previous_x(
        self,
        previous_x: f32,
    ) -> TextContext<'font, 'text, 'ctx, T, PreviousTranslation, C> {
        self.translate_previous_impl((previous_x, 0.0))
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
    pub fn translate_previous_y(
        self,
        previous_y: f32,
    ) -> TextContext<'font, 'text, 'ctx, T, PreviousTranslation, C> {
        self.translate_previous_impl((0.0, previous_y))
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
    pub fn translate_previous(
        self,
        previous_position: impl Into<(f32, f32)>,
    ) -> TextContext<'font, 'text, 'ctx, T, PreviousTranslation, C> {
        self.translate_previous_impl(previous_position.into())
    }

    /// Use the UI camera instead of the regular game camera for transforming the drawable object.
    #[inline]
    #[must_use]
    pub fn use_ui_camera(self) -> TextContext<'font, 'text, 'ctx, T, P, UiCamera> {
        TextContext {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }

    /// Use the regular game camera instead of the UI camera for transforming the drawable object.
    #[inline]
    #[must_use]
    pub fn use_main_camera(self) -> TextContext<'font, 'text, 'ctx, T, P, MainCamera> {
        TextContext {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }

    /// Perform the translation with the type.
    #[inline]
    #[must_use]
    fn translate_impl(
        self,
        position: (f32, f32),
    ) -> TextContext<'font, 'text, 'ctx, Translation, P, C> {
        let translation = self.translation.inner_translate(position);

        TextContext {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }

    /// Perform the previous translation with the type.
    #[inline]
    #[must_use]
    fn translate_previous_impl(
        self,
        previous_position: (f32, f32),
    ) -> TextContext<'font, 'text, 'ctx, T, PreviousTranslation, C> {
        let previous_translation = self
            .previous_translation
            .inner_translate_previous(previous_position);

        TextContext {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation,
            phantom: PhantomData,
        }
    }
}

/// Nothing.
impl<C: IsUiCamera> TextContext<'_, '_, '_, Empty, Empty, C> {
    /// Draw the text to the screen at the zero coordinate of the camera.
    ///
    /// Text glyphs and other sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize
        TextContext {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: Translation::default(),
            previous_translation: self.previous_translation,
            phantom: self.phantom,
        }
        .draw();
    }
}

/// Only translation.
impl<C: IsUiCamera> TextContext<'_, '_, '_, Translation, Empty, C> {
    /// Draw the text to the screen.
    ///
    /// Text glyphs and other sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: reduce duplicate code
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let font = ctx.font(self.font);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Put the start position back 1 glyph since the first action is to move the cursor
            let start_x = self.translation.x + offset_x - font.metadata.glyph_width;
            let mut x = start_x;
            let mut y = self.translation.y + offset_y;

            // Draw each character from the string
            self.text.chars().for_each(|ch| {
                let char_index = ch as usize;

                // Move the cursor
                x += font.metadata.glyph_width;

                // Don't draw characters that are not in the picture
                if char_index < font.metadata.first_char || char_index > font.metadata.last_char {
                    if ch == '\n' {
                        x = start_x;
                        y += font.metadata.glyph_height;
                    } else if ch == '\t' {
                        x += font.metadata.glyph_width * 3.0;
                    }
                    return;
                }

                // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
                let char_offset = char_index - font.metadata.first_char;

                // Setup the sprite for the glyph
                let sprite = font.sprites[char_offset];

                // Get the sprite offset
                let (mut sprite_x, mut sprite_y) =
                    sprite.pivot_offset(sprite.pivot_x(), sprite.pivot_y());

                // Offset the sprite with the camera and the local position
                sprite_x += offset_x + x;
                sprite_y += offset_y + y;

                // Create the affine matrix
                let affine_matrix = Affine2::from_translation((sprite_x, sprite_y).into());

                // Push the graphics
                ctx.graphics
                    .instances
                    .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
            });
        });
    }
}

/// Translation and previous translation.
impl<C: IsUiCamera> TextContext<'_, '_, '_, Translation, PreviousTranslation, C> {
    /// Draw the text smoothly to the screen, interpolating the position in the render step.
    ///
    /// Text glyphs and other sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: reduce duplicate code
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let font = ctx.font(self.font);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Interpolate with the previous location for smooth rendering
            let mut x = crate::math::lerp(
                self.previous_translation.previous_x,
                self.translation.x,
                ctx.blending_factor,
            );
            let mut y = crate::math::lerp(
                self.previous_translation.previous_y,
                self.translation.y,
                ctx.blending_factor,
            );

            // Put the start position back 1 glyph since the first action is to move the cursor
            let start_x = x + offset_x - font.metadata.glyph_width;
            x = start_x;
            y += offset_y;

            // Draw each character from the string
            self.text.chars().for_each(|ch| {
                let char_index = ch as usize;

                // Move the cursor
                x += font.metadata.glyph_width;

                // Don't draw characters that are not in the picture
                if char_index < font.metadata.first_char || char_index > font.metadata.last_char {
                    if ch == '\n' {
                        x = start_x;
                        y += font.metadata.glyph_height;
                    } else if ch == '\t' {
                        x += font.metadata.glyph_width * 3.0;
                    }
                    return;
                }

                // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
                let char_offset = char_index - font.metadata.first_char;

                // Setup the sprite for the glyph
                let sprite = font.sprites[char_offset];

                // Get the sprite offset
                let (mut sprite_x, mut sprite_y) =
                    sprite.pivot_offset(sprite.pivot_x(), sprite.pivot_y());

                // Offset the sprite with the camera and the local position
                sprite_x += offset_x + x;
                sprite_y += offset_y + y;

                // Create the affine matrix
                let affine_matrix = Affine2::from_translation((sprite_x, sprite_y).into());

                // Push the graphics
                ctx.graphics
                    .instances
                    .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
            });
        });
    }
}

/// Render methods for text.
impl Context {
    /// Handle text drawing using a font asset, mostly used for drawing.
    ///
    /// This will load the font asset from disk and upload it to the GPU the first time this text is referenced.
    /// Check the [`TextContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `font` - Asset path of the font, see [`Self`] for more information about asset loading and storing.
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
    pub const fn text<'font, 'text>(
        &self,
        font: &'font str,
        text: &'text str,
    ) -> TextContext<'font, 'text, '_, Empty, Empty, MainCamera> {
        TextContext {
            font,
            ctx: self,
            text,
            translation: Empty,
            previous_translation: Empty,
            phantom: PhantomData,
        }
    }
}
