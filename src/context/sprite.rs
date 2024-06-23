//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use crate::{assets::Id, Context};

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
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the algorithm passed in [`crate::config::GameConfig::with_rotation_algorithm`].
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
    ///     .draw_multiple_translated((0..10).map(|x| Vector2::new(x as f32, 0.0)));
    /// # }
    /// ```
    ///
    /// It's functionally the same as:
    ///
    /// ```no_run
    /// # use chuot::glamour::Vector2;
    /// # fn call(ctx: chuot::Context) {
    /// for x in 0..10 {
    ///     ctx.sprite("some_asset")
    ///         .translate(Vector2::new(x as f32, 0.0))
    ///         .draw();
    /// }
    /// # }
    /// ```
    ///
    /// But this runs on my PC with an average FPS of 11 when rendering 100000 sprites.
    #[inline(always)]
    pub fn draw_multiple_translated<T>(self, translations: impl Iterator<Item = T>)
    where
        T: Into<(f32, f32)>,
    {
        self.ctx.write(|ctx| {
            let sprite = ctx.sprite(self.path);

            // Push the instances if the texture is already uploaded
            // sprite.draw_multiple(
            //     self.position,
            //     self.rotation,
            //     translations,
            //     &mut ctx.instances,
            // );
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
        pixels: impl Into<Vec<u32>>,
    ) {
        self.ctx.write(|ctx| {
            // Get the sprite
            let sprite = ctx.sprite(self.path);

            // Put the update the pixels of the sprite on a queue
            /*
            ctx.texture_update_queue.push((
                sprite.image.atlas_id,
                sub_rectangle.into(),
                pixels.into(),
            ));
            */
            todo!()
        });
    }

    /// Read the pixels of a portion of the sprite.
    ///
    /// # Performance
    ///
    /// Reading pixels will copy a subregion from the image the sprite is a part of, thus it's quite slow.
    ///
    /// When you don't use this function it's recommended to disable the `read-image` feature flag, which will reduce memory usage of the game.
    ///
    /// # Returns
    ///
    /// - A tuple containing the size of the sprite and a vector of pixels in RGBA u32 format, length of the array is width * height of the sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    #[must_use]
    #[cfg(feature = "read-image")]
    pub fn read_pixels(self) -> (Size2, Vec<u32>) {
        self.ctx.write(|ctx| {
            // Get the sprite
            let sprite = ctx.assets.sprite(self.path);

            // Read the size and the pixels
            (sprite.size(), sprite.pixels())
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
        // self.ctx.write(|ctx| ctx.assets.sprite(self.path).size())

        todo!()
    }

    /// Create a new empty sprite at runtime.
    ///
    /// # Arguments
    ///
    /// * `(width, height)` - Size tuple of the new sprite in pixels.
    ///
    /// # Returns
    ///
    /// - Instance of this context so it can be chained for further operations.
    ///
    /// # Panics
    ///
    /// - When a sprite with the same ID already exists.
    #[inline]
    #[must_use]
    pub fn new(self, size: impl Into<(f32, f32)>) -> Self {
        let id = Id::new(self.path);

        self.ctx.write(|ctx| {});

        self
    }
}

/// Render methods.
///
/// All methods use a `path` as the first argument, which is then used to retrieve the assets when they haven't been loaded before..
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
