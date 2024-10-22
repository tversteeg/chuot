//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use std::{convert::Into, marker::PhantomData, rc::Rc};

use glam::Affine2;
use rgb::RGBA8;

use super::{
    ContextInner,
    extensions::{
        Empty,
        camera::{IsUiCamera, MainCamera, UiCamera},
        rotate::{Rotate, Rotation},
        scale::{Scale, Scaling},
        translate::{PreviousTranslation, Translate, TranslatePrevious, Translation},
    },
};
use crate::{
    Context,
    assets::{Id, loadable::sprite::Sprite},
};

/// Specify how a sprite should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::sprite`].
pub struct SpriteContext<'path, 'ctx, T, P, R, S, C> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Possible translation implementation, determined by type.
    pub(crate) translation: T,
    /// Possible previous translation implementation, determined by type.
    pub(crate) previous_translation: P,
    /// Possible rotation implementation, determined by type.
    pub(crate) rotation: R,
    /// Possible scaling implementation, determined by type.
    pub(crate) scaling: S,
    /// Generic types without any concrete fields.
    pub(crate) phantom: PhantomData<C>,
}

impl<'path, 'ctx, T: Translate, P: TranslatePrevious, R: Rotate, S: Scale, C: IsUiCamera>
    SpriteContext<'path, 'ctx, T, P, R, S, C>
{
    /// Only move the horizontal position.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    pub fn translate_x(self, x: f32) -> SpriteContext<'path, 'ctx, Translation, P, R, S, C> {
        self.translate_impl((x, 0.0))
    }

    /// Only move the vertical position.
    ///
    /// # Arguments
    ///
    /// * `y` - Vertical position on the buffer in pixels.
    #[inline(always)]
    #[must_use]
    pub fn translate_y(self, y: f32) -> SpriteContext<'path, 'ctx, Translation, P, R, S, C> {
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
    ) -> SpriteContext<'path, 'ctx, Translation, P, R, S, C> {
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
    ) -> SpriteContext<'path, 'ctx, T, PreviousTranslation, R, S, C> {
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
    ) -> SpriteContext<'path, 'ctx, T, PreviousTranslation, R, S, C> {
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
    ) -> SpriteContext<'path, 'ctx, T, PreviousTranslation, R, S, C> {
        self.translate_previous_impl(previous_position.into())
    }

    /// Only scale the horizontal size.
    ///
    /// # Arguments
    ///
    /// * `scale_x` - Horizontal scale on the buffer. `-1.0` to flip.
    #[inline(always)]
    #[must_use]
    pub fn scale_x(self, scale_x: f32) -> SpriteContext<'path, 'ctx, T, P, R, Scaling, C> {
        self.scale_impl((scale_x, 0.0))
    }

    /// Only move the vertical position.
    ///
    /// # Arguments
    ///
    /// * `scale_y` - Vertical scale on the buffer. `-1.0` to flip.
    #[inline(always)]
    #[must_use]
    pub fn scale_y(self, scale_y: f32) -> SpriteContext<'path, 'ctx, T, P, R, Scaling, C> {
        self.scale_impl((0.0, scale_y))
    }

    /// Move the position.
    ///
    /// # Arguments
    ///
    /// * `(scale_x, scale_y)` - Scale tuple on the buffer.
    #[inline]
    #[must_use]
    pub fn scale(
        self,
        scale: impl Into<(f32, f32)>,
    ) -> SpriteContext<'path, 'ctx, T, P, R, Scaling, C> {
        self.scale_impl(scale.into())
    }

    /// Rotate.
    ///
    /// Rotation will always be applied before translation, this mean it will always rotate around the center point specified in the sprite offset metadata.
    ///
    /// # Arguments
    ///
    /// * `rotation` - Rotation in radians, will be applied using the algorithm passed in [`crate::config::Config::with_rotation_algorithm`].
    #[inline]
    #[must_use]
    pub fn rotate(self, rotation: f32) -> SpriteContext<'path, 'ctx, T, P, Rotation, S, C> {
        let rotation = self.rotation.inner_rotate(rotation);

        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            rotation,
            scaling: self.scaling,
            previous_translation: self.previous_translation,
            phantom: PhantomData,
        }
    }

    /// Use the UI camera instead of the regular game camera for transforming the drawable object.
    #[inline]
    #[must_use]
    pub fn use_ui_camera(self) -> SpriteContext<'path, 'ctx, T, P, R, S, UiCamera> {
        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }

    /// Use the regular game camera instead of the UI camera for transforming the drawable object.
    #[inline]
    #[must_use]
    pub fn use_main_camera(self) -> SpriteContext<'path, 'ctx, T, P, R, S, MainCamera> {
        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
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
    pub fn create(self, size: impl Into<(f32, f32)>, pixels: impl AsRef<[RGBA8]>) {
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
        pixels: impl AsRef<[RGBA8]>,
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
    pub fn read_pixels(self) -> Vec<RGBA8> {
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

    /// Perform the translation with the type.
    #[inline]
    #[must_use]
    fn translate_impl(
        self,
        position: (f32, f32),
    ) -> SpriteContext<'path, 'ctx, Translation, P, R, S, C> {
        let translation = self.translation.inner_translate(position);

        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }

    /// Perform the previous translation with the type.
    #[inline]
    #[must_use]
    fn translate_previous_impl(
        self,
        previous_position: (f32, f32),
    ) -> SpriteContext<'path, 'ctx, T, PreviousTranslation, R, S, C> {
        let previous_translation = self
            .previous_translation
            .inner_translate_previous(previous_position);

        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            rotation: self.rotation,
            scaling: self.scaling,
            previous_translation,
            phantom: PhantomData,
        }
    }

    /// Perform the translation with the type.
    #[inline]
    #[must_use]
    fn scale_impl(self, scale: (f32, f32)) -> SpriteContext<'path, 'ctx, T, P, R, Scaling, C> {
        let scaling = self.scaling.inner_scale(scale);

        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling,
            phantom: PhantomData,
        }
    }

    /// Convert a generic type to a fully formed type.
    ///
    /// This has sub-optimal performance for drawing since it crosses all paths.
    #[inline]
    #[must_use]
    fn into_full_with_previous_translation(
        self,
    ) -> SpriteContext<'path, 'ctx, Translation, PreviousTranslation, Rotation, Scaling, C> {
        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation.inner_translate((0.0, 0.0)),
            previous_translation: self
                .previous_translation
                .inner_translate_previous((0.0, 0.0)),
            rotation: self.rotation.inner_rotate(0.0),
            scaling: self.scaling.inner_scale((1.0, 1.0)),
            phantom: PhantomData,
        }
    }

    /// Convert a generic type to a fully formed type without a previous translation.
    ///
    /// This has sub-optimal performance for drawing since it crosses all paths.
    #[inline]
    #[must_use]
    fn into_full_without_previous_translation(
        self,
    ) -> SpriteContext<'path, 'ctx, Translation, Empty, Rotation, Scaling, C> {
        SpriteContext {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation.inner_translate((0.0, 0.0)),
            previous_translation: Empty,
            rotation: self.rotation.inner_rotate(0.0),
            scaling: self.scaling.inner_scale((1.0, 1.0)),
            phantom: PhantomData,
        }
    }
}

