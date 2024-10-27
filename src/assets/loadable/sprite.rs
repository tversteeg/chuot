//! Sprite asset.

use glam::{Affine2, Vec2};
use nanoserde::DeRon;
use rgb::RGBA8;

use super::Loadable;
use crate::{
    assets::{
        Id,
        loader::{png::PngLoader, ron::RonLoader},
    },
    context::ContextInner,
    graphics::atlas::TextureRef,
    pivot::Pivot,
};

/// Sprite asset that can be loaded with metadata.
#[derive(Clone, Copy)]
pub(crate) struct Sprite {
    /// Reference to the texture on the GPU.
    pub(crate) texture: TextureRef,
    /// Sub rectangle of the sprite to draw, can be used to split a sprite sheet.
    pub(crate) sub_rectangle: (f32, f32, f32, f32),
    /// Sprite metadata.
    pub(crate) metadata: SpriteMetadata,
}

impl Sprite {
    /// Create and upload a new empty sprite.
    pub(crate) fn new_and_upload(
        width: f32,
        height: f32,
        pivot: Pivot,
        pixels: &[RGBA8],
        ctx: &mut ContextInner,
    ) -> Self {
        // Upload it to the GPU, returning a reference
        let texture =
            ctx.graphics
                .upload_texture(width.round() as u32, height.round() as u32, pixels);

        // Draw the full sprite
        let sub_rectangle = (0.0, 0.0, width, height);

        // Set metadata
        let metadata = SpriteMetadata { pivot };

        Self {
            texture,
            sub_rectangle,
            metadata,
        }
    }

    /// Split into equal horizontal parts.
    pub(crate) fn horizontal_parts(&self, part_width: f32) -> Vec<Self> {
        let (x, y, width, height) = self.sub_rectangle;

        // Ensure that the image can be split into equal parts
        assert!(
            width % part_width == 0.0,
            "Cannot split image into equal horizontal parts of {part_width} pixels"
        );

        // How many images we need to make
        let sub_images = (width / part_width) as usize;

        (0..sub_images)
            .map(|index| {
                // Use the same sub rectangle only changing the position and size
                let sub_rectangle = (part_width.mul_add(index as f32, x), y, part_width, height);

                Self {
                    sub_rectangle,
                    ..*self
                }
            })
            .collect()
    }

    /// Get the sprite offset.
    #[inline]
    #[must_use]
    pub(crate) fn offset_custom_pivot(&self, pivot: Pivot) -> (f32, f32) {
        pivot.pivot(self.sub_rectangle.2, self.sub_rectangle.3)
    }

    /// Get the sprite offset based on the metadata.
    #[inline]
    #[must_use]
    pub(crate) fn offset(&self) -> (f32, f32) {
        self.offset_custom_pivot(self.metadata.pivot)
    }

    /// Calculate the transformation matrix.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn affine_matrix(
        &self,
        x: f32,
        y: f32,
        previous_x: f32,
        previous_y: f32,
        blending_factor: f32,
        blend: bool,
        rotation: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> Affine2 {
        self.affine_matrix_custom_pivot(
            x,
            y,
            previous_x,
            previous_y,
            blending_factor,
            blend,
            rotation,
            scale_x,
            scale_y,
            self.metadata.pivot,
        )
    }

    /// Calculate the transformation using a custom pivot.
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn affine_matrix_custom_pivot(
        &self,
        x: f32,
        y: f32,
        previous_x: f32,
        previous_y: f32,
        blending_factor: f32,
        blend: bool,
        rotation: f32,
        scale_x: f32,
        scale_y: f32,
        pivot: Pivot,
    ) -> Affine2 {
        // Adjust by the sprite offset
        let (sprite_offset_x, sprite_offset_y) =
            pivot.pivot(self.sub_rectangle.2, self.sub_rectangle.3);

        // Apply the blending factor if applicable
        let (x, y) = if blend {
            (
                crate::math::lerp(previous_x, x, blending_factor),
                crate::math::lerp(previous_y, y, blending_factor),
            )
        } else {
            (x, y)
        };

        // Draw with a more optimized version if no rotation and scaling needs to be applied
        #[allow(clippy::float_cmp)]
        if scale_x == 1.0 && scale_y == 1.0 && rotation == 0.0 {
            Affine2::from_translation((x + sprite_offset_x, y + sprite_offset_y).into())
        } else {
            // We rotate so first apply the rotation based on the sprite offset
            let mut affine = Affine2::from_angle(rotation)
                    // Apply scaling
                    * Affine2::from_scale((scale_x, scale_y).into())
                    // Apply the sprite offset so it rotates and scales in place
                    * Affine2::from_translation((sprite_offset_x, sprite_offset_y).into());

            // Then apply the world coordinates so it stays rotated and rotated in place
            affine.translation += Vec2::new(x, y);

            affine
        }
    }

    /// Load the sprite without metadata.
    pub(crate) fn load_if_exists_without_metadata(id: &Id, ctx: &mut ContextInner) -> Option<Self> {
        // Check if there's already an static embedded texture with this ID
        let (texture, width, height) = if let Some(texture) = ctx.asset_source.embedded_texture(id)
        {
            (
                texture.reference,
                texture.width as f32,
                texture.height as f32,
            )
        } else {
            // Load the PNG
            let (width, height, pixels) = ctx.asset_source.load_if_exists::<PngLoader, _>(id)?;

            // Upload it to the GPU, returning a reference
            let texture = ctx.graphics.upload_texture(width, height, &pixels);

            (texture, width as f32, height as f32)
        };

        // Create the sub rectangle from the size
        let sub_rectangle = (0.0, 0.0, width, height);

        // Use default metadata
        let metadata = SpriteMetadata::default();

        Some(Self {
            texture,
            sub_rectangle,
            metadata,
        })
    }
}

impl Loadable for Sprite {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        // Load without metadata
        let mut sprite = Self::load_if_exists_without_metadata(id, ctx)?;

        // Load the metadata, or use the default if it doesn't exit
        sprite.metadata = SpriteMetadata::load_if_exists(id, ctx).unwrap_or_default();

        Some(sprite)
    }
}

/// Sprite metadata to load from data formats.
#[derive(Debug, Clone, Copy, Default, DeRon)]
pub struct SpriteMetadata {
    /// Pixel offset to render at.
    pub(crate) pivot: Pivot,
}

impl Loadable for SpriteMetadata {
    #[inline]
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        ctx.asset_source.load_if_exists::<RonLoader, _>(id)
    }
}
