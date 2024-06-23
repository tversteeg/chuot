//! Sprite asset.

use glam::Affine2;

use crate::{
    assets::{loader::png::PngLoader, source::AssetSource, Id},
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
}

impl Sprite {
    /// Calculate the transformation matrix.
    pub(crate) fn affine_matrix(&self, x: f32, y: f32, rotation: f32) -> Affine2 {
        // Draw with a more optimized version if no rotation needs to be applied
        if rotation == 0.0 {
            Affine2::from_translation((x, y).into())
        } else {
            Affine2::from_angle_translation(rotation, (x, y).into())
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

        Some(Self {
            texture,
            sub_rectangle,
        })
    }
}
