//! Types and helpers for drawing on the GPU.

pub(crate) mod atlas;
pub(crate) mod component;
pub(crate) mod data;
pub(crate) mod instance;
pub(crate) mod post_processing;
pub(crate) mod state;
pub(crate) mod texture;
pub(crate) mod uniform;

use self::state::PREFERRED_TEXTURE_FORMAT;
pub use self::{data::TexturedVertex, instance::Instances, texture::TextureRef};

use std::ops::Range;

use glam::Affine2;
use wgpu::{Color, PrimitiveTopology};

/// Allow something to be rendered on the GPU.
pub trait Render {
    /// Whether the mesh needs to be updated on the GPU.
    ///
    /// This is not influenced by instancing.
    fn is_dirty(&self) -> bool;

    /// Tell the object everything is up to date.
    fn mark_clean(&mut self);

    /// All transformations of the instances of this type to render.
    fn instances_mut(&mut self) -> &mut Instances;

    /// All vertices of this type to render.
    fn vertices(&self) -> &[TexturedVertex];

    /// All indices of this type to render.
    fn indices(&self) -> &[u16];

    /// Range of indices to draw.
    fn range(&self) -> Range<u32>;

    /// Texture reference to bind and render.
    ///
    /// If `None` no texture binding will be applied.
    fn texture(&self) -> Option<TextureRef> {
        None
    }

    /// Called just before rendering the objects.
    ///
    /// Can be overwritten to handle some simple logic.
    fn pre_render(&mut self) {}

    /// Called just after rendering the objects.
    ///
    /// Can be overwritten to handle some simple logic.
    /// It should clear the instances when overwritten.
    fn post_render(&mut self) {
        self.instances_mut().clear();
    }

    /// Draw an instance of this object.
    ///
    /// # Arguments
    ///
    /// * `transformation` - Absolute 2D transformation matrix of where the instance should be drawn.
    fn push_instance(&mut self, transformation: Affine2) {
        self.instances_mut().push(transformation);
    }

    /// Definition of the primitive type.
    ///
    /// Defaults to [`wgpu::PrimitiveTopology::TriangleList`].
    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }
}

/// Convert an `u32` color to a WGPU [`wgpu::Color`] taking in account sRGB.
fn u32_to_wgpu_color(argb: u32) -> Color {
    let a = ((argb & 0xFF000000) >> 24) as f64 / 255.0;
    let r = ((argb & 0x00FF0000) >> 16) as f64 / 255.0;
    let g = ((argb & 0x0000FF00) >> 8) as f64 / 255.0;
    let b = (argb & 0x000000FF) as f64 / 255.0;

    if PREFERRED_TEXTURE_FORMAT.is_srgb() {
        // Convert to sRGB space
        let a = a.powf(2.2);
        let r = r.powf(2.2);
        let g = g.powf(2.2);
        let b = b.powf(2.2);

        Color { a, r, g, b }
    } else {
        Color { a, r, g, b }
    }
}