/// Nothing.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Empty, Empty, Empty, Empty, C> {
    /// Draw the sprite to the screen at the zero coordinate of the camera.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            let (sprite, affine_matrix) =
                ctx.sprite_with_base_affine_matrix(self.path, C::is_ui_camera());

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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Only translation.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Translation, Empty, Empty, Empty, C> {
    /// Draw the sprite to the screen.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            let (sprite, mut affine_matrix) =
                ctx.sprite_with_base_affine_matrix(self.path, C::is_ui_camera());

            // Translate the coordinates
            affine_matrix.translation.x += self.translation.x;
            affine_matrix.translation.y += self.translation.y;

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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation and previous translation.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, PreviousTranslation, Empty, Empty, C>
{
    /// Draw the sprite to the screen, interpolating the position in the render step.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            let (sprite, mut affine_matrix) =
                ctx.sprite_with_base_affine_matrix(self.path, C::is_ui_camera());

            // Translate the coordinates
            affine_matrix.translation.x += crate::math::lerp(
                self.previous_translation.previous_x,
                self.translation.x,
                ctx.blending_factor,
            );
            affine_matrix.translation.y += crate::math::lerp(
                self.previous_translation.previous_y,
                self.translation.y,
                ctx.blending_factor,
            );

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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_with_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Only rotation.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Empty, Empty, Rotation, Empty, C> {
    /// Draw the sprite rotated to the screen at the zero coordinate of the camera.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Only scaling.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Empty, Empty, Empty, Scaling, C> {
    /// Draw the sprite scaled to the screen at the zero coordinate of the camera.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation and rotation.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, Empty, Rotation, Empty, C>
{
    /// Draw the sprite rotated to the screen.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation, previous translation and rotation.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, PreviousTranslation, Rotation, Empty, C>
{
    /// Draw the sprite rotated to the screen, interpolating in the render step.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_with_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_with_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation and scaling.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Translation, Empty, Empty, Scaling, C> {
    /// Draw the sprite scaled to the screen.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation, previous translation and scaling.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, PreviousTranslation, Empty, Scaling, C>
{
    /// Draw the sprite scaled to the screen, interpolating in the render step.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_with_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_with_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Rotation and scaling.
impl<'path, 'ctx, C: IsUiCamera> SpriteContext<'path, 'ctx, Empty, Empty, Rotation, Scaling, C> {
    /// Draw the sprite rotated and scaled to the screen at the zero coordinate of the camera.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation().draw();
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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        // TODO: optimize using affine matrix base
        self.into_full_without_previous_translation()
            .draw_multiple_translated(translations);
    }
}

/// Translation, rotation and scaling.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, Empty, Rotation, Scaling, C>
{
    /// Draw the sprite rotated and scaled to the screen.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                0.0,
                0.0,
                0.0,
                false,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
            );

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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                0.0,
                0.0,
                0.0,
                false,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
            );

            // Push the graphics
            ctx.graphics
                .instances
                .extend(translations.map(Into::into).map(|(x_offset, y_offset)| {
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
}

/// Translation, previous translation, rotation and scaling.
impl<'path, 'ctx, C: IsUiCamera>
    SpriteContext<'path, 'ctx, Translation, PreviousTranslation, Rotation, Scaling, C>
{
    /// Draw the sprite rotated and scaled to the screen, interpolating the position in the render step.
    ///
    /// Sprites that are drawn last are always shown on top of sprites that are drawn earlier.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                self.previous_translation.previous_x + offset_x,
                self.previous_translation.previous_y + offset_y,
                ctx.blending_factor,
                true,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
            );

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
    #[inline]
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprite with
            let camera = ctx.camera(C::is_ui_camera());
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                self.previous_translation.previous_x + offset_x,
                self.previous_translation.previous_y + offset_y,
                ctx.blending_factor,
                true,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
            );

            // Push the graphics
            ctx.graphics
                .instances
                .extend(translations.map(Into::into).map(|(x_offset, y_offset)| {
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
    pub const fn sprite<'path>(
        &self,
        path: &'path str,
    ) -> SpriteContext<'path, '_, Empty, Empty, Empty, Empty, MainCamera> {
        SpriteContext {
            path,
            ctx: self,
            translation: Empty,
            previous_translation: Empty,
            rotation: Empty,
            scaling: Empty,
            phantom: PhantomData,
        }
    }
}

/// Helper functions to reduce code duplication.
impl ContextInner {
    /// Get the sprite with it's base offset calculated from the camera and its internal offset.
    #[inline]
    fn sprite_with_base_affine_matrix(
        &mut self,
        path: &str,
        is_ui_camera: bool,
    ) -> (Rc<Sprite>, Affine2) {
        let sprite = self.sprite(path);

        // Get the sprite offset
        let (mut sprite_x, mut sprite_y) = sprite.offset();

        // Offset the sprite with the camera
        let camera = self.camera(is_ui_camera);
        sprite_x += camera.offset_x();
        sprite_y += camera.offset_y();

        // Create the affine matrix
        let affine_matrix = Affine2::from_translation((sprite_x, sprite_y).into());

        (sprite, affine_matrix)
    }
}
