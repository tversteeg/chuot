//! Sprite asset.

use glam::{Affine2, Vec2};
use nanoserde::DeRon;

use crate::{
    assets::{
        loader::{png::PngLoader, ron::RonLoader},
        source::AssetSource,
        Id,
    },
    context::ContextInner,
    graphics::atlas::TextureRef,
};

use super::Loadable;

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
        pixels: &[u32],
        ctx: &mut ContextInner,
    ) -> Self {
        // Upload it to the GPU, returning a reference
        let texture =
            ctx.graphics
                .upload_texture(width.round() as u32, height.round() as u32, pixels);

        // Draw the full sprite
        let sub_rectangle = (0.0, 0.0, width, height);

        // Use default metadata
        let metadata = SpriteMetadata::default();

        Self {
            texture,
            sub_rectangle,
            metadata,
        }
    }

    /// Calculate the transformation matrix.
    #[inline]
    pub(crate) fn affine_matrix(&self, x: f32, y: f32, rotation: f32) -> Affine2 {
        // Adjust by the sprite offset
        let (sprite_offset_x, sprite_offset_y) = self
            .metadata
            .offset
            .offset(self.sub_rectangle.2, self.sub_rectangle.3);

        // Draw with a more optimized version if no rotation needs to be applied
        if rotation == 0.0 {
            Affine2::from_translation((x + sprite_offset_x, y + sprite_offset_y).into())
        } else {
            // We rotate so first apply the rotation based on the sprite offset
            let mut affine = Affine2::from_angle(rotation)
                * Affine2::from_translation((sprite_offset_x, sprite_offset_y).into());

            // Then apply the world coordinates so it stays rotated in place
            affine.translation += Vec2::new(x, y);

            affine
        }
    }
}

impl Loadable for Sprite {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        // Load the PNG
        let mut png = ctx.asset_source.load_if_exists::<PngLoader, _>(id)?;

        // Read the PNG
        let mut pixels = vec![0_u32; png.output_buffer_size()];
        let info = png
            .next_frame(bytemuck::cast_slice_mut(&mut pixels))
            .expect("Error reading image");

        // Upload it to the GPU, returning a reference
        let texture = ctx
            .graphics
            .upload_texture(info.width, info.height, &pixels);

        // Create the sub rectangle from the size
        let sub_rectangle = (0.0, 0.0, info.width as f32, info.height as f32);

        // Load the metadata, or use the default if it doesn't exit
        let metadata = SpriteMetadata::load_if_exists(id, ctx).unwrap_or_default();

        Some(Self {
            texture,
            sub_rectangle,
            metadata,
        })
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, DeRon)]
pub enum SpriteOffset {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    #[default]
    LeftTop,
    /// Sprite will be offset with the custom coordinates counting from the left top.
    Custom {
        /// X offset from the left.
        x: f32,
        /// Y offset from the top.
        y: f32,
    },
}

impl SpriteOffset {
    /// Get the offset based on the sprite size.
    #[inline]
    pub(crate) fn offset(&self, sprite_width: f32, sprite_height: f32) -> (f32, f32) {
        match self {
            Self::Middle => (-sprite_width / 2.0, -sprite_height / 2.0),
            Self::MiddleTop => (-sprite_width / 2.0, 0.0),
            Self::LeftTop => (0.0, 0.0),
            Self::Custom { x, y } => (-x, -y),
        }
    }
}

/// Sprite metadata to load from data formats.
#[derive(Debug, Clone, Copy, Default, DeRon)]
pub struct SpriteMetadata {
    /// Pixel offset to render at.
    pub(crate) offset: SpriteOffset,
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
