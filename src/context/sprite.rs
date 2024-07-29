//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use std::{marker::PhantomData, rc::Rc};

use glam::Affine2;
use rgb::RGBA8;

use crate::{
    assets::{loadable::sprite::Sprite, Id},
    Camera, Context, Draw, Rotate, Scale, Translate, TranslatePrevious,
};

use super::{
    extensions::{
        camera::{IsUiCamera, MainCamera, UiCamera},
        rotate::Rotation,
        scale::Scaling,
        translate::{PreviousTranslation, Translation},
        Empty,
    },
    ContextInner,
};

/// With translation, ensures the type is used properly.
type WithTranslation<'path, 'ctx, P, R, S, C> = SpriteContext<'path, 'ctx, Translation, P, R, S, C>;
/// Without translation, ensures the type is used properly.
type WithoutTranslation<'path, 'ctx, P, R, S, C> = SpriteContext<'path, 'ctx, Empty, P, R, S, C>;
/// With previous translation, ensures the type is used properly.
type WithPreviousTranslation<'path, 'ctx, T, R, S, C> =
    SpriteContext<'path, 'ctx, T, PreviousTranslation, R, S, C>;
/// Without previous translation, ensures the type is used properly.
type WithoutPreviousTranslation<'path, 'ctx, T, R, S, C> =
    SpriteContext<'path, 'ctx, T, Empty, R, S, C>;
/// With rotation, ensures the type is used properly.
type WithRotation<'path, 'ctx, T, P, S, C> = SpriteContext<'path, 'ctx, T, P, Rotation, S, C>;
/// Without previous translation, ensures the type is used properly.
type WithoutRotation<'path, 'ctx, T, P, S, C> = SpriteContext<'path, 'ctx, T, P, Empty, S, C>;
/// With rotation, ensures the type is used properly.
type WithScaling<'path, 'ctx, T, P, R, C> = SpriteContext<'path, 'ctx, T, P, R, Scaling, C>;
/// Without previous translation, ensures the type is used properly.
type WithoutScaling<'path, 'ctx, T, P, R, C> = SpriteContext<'path, 'ctx, T, P, R, Empty, C>;

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

impl<'path, 'ctx, T, P, R, S, C: IsUiCamera> SpriteContext<'path, 'ctx, T, P, R, S, C> {
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
    pub fn draw_multiple_translated<I>(self, translations: impl Iterator<Item = I>)
    where
        I: Into<(f32, f32)>,
    {
        todo!()
        /*
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprites with
            let camera = ctx.camera_mut(self.ui_camera);
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                self.x + offset_x,
                self.y + offset_y,
                self.previous_x + offset_x,
                self.previous_y + offset_y,
                ctx.blending_factor,
                self.blend,
                self.rotation,
                self.scale_x,
                self.scale_y,
            );

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
        */
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

    /// Draw a single item.
    fn draw_single(
        self,
        DrawSingleArgs {
            translation,
            previous_translation,
            rotation,
            scale,
            blend,
        }: DrawSingleArgs,
    ) {
        self.ctx.write(|ctx| {
            // Push the instance if the texture is already uploaded
            let sprite = ctx.sprite(self.path);

            // Get the camera to draw the sprite with
            let camera = ctx.camera_mut(false);
            let offset_x = camera.offset_x();
            let offset_y = camera.offset_y();

            // Create the affine matrix
            let affine_matrix = sprite.affine_matrix(
                translation.x + offset_x,
                translation.y + offset_y,
                previous_translation.previous_x + offset_x,
                previous_translation.previous_y + offset_y,
                ctx.blending_factor,
                blend,
                rotation.0,
                scale.scale_x,
                scale.scale_y,
            );

            // Push the graphics
            ctx.graphics
                .instances
                .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
        });
    }
}

/// No translation yet.
impl<'path, 'ctx, P, R, S, C> Translate for WithoutTranslation<'path, 'ctx, P, R, S, C> {
    type Into = WithTranslation<'path, 'ctx, P, R, S, C>;

