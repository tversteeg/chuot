//! Allow types to expose a texture to the GPU.

use std::{
    borrow::Cow,
    sync::{Arc, Mutex, OnceLock},
};

use bytemuck::{Pod, Zeroable};
use glamour::Size2;

use super::atlas::Atlas;

/// Value needed to be passed along vertices to identify the texture in the atlas.
pub type TextureRef = u16;

/// Allow something to upload a texture to the GPU.
pub trait Texture {
    /// Dimensions of the texture.
    fn size(&self) -> Size2<u32>;

    /// Image representation we can upload to the GPU.
    fn to_rgba_image(&mut self) -> Vec<u8>;
}

/// Texture holder that can be in a non-uploaded state.
pub enum PendingOrUploaded<T: Texture> {
    /// Texture still needs to be uploaded.
    Pending(Box<T>),
    /// Texture is already uploaded and we've got a reference.
    Uploaded(TextureRef),
}

impl<T: Texture> PendingOrUploaded<T> {
    /// Setup the texture type for uploading.
    pub fn new(texture: T) -> Self {
        Self::Pending(Box::new(texture))
    }

    /// Get the reference as an option, if not uploaded yet.
    pub fn as_ref(&self) -> Option<TextureRef> {
        match self {
            PendingOrUploaded::Pending(..) => None,
            PendingOrUploaded::Uploaded(texture_ref) => Some(*texture_ref),
        }
    }

    /// Upload to an atlas if it's not uploaded yet.
    pub fn upload(&mut self, atlas: &mut Atlas, queue: &wgpu::Queue) {
        if let Self::Uploaded(..) = self {
            return;
        }

        // First take the value, replacing it with a default ref we will overwrite
        let Self::Pending(texture) = std::mem::take(self) else {
            unreachable!()
        };

        // Upload the texture to the atlas
        let texture_ref = atlas.add(*texture, queue);

        *self = PendingOrUploaded::Uploaded(texture_ref);
    }
}

impl<T: Texture> Texture for PendingOrUploaded<T> {
    /// Throw an error when the inner type is already uploaded.
    fn size(&self) -> Size2<u32> {
        match self {
            PendingOrUploaded::Pending(pending) => pending.size(),
            PendingOrUploaded::Uploaded(..) => panic!("Texture is already uploaded"),
        }
    }

    /// Throw an error when the inner type is already uploaded.
    fn to_rgba_image(&mut self) -> Vec<u8> {
        match self {
            PendingOrUploaded::Pending(pending) => pending.to_rgba_image(),
            PendingOrUploaded::Uploaded(..) => panic!("Texture is already uploaded"),
        }
    }
}

impl<T: Texture> Default for PendingOrUploaded<T> {
    fn default() -> Self {
        Self::Uploaded(0)
    }
}
