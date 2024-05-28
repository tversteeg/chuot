//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use glamour::{Angle, Rect, Size2, Vector2};

use crate::Context;

/// Specify how a sprite should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::sprite`].
pub struct DrawSpriteContext<'path, 'ctx> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Position to draw the sprite at.
    pub(crate) position: Vector2,
    /// Rotation in radians.
    pub(crate) rotation: Angle,
}

impl<'path, 'ctx> DrawSpriteContext<'path, 'ctx> {
    /// Move the position of the sprite.
    ///
    /// # Arguments
    ///
    /// * `position` - Absolute position of the target sprite on the buffer in pixels, will be offset by the sprite offset metadata.
    #[inline(always)]
    #[must_use]
    pub fn translate(mut self, offset: impl Into<Vector2>) -> Self {
        self.position += offset.into();

        self
    }

    /// Rotate the sprite.
    ///
    /// Rotation will always be applied before translation, this mean it will always rotate around the center point specified in the sprite offset metadata.
    ///
    /// # Arguments
    ///
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the algorithm passed in [`crate::config::GameConfig::with_rotation_algorithm`].
    #[inline(always)]
    #[must_use]
    pub fn rotate(mut self, rotation: impl Into<Angle>) -> Self {
        self.rotation += rotation.into();

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
            ctx.assets
                .sprite(self.path)
                .draw(self.position, self.rotation, &mut ctx.instances);
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
    /// * `translations` - Iterator of translation offsets, will draw each item as a sprite at the base position with the offset applied.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    ///
    /// # Example
    ///
    /// This example runs on my PC with an average FPS of 35 when rendering 100000 sprites.
    ///
    /// ```no_run
    /// # use chuot::glamour::Vector2;
    /// # fn call(ctx: chuot::Context) {
    /// ctx.sprite("some_asset")
    ///   .draw_multiple_translated((0..10).map(|x| Vector2::new(x as f32, 0.0)));
    /// # }
    /// ```
    ///
    /// It's functionally the same as:
    ///
    /// ```no_run
    /// # use chuot::glamour::Vector2;
    /// # fn call(ctx: chuot::Context) {
    /// for x in 0..10 {
    ///   ctx.sprite("some_asset")
    ///     .translate(Vector2::new(x as f32, 0.0))
    ///     .draw();
    /// }
    /// # }
    /// ```
    ///
    /// But this runs on my PC with an average FPS of 11 when rendering 100000 sprites.
    #[inline(always)]
    pub fn draw_multiple_translated(self, translations: impl Iterator<Item = Vector2>) {
        self.ctx.write(|ctx| {
            let sprite = ctx.assets.sprite(self.path);

            // Push the instances if the texture is already uploaded
            sprite.draw_multiple(
                self.position,
                self.rotation,
                translations,
                &mut ctx.instances,
            );
        });
    }

    /// Update the pixels of a portion of the sprite.
    ///
    /// # Arguments
    ///
    /// * `sub_rectangle` - Sub rectangle within the sprite to update. Width * height must be equal to the amount of pixels, and fall within the sprite's rectangle.
    /// * `pixels` - Array of ARGB pixels, amount must match size of the sub rectangle.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When the sub rectangle does not fit inside the sprite's rectangle.
    /// - When the size of the sub rectangle does not match the amount of pixels.
    #[inline]
    pub fn update_pixels(self, sub_rectangle: impl Into<Rect>, pixels: impl Into<Vec<u32>>) {
        self.ctx.write(|ctx| {
            // Get the sprite
            let sprite = ctx.assets.sprite(self.path);

            // Put the update the pixels of the sprite on a queue
            ctx.texture_update_queue.push((
                sprite.image.atlas_id,
                sub_rectangle.into(),
                pixels.into(),
            ));
        });
    }

    /// Get the size of the sprite in pixels.
    ///
    /// # Returns
    ///
    /// - Size of the sprite in pixels.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    pub fn size(&self) -> Size2 {
        self.ctx.write(|ctx| ctx.assets.sprite(self.path).size())
    }
}
