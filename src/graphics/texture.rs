//! Allow types to expose a texture to the GPU.

use glamour::Size2;

/// Value needed to be passed along vertices to identify the texture in the atlas.
pub(crate) type TextureRef = u16;

/// Allow something to upload a texture to the GPU.
pub trait Texture {
    /// Dimensions of the texture.
    fn size(&self) -> Size2<u32>;
}
