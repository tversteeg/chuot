//! Types and helpers for drawing on the GPU.

pub(crate) mod component;
pub(crate) mod context;
pub(crate) mod data;
pub(crate) mod instance;
pub(crate) mod state;
pub(crate) mod texture;

pub use self::{data::TexturedVertex, instance::Instances, texture::TextureRef};

use std::ops::Range;

use vek::Mat3;
use wgpu::PrimitiveTopology;

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
    fn texture(&self) -> Option<&TextureRef> {
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
    fn push_instance(&mut self, transformation: Mat3<f64>) {
        self.instances_mut().push(transformation.as_());
    }

    /// Definition of the primitive type.
    ///
    /// Defaults to [`wgpu::PrimitiveTopology::TriangleList`].
    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }
}
