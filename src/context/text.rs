//! Zero-cost abstraction types for building more complicated text drawing constructions.

use crate::Context;

/// Specify how the text should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::text`].
pub struct TextContext<'font, 'text, 'ctx> {
    /// Path of the font to draw.
    pub(crate) font: &'font str,
    /// Reference to the context the text will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// X position to draw the text at.
    pub(crate) x: f32,
    /// Y position to draw the text at.
    pub(crate) y: f32,
    /// Whether to blend the rendering with the previous state.
    pub(crate) blend: bool,
    /// Previous X position to draw the sprite with blending.
    pub(crate) previous_x: f32,
    /// Previous Y position to draw the sprite with blending.
    pub(crate) previous_y: f32,
    /// Text to draw.
    pub(crate) text: &'text str,
    /// Whether to use the UI camera for positioning the text, `false` uses the regular game camera.
    pub(crate) ui_camera: bool,
}

impl<'font, 'text, 'ctx> TextContext<'font, 'text, 'ctx> {
    /// Only move the horizontal position of the text.
    ///
    /// # Arguments
    ///
    /// * `x` - Absolute horizontal position of the target text on the buffer in pixels, will be offset by the text offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate_x(mut self, x: f32) -> Self {
        self.x += x;

        self
    }

    /// Only move the vertical position of the text.
    ///
    /// # Arguments
    ///
    /// * `y` - Absolute vertical position of the target text on the buffer in pixels, will be offset by the text offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate_y(mut self, y: f32) -> Self {
        self.y += y;

        self
    }

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

    /// Use the UI camera instead of the regular game camera for transforming the sprite.
    #[inline(always)]
    #[must_use]
    pub const fn use_ui_camera(mut self) -> Self {
        self.ui_camera = true;

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
            // Push the instance if the texture is already uploaded
            let font = ctx.font(self.font);

            // Get the camera to draw the sprite with
            let camera = ctx.camera_mut(self.ui_camera);
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Calculate the difference for the smooth rendering
            let (diff_x, diff_y) = if self.blend {
                (self.previous_x - self.x, self.previous_y - self.y)
            } else {
                (0.0, 0.0)
            };

            // Put the start position back 1 glyph since the first action is to move the cursor
            let start_x = self.x + offset_x - font.metadata.glyph_width;
            let mut x = start_x;
            let mut y = self.y + offset_y;

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
                let affine_matrix = sprite.affine_matrix(
                    x,
                    y,
                    x + diff_x,
                    y + diff_y,
                    ctx.blending_factor,
                    self.blend,
                    0.0,
                    1.0,
                    1.0,
                );

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
    ) -> TextContext<'font, 'text, '_> {
        TextContext {
            font,
            ctx: self,
            x: 0.0,
            y: 0.0,
            blend: false,
            previous_x: 0.0,
            previous_y: 0.0,
            text,
            ui_camera: false,
        }
    }
}
