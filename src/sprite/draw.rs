//! Zero-cost abstraction types for building more complicated sprite drawing constructions.

use glamour::{Angle, Vector2};

use crate::Context;

/// Specify how a sprite should be drawn.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`crate::Context::draw_sprite`].
pub struct DrawSpriteContext<'path, 'ctx> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Position to draw the sprite at.
    pub(crate) position: Vector2,
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
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the RotSprite algorithm.
    #[inline(always)]
    #[must_use]
    pub fn rotate(self, rotation: impl Into<Angle>) -> DrawSpriteContextRotated<'path, 'ctx> {
        DrawSpriteContextRotated {
            path: self.path,
            ctx: self.ctx,
            position: self.position,
            rotation: rotation.into(),
        }
    }

    /// Draw the sprite.
    ///
    /// It's not necessary to call this function since [`std::ops::Drop`] is also implemented for this type.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            ctx.load_sprite_if_not_loaded(self.path);

            let sprite = ctx
                .sprites
                .get_mut(self.path)
                .expect("Error accessing sprite in context");

            // Push the instance if the texture is already uploaded
            sprite.draw(self.position, &mut ctx.instances);
        });
    }
}

/// Specify how a sprite should be drawn with a rotation.
///
/// Must call [`Self::draw`] to finish drawing.
///
/// Used by [`DrawSpriteContext`].
///
/// This type is an optimization.
pub struct DrawSpriteContextRotated<'path, 'ctx> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Position to draw the sprite at.
    pub(crate) position: Vector2,
    /// Rotation in radians to draw the sprite at.
    pub(crate) rotation: Angle,
}

impl<'path, 'ctx> DrawSpriteContextRotated<'path, 'ctx> {
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
    /// # Arguments
    ///
    /// * `rotation` - Rotation of the target sprite in radians, will be applied using the RotSprite algorithm.
    #[inline(always)]
    #[must_use]
    pub fn rotate(mut self, rotation: impl Into<Angle>) -> Self {
        self.rotation += rotation.into();

        self
    }

    /// Draw the sprite.
    ///
    /// It's not necessary to call this function since [`std::ops::Drop`] is also implemented for this type.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn draw(self) {
        self.ctx.write(|ctx| {
            ctx.load_sprite_if_not_loaded(self.path);

            let sprite = ctx
                .sprites
                .get_mut(self.path)
                .expect("Error accessing sprite in context");

            // Push the instance if the texture is already uploaded
            sprite.draw_rotated(self.position, self.rotation, &mut ctx.instances);
        });
    }
}
