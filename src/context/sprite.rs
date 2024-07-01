//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use crate::{
    assets::{loadable::sprite::Sprite, Id},
    Context,
};

/// Specify how a sprite should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::sprite`].
pub struct SpriteContext<'path, 'ctx> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// X position to draw the sprite at.
    pub(crate) x: f32,
    /// Y position to draw the sprite at.
    pub(crate) y: f32,
    /// Rotation in radians.
    pub(crate) rotation: f32,
}

impl<'path, 'ctx> SpriteContext<'path, 'ctx> {
    /// Only move the horizontal position of the sprite.
    ///
    /// # Arguments
    ///
    /// * `x` - Absolute horizontal position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate_x(mut self, x: f32) -> Self {
        self.x += x;

        self
    }

    /// Only move the vertical position of the sprite.
    ///
    /// # Arguments
    ///
    /// * `y` - Absolute vertical position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate_y(mut self, y: f32) -> Self {
        self.y += y;

        self
    }

    /// Move the position of the sprite.
    ///
    /// # Arguments
    ///
    /// * `(x, y)` - Absolute position tuple of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate(mut self, position: impl Into<(f32, f32)>) -> Self {
        let (x, y) = position.into();
        self.x += x;
        self.y += y;

        self
    }

    /// Rotate the sprite.
    ///
    /// Rotation will always be applied before translation, this mean it will always rotate around the center point specified in the sprite offset metadata.
    ///
    /// # Arguments
    ///
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the algorithm passed in [`crate::config::Config::with_rotation_algorithm`].
    #[inline(always)]
    #[must_use]
    pub fn rotate(mut self, rotation: f32) -> Self {
        self.rotation += rotation;

        self
    }

    /// Draw the sprite.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(self.x, self.y, self.rotation);

            // Push the graphics
            ctx.graphics
                .instances
                .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
        });
    }

    /// Optimized way to draw the sprite multiple times with a translation added to each.
    ///
    /// Calling [`Self::translate`] and/or [`Self::rotate`] before this method will create a base matrix onto which each item translation is applied afterwards.
    /// This allows you to easily draw thousands of sprites, perfect for particle effects.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Arguments
    ///
    /// * `translations` - Iterator of translation `(x, y)` tuple offsets, will draw each item as a sprite at the base position with the offset applied.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    ///
    /// # Example
    ///
    /// This example runs on my PC with an average FPS of 35 when rendering 100000 sprites:
    ///
    /// ```no_run
    /// # fn call(ctx: chuot::Context) {
    /// ctx.sprite("some_asset")
    ///     .draw_multiple_translated((0..10).map(|x| (x as f32, 0.0)));
    /// # }
    /// ```
    ///
    /// It's functionally the same as:
    ///
    /// ```no_run
    /// # fn call(ctx: chuot::Context) {
    /// for x in 0..10 {
    ///     ctx.sprite("some_asset").translate_x(x as f32).draw();
    /// }
    /// # }
    /// ```
    ///
    /// But the second example runs on my PC with an average FPS of 11 when rendering 100000 sprites.
    #[inline(always)]
    pub fn draw_multiple_translated<T>(self, translations: impl Iterator<Item = T>)
    where
        T: Into<(f32, f32)>,
    {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(self.x, self.y, self.rotation);

            // Push the graphics
            ctx.graphics
                .instances
                .extend(translations.map(|translation| {
                    let (x_offset, y_offset) = translation.into();

                    // Copy the matrix
                    let mut affine_matrix_with_offset = affine_matrix;
                    affine_matrix_with_offset.translation.x += x_offset;
                    affine_matrix_with_offset.translation.y += y_offset;

                    (
                        affine_matrix_with_offset,
                        sprite.sub_rectangle,
                        sprite.texture,
                    )
                }));
        });
    }

    /// Create a new empty sprite at runtime.
    ///
    /// # Arguments
    ///
    /// * `(width, height)` - Size tuple of the new sprite in pixels.
    /// * `pixels` - Array of RGBA `u32` pixels to use as the texture of the sprite.
    ///
    /// # Panics
    ///
    /// - When a sprite with the same ID already exists.
    /// - When `width * height != pixels.len()`.
    #[inline]
    pub fn create(self, size: impl Into<(f32, f32)>, pixels: impl AsRef<[u32]>) {
        let (width, height) = size.into();
        let pixels = pixels.as_ref();

        self.ctx.write(|ctx| {
            // Create the sprite
            let asset = Sprite::new_and_upload(width, height, pixels, ctx);

            // Register the sprite
            ctx.sprites.insert(Id::new(self.path), asset);
        });
    }

    /// Update the pixels of a portion of the sprite.
    ///
    /// # Arguments
    ///
    /// * `(x, y, width, height)` - Sub rectangle tuple within the sprite to update. Width * height must be equal to the amount of pixels, and fall within the sprite's rectangle.
    /// * `pixels` - Array of ARGB pixels, amount must match size of the sub rectangle.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When the sub rectangle does not fit inside the sprite's rectangle.
    /// - When the size of the sub rectangle does not match the amount of pixels.
    #[inline]
    pub fn update_pixels(
        self,
        sub_rectangle: impl Into<(f32, f32, f32, f32)>,
        pixels: impl AsRef<[u32]>,
    ) {
        let sub_rectangle = sub_rectangle.into();
        let pixels = pixels.as_ref();

        self.ctx.write(|ctx| {
            // Get the sprite
            let sprite = ctx.sprite(self.path);

            // Push the sprite pixels to the GPU
            ctx.graphics.atlas.update_pixels(
                sprite.texture,
                sub_rectangle,
                pixels,
                &ctx.graphics.queue,
            );
        });
    }

    /// Read the pixels of a portion of the sprite.
    ///
    /// # Performance
    ///
    /// Reading pixels will copy a subregion from the image the sprite is a part of, thus it's quite slow.
    ///
    /// When you don't use this function it's recommended to disable the `read-texture` feature flag, which will reduce memory usage of the game.
    ///
    /// # Returns
    ///
    /// - A vector of pixels in RGBA `u32` format, length of the array is width * height of the sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    #[cfg(feature = "read-texture")]
    pub fn read_pixels(self) -> Vec<u32> {
        self.ctx.write(|ctx| {
            // Get the sprite
            let sprite = ctx.sprite(self.path);

            // Get the pixels for the texture of the sprite
            ctx.graphics.atlas.textures[&sprite.texture].clone()
        })
    }

    /// Get the size of the sprite in pixels.
    ///
    /// # Returns
    ///
    /// - `(width, height)`, size of the sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn size(&self) -> (f32, f32) {
        self.ctx.write(|ctx| {
            let sprite = ctx.sprite(self.path);

            (sprite.sub_rectangle.2, sprite.sub_rectangle.3)
        })
    }

    /// Get the width of the sprite in pixels.
    ///
    /// # Returns
    ///
    /// - `width`, horizontal size of the sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.ctx.write(|ctx| ctx.sprite(self.path).sub_rectangle.2)
    }

    /// Get the height of the sprite in pixels.
    ///
    /// # Returns
    ///
    /// - `height`, vertical size of the sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn height(&self) -> f32 {
        self.ctx.write(|ctx| ctx.sprite(self.path).sub_rectangle.3)
    }
}

/// Render methods for sprites.
impl Context {
    /// Handle sprite assets, mostly used for drawing.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    /// Check the [`SpriteContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`Self`] for more information about asset loading and storing.
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to specify the transformations of the sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    #[must_use]
    pub const fn sprite<'path>(&self, path: &'path str) -> SpriteContext<'path, '_> {
        SpriteContext {
            path,
            ctx: self,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
        }
    }
}