    #[inline]
    fn inner_translate(self, translation: (f32, f32)) -> Self::Into {
        Self::Into {
            path: self.path,
            ctx: self.ctx,
            translation: Translation::new(translation),
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }
}

/// Already has translation.
impl<'path, 'ctx, P, R, S, C> Translate for WithTranslation<'path, 'ctx, P, R, S, C> {
    type Into = Self;

    #[inline]
    fn inner_translate(mut self, translation: (f32, f32)) -> Self::Into {
        self.translation = self.translation.inner_translate(translation);

        self
    }
}

/// No previous translation yet.
impl<'path, 'ctx, T, R, S, C> TranslatePrevious
    for WithoutPreviousTranslation<'path, 'ctx, T, R, S, C>
{
    type Into = WithPreviousTranslation<'path, 'ctx, T, R, S, C>;

    #[inline]
    fn inner_translate_previous(self, translation: (f32, f32)) -> Self::Into {
        Self::Into {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: PreviousTranslation::new(translation),
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }
}

/// Already has previous translation.
impl<'path, 'ctx, T, R, S, C> TranslatePrevious
    for WithPreviousTranslation<'path, 'ctx, T, R, S, C>
{
    type Into = Self;

    #[inline]
    fn inner_translate_previous(mut self, previous_translation: (f32, f32)) -> Self::Into {
        self.previous_translation = self
            .previous_translation
            .inner_translate_previous(previous_translation);

        self
    }
}

/// No rotation yet.
impl<'path, 'ctx, T, P, S, C> Rotate for WithoutRotation<'path, 'ctx, T, P, S, C> {
    type Into = WithRotation<'path, 'ctx, T, P, S, C>;

    #[inline]
    fn rotate(self, rotation: f32) -> Self::Into {
        Self::Into {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: Rotation(rotation),
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }
}

/// Already has rotation.
impl<'path, 'ctx, T, P, S, C> Rotate for WithRotation<'path, 'ctx, T, P, S, C> {
    type Into = Self;

    #[inline]
    fn rotate(mut self, rotation: f32) -> Self::Into {
        self.rotation = self.rotation.rotate(rotation);

        self
    }
}

/// No translation yet.
impl<'path, 'ctx, T, P, R, C> Scale for WithoutScaling<'path, 'ctx, T, P, R, C> {
    type Into = WithScaling<'path, 'ctx, T, P, R, C>;

    #[inline]
    fn inner_scale(self, scaling: (f32, f32)) -> Self::Into {
        Self::Into {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: Scaling::new(scaling),
            phantom: PhantomData,
        }
    }
}

/// Already has translation.
impl<'path, 'ctx, T, P, R, C> Scale for WithScaling<'path, 'ctx, T, P, R, C> {
    type Into = Self;

    #[inline]
    fn inner_scale(mut self, scaling: (f32, f32)) -> Self::Into {
        self.scaling = self.scaling.inner_scale(scaling);

        self
    }
}

/// Select the camera.
impl<'path, 'ctx, T, P, R, S, C> Camera for SpriteContext<'path, 'ctx, T, P, R, S, C> {
    type IntoUi = SpriteContext<'path, 'ctx, T, P, R, S, UiCamera>;
    type IntoMain = SpriteContext<'path, 'ctx, T, P, R, S, MainCamera>;

    #[inline]
    fn use_ui_camera(self) -> Self::IntoUi {
        Self::IntoUi {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn use_main_camera(self) -> Self::IntoMain {
        Self::IntoMain {
            path: self.path,
            ctx: self.ctx,
            translation: self.translation,
            previous_translation: self.previous_translation,
            rotation: self.rotation,
            scaling: self.scaling,
            phantom: PhantomData,
        }
    }
}

/// Nothing.
impl<'path, 'ctx, C: IsUiCamera> Draw
    for SpriteContext<'path, 'ctx, Empty, Empty, Empty, Empty, C>
{
    #[inline]
    fn draw(self) {
        self.ctx.write(|ctx| {
            let (sprite, affine_matrix) =
                ctx.sprite_with_base_affine_matrix(self.path, C::is_ui_camera());

            // Push the graphics
            ctx.graphics
                .instances
                .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
        });
    }
}

/// Only translation.
impl<'path, 'ctx, C: IsUiCamera> Draw
    for SpriteContext<'path, 'ctx, Translation, Empty, Empty, Empty, C>
{
    #[inline]
    fn draw(self) {
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
}

/// Only rotation.
impl<'path, 'ctx, C: IsUiCamera> Draw
    for SpriteContext<'path, 'ctx, Empty, Empty, Rotation, Empty, C>
{
    #[inline]
    fn draw(self) {
        let rotation = self.rotation;

        // TODO: optimize using affine matrix base
        self.draw_single(DrawSingleArgs {
            rotation,
            ..Default::default()
        });
    }
}

/// Translation and rotation.
impl<'path, 'ctx, C: IsUiCamera> Draw
    for SpriteContext<'path, 'ctx, Translation, Empty, Rotation, Empty, C>
{
    #[inline]
    fn draw(self) {
        let translation = self.translation;
        let rotation = self.rotation;

        // TODO: optimize using affine matrix base
        self.draw_single(DrawSingleArgs {
            translation,
            rotation,
            ..Default::default()
        });
    }
}

/*

impl<'path, 'ctx> Draw for WithTranslation<SpriteContext<'path, 'ctx>, WithTranslationPrevious> {
    #[inline]
    fn draw(self) {
        self.base.ctx.write(|ctx| {
            let (sprite, mut affine_matrix) =
                ctx.sprite_with_base_affine_matrix(self.base.path, false);

            // Translate the coordinates
            affine_matrix.translation.x +=
                crate::math::lerp(self.previous.previous_x, self.x, ctx.blending_factor);
            affine_matrix.translation.y +=
                crate::math::lerp(self.previous.previous_y, self.y, ctx.blending_factor);

            // Push the graphics
            ctx.graphics
                .instances
                .push(affine_matrix, sprite.sub_rectangle, sprite.texture);
        });
    }
}

impl<'path, 'ctx> Draw for WithRotation<SpriteContext<'path, 'ctx>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.draw_single(DrawSingleArgs {
            rotation: self.rotation,
            ..Default::default()
        });
    }
}

impl<'path, 'ctx> Draw for WithRotation<WithTranslation<SpriteContext<'path, 'ctx>>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.base.draw_single(DrawSingleArgs {
            x: self.base.x,
            y: self.base.y,
            rotation: self.rotation,
            ..Default::default()
        });
    }
}

impl<'path, 'ctx> Draw for WithScale<SpriteContext<'path, 'ctx>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.draw_single(DrawSingleArgs {
            scale_x: self.scale_x,
            scale_y: self.scale_y,
            ..Default::default()
        });
    }
}

impl<'path, 'ctx> Draw for WithScale<WithTranslation<SpriteContext<'path, 'ctx>>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.base.draw_single(DrawSingleArgs {
            x: self.base.x,
            y: self.base.y,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
            ..Default::default()
        });
    }
}

impl<'path, 'ctx> Draw for WithScale<WithRotation<SpriteContext<'path, 'ctx>>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.base.draw_single(DrawSingleArgs {
            rotation: self.base.rotation,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
            ..Default::default()
        });
    }
}

impl<'path, 'ctx> Draw for WithScale<WithRotation<WithTranslation<SpriteContext<'path, 'ctx>>>> {
    #[inline]
    fn draw(self) {
        // TODO: optimize using affine matrix base
        self.base.base.base.draw_single(DrawSingleArgs {
            x: self.base.base.x,
            y: self.base.base.y,
            rotation: self.base.rotation,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
            ..Default::default()
        });
    }
}
*/

/// Arguments for drawing a single item.
#[derive(Default)]
struct DrawSingleArgs {
    translation: Translation,
    previous_translation: PreviousTranslation,
    rotation: Rotation,
    scale: Scaling,
    blend: bool,
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
