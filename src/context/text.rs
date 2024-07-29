//! Zero-cost abstraction types for building more complicated text drawing constructions.

use std::marker::PhantomData;

use glam::Affine2;

use crate::{Camera, Context, Translate, TranslatePrevious};

use super::extensions::{
    camera::{IsUiCamera, MainCamera, UiCamera},
    draw::Draw,
    translate::{PreviousTranslation, Translation},
    Empty,
};

/// With translation, ensures the type is used properly.
type WithTranslation<'font, 'text, 'ctx, P, C> = TextContext<'font, 'text, 'ctx, Translation, P, C>;
/// Without translation, ensures the type is used properly.
type WithoutTranslation<'font, 'text, 'ctx, P, C> = TextContext<'font, 'text, 'ctx, Empty, P, C>;
/// With previous translation, ensures the type is used properly.
type WithPreviousTranslation<'font, 'text, 'ctx, T, C> =
    TextContext<'font, 'text, 'ctx, T, PreviousTranslation, C>;
/// Without previous translation, ensures the type is used properly.
type WithoutPreviousTranslation<'font, 'text, 'ctx, T, C> =
    TextContext<'font, 'text, 'ctx, T, Empty, C>;

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

impl<'font, 'text, 'ctx, T, P, C: IsUiCamera> TextContext<'font, 'text, 'ctx, T, P, C> {
    /// Draw the text.
    ///
    /// Text glyphs and other sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    fn draw_impl(self, x: f32, y: f32, blend: bool, previous_x: f32, previous_y: f32) {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let font = ctx.font(self.font);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            let (mut x, mut y) = if blend {
                // Interpolate with the previous location for smooth rendering
                (
                    crate::math::lerp(previous_x, x, ctx.blending_factor),
                    crate::math::lerp(previous_y, y, ctx.blending_factor),
                )
            } else {
                (x, y)
            };

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
                let (mut sprite_x, mut sprite_y) = sprite.offset();

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

/// No translation yet.
impl<'font, 'text, 'ctx, P, C> Translate for WithoutTranslation<'font, 'text, 'ctx, P, C> {
    type Into = WithTranslation<'font, 'text, 'ctx, P, C>;

    #[inline]
    fn inner_translate(self, translation: (f32, f32)) -> Self::Into {
        Self::Into {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: Translation::new(translation),
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }
}

/// Already has translation.
impl<'font, 'text, 'ctx, P, C> Translate for WithTranslation<'font, 'text, 'ctx, P, C> {
    type Into = Self;

    #[inline]
    fn inner_translate(mut self, translation: (f32, f32)) -> Self::Into {
        self.translation = self.translation.inner_translate(translation);

        self
    }
}

/// No previous translation yet.
impl<'font, 'text, 'ctx, T, C> TranslatePrevious
    for WithoutPreviousTranslation<'font, 'text, 'ctx, T, C>
{
    type Into = WithPreviousTranslation<'font, 'text, 'ctx, T, C>;

    #[inline]
    fn inner_translate_previous(self, previous_position: (f32, f32)) -> Self::Into {
        Self::Into {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation: PreviousTranslation::new(previous_position),
            phantom: PhantomData,
        }
    }
}

/// Already has previous translation.
impl<'font, 'text, 'ctx, T, C> TranslatePrevious
    for WithPreviousTranslation<'font, 'text, 'ctx, T, C>
{
    type Into = Self;

    #[inline]
    fn inner_translate_previous(mut self, previous_position: (f32, f32)) -> Self::Into {
        self.previous_translation = self
            .previous_translation
            .inner_translate_previous(previous_position);

        self
    }
}

/// Select the camera.
impl<'font, 'text, 'ctx, T, P, C> Camera for TextContext<'font, 'text, 'ctx, T, P, C> {
    type IntoUi = TextContext<'font, 'text, 'ctx, T, P, UiCamera>;
    type IntoMain = TextContext<'font, 'text, 'ctx, T, P, MainCamera>;

    #[inline]
    fn use_ui_camera(self) -> Self::IntoUi {
        Self::IntoUi {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn use_main_camera(self) -> Self::IntoMain {
        Self::IntoMain {
            font: self.font,
            ctx: self.ctx,
            text: self.text,
            translation: self.translation,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }
}

/// No translation, no previous translation.
impl<'font, 'text, 'ctx, C: IsUiCamera> Draw for TextContext<'font, 'text, 'ctx, Empty, Empty, C> {
    #[inline]
    fn draw(self) {
        self.draw_impl(0.0, 0.0, false, 0.0, 0.0);
    }
}

/// No previous translation.
impl<'font, 'text, 'ctx, C: IsUiCamera> Draw
    for TextContext<'font, 'text, 'ctx, Translation, Empty, C>
{
    #[inline]
    fn draw(self) {
        let translation = self.translation;

        self.draw_impl(translation.x, translation.y, false, 0.0, 0.0);
    }
}

/// Everything.
impl<'font, 'text, 'ctx, C: IsUiCamera> Draw
    for TextContext<'font, 'text, 'ctx, Translation, PreviousTranslation, C>
{
    #[inline]
    fn draw(self) {
        let translation = self.translation;
        let previous_translation = self.previous_translation;

        self.draw_impl(
            translation.x,
            translation.y,
            true,
            previous_translation.previous_x,
            previous_translation.previous_y,
        );
    }
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
    /// * `font` - Asset path of the font, see [`Self`] for more information about asset loading and storing.
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
