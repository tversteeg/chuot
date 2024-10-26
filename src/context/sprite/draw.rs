//! Different implementations for drawing a sprite.

use super::SpriteContext;
use crate::context::extensions::{
    Empty,
    camera::IsUiCamera,
    pivot::Pivoting,
    rotate::Rotation,
    scale::Scaling,
    translate::{PreviousTranslation, Translation},
};

/// Nothing.
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Empty, Empty, Empty, C> {
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Empty, Empty, Empty, C> {
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
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Empty, Empty, Empty, C>
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Rotation, Empty, Empty, C> {
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Empty, Scaling, Empty, C> {
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Rotation, Empty, Empty, C> {
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
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Rotation, Empty, Empty, C>
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Empty, Scaling, Empty, C> {
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
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Empty, Scaling, Empty, C>
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Rotation, Scaling, Empty, C> {
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
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Rotation, Scaling, Empty, C> {
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
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Rotation, Scaling, Empty, C>
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

/// Pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Empty, Empty, Pivoting, C> {
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
            let (sprite, affine_matrix) = ctx.sprite_with_base_affine_matrix_custom_pivot(
                self.path,
                C::is_ui_camera(),
                self.pivot.value(),
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Empty, Empty, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, previous translation and pivoting.
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Empty, Empty, Pivoting, C>
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
        self.into_full_with_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Rotation and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Rotation, Empty, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw();
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Scaling and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Empty, Scaling, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw();
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, rotation and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Rotation, Empty, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw();
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, previous translation, rotation and pivoting.
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Rotation, Empty, Pivoting, C>
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
        self.into_full_with_previous_translation_and_pivot().draw();
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
        self.into_full_with_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, scaling and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Empty, Scaling, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw();
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, previous translation, scaling and pivoting.
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Empty, Scaling, Pivoting, C>
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
        self.into_full_with_previous_translation_and_pivot().draw();
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
        self.into_full_with_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Rotation, scaling and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Empty, Empty, Rotation, Scaling, Pivoting, C> {
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
        self.into_full_without_previous_translation_and_pivot()
            .draw();
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
        self.into_full_without_previous_translation_and_pivot()
            .draw_multiple_translated(translations);
    }
}

/// Translation, rotation, scaling and pivoting.
impl<C: IsUiCamera> SpriteContext<'_, '_, Translation, Empty, Rotation, Scaling, Pivoting, C> {
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
            let affine_matrix = sprite.affine_matrix_custom_pivot(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                0.0,
                0.0,
                0.0,
                false,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
                self.pivot.value(),
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
            let affine_matrix = sprite.affine_matrix_custom_pivot(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                0.0,
                0.0,
                0.0,
                false,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
                self.pivot.value(),
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

/// Translation, previous translation, rotation, scaling and pivoting.
impl<C: IsUiCamera>
    SpriteContext<'_, '_, Translation, PreviousTranslation, Rotation, Scaling, Pivoting, C>
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
            let affine_matrix = sprite.affine_matrix_custom_pivot(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                self.previous_translation.previous_x + offset_x,
                self.previous_translation.previous_y + offset_y,
                ctx.blending_factor,
                true,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
                self.pivot.value(),
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
            let affine_matrix = sprite.affine_matrix_custom_pivot(
                self.translation.x + offset_x,
                self.translation.y + offset_y,
                self.previous_translation.previous_x + offset_x,
                self.previous_translation.previous_y + offset_y,
                ctx.blending_factor,
                true,
                self.rotation.value(),
                self.scaling.scale_x,
                self.scaling.scale_y,
                self.pivot.value(),
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
