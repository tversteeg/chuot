//! Sprite asset.

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
    texture: TextureRef,
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

        Some(Self { texture })
    }
}
